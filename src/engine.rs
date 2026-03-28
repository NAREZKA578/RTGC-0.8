use winit::{
    event::{WindowEvent, ElementState, MouseButton},
    keyboard::{KeyCode, PhysicalKey},
};
use std::sync::Arc;
use crate::config::Config;
use crate::graphics::GlContext;
use crate::graphics::material::MaterialManager;
use crate::graphics::mesh::Mesh;
use crate::input::InputManager;
use crate::audio::AudioSystem;
use crate::ecs::EcsManager;
use crate::physics;
use crate::graphics::renderer::{MenuState, Renderer};
use crate::game::{WeatherSystem, Cargo, Winch, MissionGenerator, Mission, MainMenu};
use crate::world::DayNightCycle;
use crate::graphics::particles::ParticleSystem;
use crate::graphics::debug_renderer::DebugRenderer;
use crate::profiler;
use crate::ui::HudManager;
use crate::assets::VehicleLoader;
use crate::world::{OpenWorld, CHUNK_SIZE, HEIGHTMAP_RESOLUTION, ChunkId, generate_chunk_mesh, TerrainVertex};
use crate::world::{Settlement, RoadNetwork, BuildingPlacer};
use nalgebra::{Vector3, UnitQuaternion, Matrix4};
use crate::physics::Vehicle;
use crate::network::PlayerInput;
use winit::event_loop::ActiveEventLoop;
use winit::window::WindowId;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent as WinitWindowEvent;

/// Состояния игры
#[derive(Debug, Clone, PartialEq)]
pub enum GameState {
    MainMenu,
    Loading,
    Playing,
    Paused,
}

// Fixed timestep for physics (60 Hz)
const PHYSICS_TIMESTEP: f32 = 1.0 / 60.0;

pub struct Engine {
    pub graphics_context: GlContext,
    pub input_manager: InputManager,
    pub audio_system: AudioSystem,
    pub ecs_manager: EcsManager,
    pub physics_world: physics::PhysicsWorld,
    last_frame_time: std::time::Instant,
    physics_accumulator: f32,
    physics_timestep: f32,
    hud_manager: HudManager,
    material_manager: MaterialManager,
    // Рендерер для отрисовки 3D сцены
    renderer: Option<Renderer>,
    // Главное меню
    main_menu: MainMenu,
    // Состояние игры
    game_state: GameState,
    open_world: Option<OpenWorld>,
    world_seed: u64,
    settlements: Vec<Settlement>,
    road_network: Option<RoadNetwork>,
    mission_generator: Option<MissionGenerator>,
    current_mission: Option<Mission>,
    helicopter: Option<crate::physics::Helicopter>,
    compass_heading: f32,
    vehicle_chassis_id: Option<usize>,
    chunk_mesh_data: Option<(Vec<f32>, Vec<u32>)>,
    weather_system: WeatherSystem,
    day_night_cycle: DayNightCycle,
    particle_system: ParticleSystem,
    debug_renderer: DebugRenderer,
    debug_mode: bool,
    cargo: Option<Cargo>,
    winch: Winch,
    vehicle_throttle: f32,
    vehicle_steering: f32,
    vehicle_brake: f32,
    vehicle: Option<Vehicle>,
    vehicle_health: f32,
    vehicle_fuel: f32,
    save_timer: f32,
    mouse_x: f32,
    mouse_y: f32,
}

impl Engine {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // DEBUG: Инициализация движка
        eprintln!("DEBUG [engine]: Engine::new() - Starting initialization");
        
        // Загрузка конфигурации
        let config = Config::load("config.json").unwrap_or_else(|_| {
            tracing::warn!("Не удалось загрузить config.json, используются настройки по умолчанию");
            Config::default()
        });

        eprintln!("DEBUG [engine]: Config loaded, creating graphics context");
        
        // GlContext будет создан в resumed() через init_window
        let graphics_context = GlContext::new_placeholder();
        let input_manager = InputManager::new();
        eprintln!("DEBUG [engine]: Input manager created");
        
        let audio_system = AudioSystem::new()?;
        eprintln!("DEBUG [engine]: Audio system created");
        
        let ecs_manager = EcsManager::new();
        let physics_world = physics::PhysicsWorld::new();
        let hud_manager = HudManager::new();
        let material_manager = MaterialManager::new(crate::graphics::material::TextureQuality::Medium);
        let particle_system = ParticleSystem::new(1000);
        let debug_renderer = DebugRenderer::new();
        let weather_system = WeatherSystem::new(42);  // seed для погоды
        let day_night_cycle = DayNightCycle::new(55.0, 82.9);  // широта и долгота (Новосибирск)
        let winch = Winch::new(0);  // индекс тела транспортного средства

        Ok(Self {
            graphics_context,
            input_manager,
            audio_system,
            ecs_manager,
            physics_world,
            last_frame_time: std::time::Instant::now(),
            physics_accumulator: 0.0,
            physics_timestep: PHYSICS_TIMESTEP,
            hud_manager,
            material_manager,
            renderer: None,  // Будет инициализирован в resumed()
            main_menu: MainMenu::new(),
            game_state: GameState::MainMenu,
            open_world: None,
            world_seed: 42,
            settlements: Vec::new(),
            road_network: None,
            mission_generator: None,
            current_mission: None,
            helicopter: None,
            compass_heading: 0.0,
            vehicle_chassis_id: None,
            chunk_mesh_data: None,
            weather_system,
            day_night_cycle,
            particle_system,
            debug_renderer,
            debug_mode: false,
            cargo: None,
            winch,
            vehicle_throttle: 0.0,
            vehicle_steering: 0.0,
            vehicle_brake: 0.0,
            vehicle: None,
            vehicle_health: 100.0,
            vehicle_fuel: 100.0,
            save_timer: 0.0,
            mouse_x: 0.0,
            mouse_y: 0.0,
        })
    }

    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        use winit::event_loop::ControlFlow;
        
        let event_loop = winit::event_loop::EventLoop::new()?;
        event_loop.set_control_flow(ControlFlow::Poll);
        
        // Инициализация мира
        self.open_world = Some(OpenWorld::new(self.world_seed));
        
        // Загрузка начальных данных будет вызвана в resumed()
        
        event_loop.run_app(self)?;
        
        Ok(())
    }

    pub fn update(&mut self, dt: f32) -> Result<(), Box<dyn std::error::Error>> {
        // DEBUG: Отладка обновления движка
        eprintln!("DEBUG [engine]: Engine::update() - dt={:.4}", dt);
        
        profiler::begin_frame();
        let _profile = profiler::ProfileScope::new("Engine::update");

        // Обновление ввода
        self.input_manager.update();

        // Обновление hover-эффектов меню
        if self.game_state == GameState::MainMenu {
            self.update_menu_hover();
        }

        // Физический шаг с фиксированным timestep
        self.physics_accumulator += dt;
        while self.physics_accumulator >= self.physics_timestep {
            self.physics_step(self.physics_timestep)?;
            self.physics_accumulator -= self.physics_timestep;
        }
        
        // Обновление систем
        self.ecs_manager.update(dt);
        let current_hour = self.day_night_cycle.get_hour();
        self.weather_system.update(dt, current_hour);
        self.day_night_cycle.advance_time(dt);
        self.particle_system.update(dt);
        
        // Обновление HUD с данными автомобиля
        if let Some(ref vehicle) = self.vehicle {
            let hud_data = crate::ui::hud::VehicleHudData {
                speed_kmh: vehicle.speed() * 3.6, // m/s to km/h
                engine_rpm: 2000.0, // заглушка, пока нет метода get_engine_rpm
                fuel_level: self.vehicle_fuel / 100.0,
                vehicle_health: self.vehicle_health / 100.0,
                ..Default::default()
            };
            self.hud_manager.update(hud_data, dt);
        }
        
        // Обновление вертолета
        if let Some(ref mut heli) = self.helicopter {
            heli.update(dt);
        }
        
        // Обновление транспортного средства
        if let Some(ref mut vehicle) = self.vehicle {
            // Сначала устанавливаем управление
            let controls = crate::physics::vehicle::VehicleControls::new(
                self.vehicle_throttle,
                self.vehicle_brake,
                self.vehicle_steering,
                0.0, // handbrake
            );
            vehicle.set_controls(controls);
            
            // Затем обновляем физику
            let terrain_getter = |x: f32, z: f32| -> f32 {
                if let Some(ref open_world) = self.open_world {
                    // open_world.get_height(x, z) // TODO: method not found
                    0.0
                } else {
                    0.0
                }
            };
            let surface_getter = |_x: f32, _z: f32| -> crate::world::terrain_generator::SurfaceType {
                crate::world::terrain_generator::SurfaceType::Grass
            };
            vehicle.update(dt, terrain_getter, surface_getter);
        }
        
        // Обновление лебедки
        // if let Some(ref mut cargo) = self.cargo {
        //     self.winch.update(dt, cargo);
        // }
        
        // Таймер автосохранения
        self.save_timer += dt;
        if self.save_timer >= 60.0 {
            self.save_game()?;
            self.save_timer = 0.0;
        }
        
        profiler::end_frame();
        Ok(())
    }

    pub fn render(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // DEBUG: Отладка рендеринга
        eprintln!("DEBUG [engine]: Engine::render() - game_state={:?}", self.game_state);
        
        let _profile = profiler::ProfileScope::new("Engine::render");

        // Рендеринг через Renderer
        if let Some(ref mut renderer) = self.renderer {
            eprintln!("DEBUG [engine]: Renderer exists, updating menu_state");
            
            // Обновление позиции мыши для UI
            renderer.mouse_x = self.mouse_x;
            renderer.mouse_y = self.mouse_y;
            
            // Обновление камеры в рендерере
            if let Some(ref heli) = self.helicopter {
                let heli_pos = heli.position;
                renderer.camera.position = heli_pos;
            }

            // Обновление menu_state в renderer на основе game_state
            renderer.menu_state = match self.game_state {
                GameState::MainMenu => {
                    eprintln!("DEBUG [engine]: Setting MenuState::MainMenu");
                    MenuState::MainMenu
                },
                GameState::Loading => {
                    eprintln!("DEBUG [engine]: Setting MenuState::Loading");
                    MenuState::Loading
                },
                GameState::Playing => {
                    eprintln!("DEBUG [engine]: Setting MenuState::InGame");
                    MenuState::InGame
                },
                GameState::Paused => {
                    eprintln!("DEBUG [engine]: Setting MenuState::Paused");
                    MenuState::Paused
                },
            };

            // Вызов рендера (Renderer сам очищает экран и рисует всё)
            eprintln!("DEBUG [engine]: Calling renderer.render()");
            renderer.render()?;
            
            // Рендеринг частиц (поверх сцены)
            if let Some(ref gl) = self.graphics_context.gl {
                if self.game_state == GameState::Playing {
                    // Получение матриц вида и проекции
                    let view_matrix = Matrix4::identity();
                    let proj_matrix = self.graphics_context.get_projection_matrix(std::f32::consts::PI / 4.0, 0.1, 1000.0);
                    let view_proj = proj_matrix * view_matrix;
                    
                    self.particle_system.render(gl, view_proj);
                    
                    // Отладочный рендеринг
                    if self.debug_mode {
                        self.debug_renderer.flush_to_gl(gl, view_proj);
                    }
                }
            }
            
            // Рендеринг HUD
            if self.game_state == GameState::Playing || self.game_state == GameState::Paused {
                // Обновление данных HUD
                if let Some(ref vehicle) = self.vehicle {
                    let hud_data = crate::ui::hud::VehicleHudData {
                        speed_kmh: vehicle.speed() * 3.6,
                        engine_rpm: 2000.0,
                        fuel_level: self.vehicle_fuel / 100.0,
                        vehicle_health: self.vehicle_health / 100.0,
                        heading_degrees: self.compass_heading,
                        ..Default::default()
                    };
                    self.hud_manager.update(hud_data, 0.016);
                } else if let Some(ref heli) = self.helicopter {
                    let hud_data = crate::ui::hud::VehicleHudData {
                        speed_kmh: heli.velocity.norm() * 3.6,
                        altitude_m: heli.position.y,
                        heading_degrees: self.compass_heading,
                        ..Default::default()
                    };
                    self.hud_manager.update(hud_data, 0.016);
                }
                
                // Отрисовка HUD через Renderer
                self.hud_manager.render(renderer);
            }
            
            // Завершение кадра через Renderer (swap buffers)
            self.graphics_context.end_frame()?;
        } else if let Some(ref gl) = self.graphics_context.gl {
            // Фолбэк рендеринг если Renderer не инициализирован
            self.graphics_context.begin_frame()?;
            
            let view_matrix = Matrix4::identity();
            let proj_matrix = self.graphics_context.get_projection_matrix(std::f32::consts::PI / 4.0, 0.1, 1000.0);
            let view_proj = proj_matrix * view_matrix;
            
            self.particle_system.render(gl, view_proj);
            
            if self.debug_mode {
                self.debug_renderer.flush_to_gl(gl, view_proj);
            }
            
            self.graphics_context.end_frame()?;
        }

        Ok(())
    }

    pub fn handle_event(&mut self, event: &WindowEvent) -> Result<(), Box<dyn std::error::Error>> {
        match event {
            WindowEvent::CloseRequested => {
                return Err("Window closed".into());
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if event.state == ElementState::Pressed {
                    if let PhysicalKey::Code(key_code) = event.physical_key {
                        // Обработка событий главного меню
                        if self.game_state == GameState::MainMenu {
                            match key_code {
                                KeyCode::Enter => {
                                    // Начать новую игру
                                    self.main_menu.start_new_game();
                                    eprintln!("DEBUG: Starting new game - character creation");
                                }
                                KeyCode::Escape => {
                                    // Выход из игры
                                    return Err("Escape pressed in main menu".into());
                                }
                                _ => {}
                            }
                        }
                        
                        match key_code {
                            // Настройки: Открытие/закрытие настроек на Escape (если открыты настройки или инвентарь)
                            KeyCode::Escape => {
                                // Если открыты настройки или инвентарь - закрываем их
                                if self.hud_manager.is_settings_open() || self.hud_manager.is_inventory_open() {
                                    self.hud_manager.set_settings_open(false);
                                    self.hud_manager.set_inventory_open(false);
                                } else if self.game_state == GameState::Playing {
                                    // Пауза в игре
                                    self.game_state = GameState::Paused;
                                    self.hud_manager.toggle_settings();
                                } else if self.game_state == GameState::Paused {
                                    // Возобновить игру
                                    self.game_state = GameState::Playing;
                                    self.hud_manager.set_settings_open(false);
                                } else {
                                    // Иначе выход из игры
                                    return Err("Escape pressed".into());
                                }
                            }
                            KeyCode::F3 => {
                                self.debug_mode = !self.debug_mode;
                            }
                            // Ф1.6: Открытие инвентаря на Tab
                            KeyCode::Tab => {
                                // Не открывать инвентарь если открыты настройки
                                if !self.hud_manager.is_settings_open() && self.game_state == GameState::Playing {
                                    self.hud_manager.toggle_inventory();
                                }
                            }
                            // Настройки: Открытие настроек на F1
                            KeyCode::F1 => {
                                // Не открывать настройки если открыт инвентарь
                                if !self.hud_manager.is_inventory_open() && self.game_state == GameState::Playing {
                                    self.hud_manager.toggle_settings();
                                }
                            }
                            KeyCode::KeyW => self.input_manager.set_key_state(PhysicalKey::Code(KeyCode::KeyW), true),
                            KeyCode::KeyS => self.input_manager.set_key_state(PhysicalKey::Code(KeyCode::KeyS), true),
                            KeyCode::KeyA => self.input_manager.set_key_state(PhysicalKey::Code(KeyCode::KeyA), true),
                            KeyCode::KeyD => self.input_manager.set_key_state(PhysicalKey::Code(KeyCode::KeyD), true),
                            KeyCode::Space => self.input_manager.set_key_state(PhysicalKey::Code(KeyCode::Space), true),
                            KeyCode::ShiftLeft => self.input_manager.set_key_state(PhysicalKey::Code(KeyCode::ShiftLeft), true),
                            _ => {}
                        }
                    }
                } else if event.state == ElementState::Released {
                    if let PhysicalKey::Code(key_code) = event.physical_key {
                        match key_code {
                            KeyCode::KeyW => self.input_manager.set_key_state(PhysicalKey::Code(KeyCode::KeyW), false),
                            KeyCode::KeyS => self.input_manager.set_key_state(PhysicalKey::Code(KeyCode::KeyS), false),
                            KeyCode::KeyA => self.input_manager.set_key_state(PhysicalKey::Code(KeyCode::KeyA), false),
                            KeyCode::KeyD => self.input_manager.set_key_state(PhysicalKey::Code(KeyCode::KeyD), false),
                            KeyCode::Space => self.input_manager.set_key_state(PhysicalKey::Code(KeyCode::Space), false),
                            KeyCode::ShiftLeft => self.input_manager.set_key_state(PhysicalKey::Code(KeyCode::ShiftLeft), false),
                            _ => {}
                        }
                    }
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.mouse_x = position.x as f32;
                self.mouse_y = position.y as f32;
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if *button == winit::event::MouseButton::Left {
                    self.input_manager.set_mouse_button_state(MouseButton::Left.into(), *state == ElementState::Pressed);
                    
                    // DEBUG: Обработка клика по кнопкам меню
                    if *state == ElementState::Pressed && self.game_state == GameState::MainMenu {
                        self.handle_menu_click();
                    }
                } else if *button == winit::event::MouseButton::Right {
                    self.input_manager.set_mouse_button_state(MouseButton::Right.into(), *state == ElementState::Pressed);
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn physics_step(&mut self, dt: f32) -> Result<(), Box<dyn std::error::Error>> {
        // Обновление физики транспорта
        // if let Some(ref mut vehicle) = self.vehicle {
        //     vehicle.physics_update(dt, &mut self.physics_world);
        // }

        // Обновление физики вертолета
        // if let Some(ref mut heli) = self.helicopter {
        //     heli.physics_update(dt, &mut self.physics_world);
        // }

        // Шаг физического мира
        self.physics_world.step(dt);

        // Обновление лебедки и груза
        // if let Some(ref mut cargo) = self.cargo {
        //     self.winch.physics_update(dt, cargo, &mut self.physics_world);
        // }

        Ok(())
    }

    /// Обработка кликов по кнопкам главного меню
    fn handle_menu_click(&mut self) {
        // DEBUG: Обработка клика по кнопкам меню
        eprintln!("DEBUG [engine]: handle_menu_click() - mouse=({}, {})", self.mouse_x, self.mouse_y);
        
        let w = self.renderer.as_ref().map(|r| r.width as f32).unwrap_or(800.0);
        let h = self.renderer.as_ref().map(|r| r.height as f32).unwrap_or(600.0);
        
        // Координаты кнопок меню (должны совпадать с renderer.rs)
        let button_width = 240.0;
        let button_height = 40.0;
        let center_x = w / 2.0;
        
        // Позиции кнопок по Y
        let new_game_y = h / 2.0 - 80.0;
        let continue_y = h / 2.0 - 30.0;
        let settings_y = h / 2.0 + 20.0;
        let exit_y = h / 2.0 + 70.0;
        
        // Проверка попадания клика по кнопкам
        let click_x = self.mouse_x;
        let click_y = h - self.mouse_y; // Инвертируем Y для OpenGL
        
        // "Новая игра"
        if click_x >= center_x - button_width / 2.0 && click_x <= center_x + button_width / 2.0
            && click_y >= new_game_y && click_y <= new_game_y + button_height {
            eprintln!("DEBUG [engine]: Menu click - NEW GAME");
            self.game_state = GameState::Playing;
            if let Some(ref mut renderer) = self.renderer {
                renderer.menu_state = crate::graphics::renderer::MenuState::InGame;
            }
            return;
        }
        
        // "Продолжить"
        if click_x >= center_x - button_width / 2.0 && click_x <= center_x + button_width / 2.0
            && click_y >= continue_y && click_y <= continue_y + button_height {
            eprintln!("DEBUG [engine]: Menu click - CONTINUE");
            // Загрузка последнего сохранения
            if let Some(save_path) = self.main_menu.continue_game() {
                eprintln!("DEBUG [engine]: Loading save from: {:?}", save_path);
                self.game_state = GameState::Playing;
            } else {
                eprintln!("DEBUG [engine]: No saves found");
            }
            return;
        }
        
        // "Настройки"
        if click_x >= center_x - button_width / 2.0 && click_x <= center_x + button_width / 2.0
            && click_y >= settings_y && click_y <= settings_y + button_height {
            eprintln!("DEBUG [engine]: Menu click - SETTINGS (not implemented)");
            return;
        }
        
        // "Выход"
        if click_x >= center_x - button_width / 2.0 && click_x <= center_x + button_width / 2.0
            && click_y >= exit_y && click_y <= exit_y + button_height {
            eprintln!("DEBUG [engine]: Menu click - EXIT");
            // Сигнал на выход из приложения
            std::process::exit(0);
        }
    }

    /// Обновление hover-эффектов для кнопок меню
    fn update_menu_hover(&mut self) {
        let w = self.renderer.as_ref().map(|r| r.width as f32).unwrap_or(800.0);
        let h = self.renderer.as_ref().map(|r| r.height as f32).unwrap_or(600.0);
        
        let button_width = 240.0;
        let button_height = 40.0;
        let center_x = w / 2.0;
        
        let new_game_y = h / 2.0 - 80.0;
        let continue_y = h / 2.0 - 30.0;
        let settings_y = h / 2.0 + 20.0;
        let exit_y = h / 2.0 + 70.0;
        
        let mouse_x = self.mouse_x;
        let mouse_y = h - self.mouse_y;
        
        // Проверка наведения на кнопки
        let hovered = if mouse_x >= center_x - button_width / 2.0 && mouse_x <= center_x + button_width / 2.0 {
            if mouse_y >= new_game_y && mouse_y <= new_game_y + button_height {
                Some("Новая игра")
            } else if mouse_y >= continue_y && mouse_y <= continue_y + button_height {
                Some("Продолжить")
            } else if mouse_y >= settings_y && mouse_y <= settings_y + button_height {
                Some("Настройки")
            } else if mouse_y >= exit_y && mouse_y <= exit_y + button_height {
                Some("Выход")
            } else {
                None
            }
        } else {
            None
        };
        
        if let Some(button) = hovered {
            eprintln!("DEBUG [engine]: Menu hover - {}", button);
        }
    }

    fn load_initial_data(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Загрузка начальных данных игры
        // Генерация мира
        if let Some(ref mut open_world) = self.open_world {
            // open_world.generate_terrain(); // TODO: method not found
        }

        // Создание поселений
        if let Some(ref open_world) = self.open_world {
            self.settlements = Self::generate_settlements_simple(open_world, 5);
        }

        // Создание дорожной сети - используем правильный метод generate_from_settlements
        if !self.settlements.is_empty() {
            let seed = self.world_seed;
            self.road_network = Some(RoadNetwork::generate_from_settlements(&self.settlements, seed));
        }

        // Создание генератора миссий
        if let Some(road_network) = &self.road_network {
            let settlements_clone = self.settlements.clone();
            let road_network_clone = road_network.clone();
            self.mission_generator = Some(MissionGenerator::new(settlements_clone, road_network_clone, self.world_seed));
        }

        // Создание начальной миссии
        if let Some(ref mut generator) = self.mission_generator {
            self.current_mission = generator.generate_mission(Vector3::zeros());
        }

        // Создание игрока (вертолет или транспортное средство)
        self.create_player_vehicle()?;

        Ok(())
    }

    fn generate_settlements_simple(open_world: &OpenWorld, count: usize) -> Vec<Settlement> {
        use rand::{Rng, SeedableRng};
        use rand_chacha::ChaCha8Rng;
        
        let mut rng = ChaCha8Rng::seed_from_u64(open_world.seed);
        let mut settlements = Vec::new();
        
        for i in 0..count {
            let grid_x = (rng.gen::<f32>() * 10.0) as i32;
            let grid_z = (rng.gen::<f32>() * 10.0) as i32;
            let center_x = grid_x as f32 * CHUNK_SIZE as f32;
            let center_z = grid_z as f32 * CHUNK_SIZE as f32;
            
            // Используем правильный метод генерации поселений
            if let Some(settlement) = Settlement::generate(open_world.seed + i as u64, grid_x, grid_z, center_x, center_z) {
                settlements.push(settlement);
            }
        }
        
        settlements
    }

    fn create_player_vehicle(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Создаем начальное транспортное средство для игрока
        let spawn_pos = nalgebra::Vector3::new(0.0, 100.0, 0.0);

        // По умолчанию создаем вертолет
        self.helicopter = Some(crate::physics::Helicopter::new(spawn_pos));

        // Альтернативно можно создать наземное транспортное средство
        // self.vehicle = Some(Vehicle::new(spawn_pos));
        // self.vehicle_chassis_id = Some(self.vehicle.as_ref().unwrap().chassis_body_id);

        Ok(())
    }

    fn save_game(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Сохранение состояния игры
        use std::fs::File;
        use std::io::Write;
        
        let save_data = format!(
            "seed={}\nvehicle_health={}\nvehicle_fuel={}\ncompass_heading={}\n",
            self.world_seed,
            self.vehicle_health,
            self.vehicle_fuel,
            self.compass_heading
        );
        
        let mut file = File::create("savegame.dat")?;
        file.write_all(save_data.as_bytes())?;
        
        Ok(())
    }
}

impl ApplicationHandler for Engine {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Инициализация GL контекста при возобновлении
        let window_attrs = winit::window::WindowAttributes::default()
            .with_inner_size(winit::dpi::LogicalSize::new(1280, 720))
            .with_title("RTGC");

        match GlContext::new(event_loop, window_attrs) {
            Ok(ctx) => {
                // Инициализация Renderer после создания контекста
                if let Some(ref gl) = ctx.gl {
                    match Renderer::new(gl.clone()) {
                        Ok(renderer) => {
                            self.renderer = Some(renderer);
                            eprintln!("DEBUG: Renderer initialized successfully");
                        }
                        Err(e) => {
                            eprintln!("ERROR: Failed to initialize Renderer: {}", e);
                        }
                    }
                }
                
                self.graphics_context = ctx;

                // Загрузка начальных данных
                if let Err(e) = self.load_initial_data() {
                    eprintln!("ERROR: Failed to load initial data: {}", e);
                    eprintln!("ERROR: Backtrace: {:?}", std::backtrace::Backtrace::force_capture());
                }
            }
            Err(e) => {
                eprintln!("ERROR: Failed to initialize window: {}", e);
                eprintln!("ERROR: Backtrace: {:?}", std::backtrace::Backtrace::force_capture());
                event_loop.exit();
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WinitWindowEvent) {
        match event {
            WinitWindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WinitWindowEvent::Destroyed => {
                event_loop.exit();
            }
            WinitWindowEvent::KeyboardInput { event, .. } => {
                if event.state == ElementState::Pressed {
                    if let PhysicalKey::Code(key_code) = event.physical_key {
                        match key_code {
                            KeyCode::Escape => {
                                event_loop.exit();
                            }
                            KeyCode::F3 => {
                                self.debug_mode = !self.debug_mode;
                            }
                            KeyCode::KeyW => self.input_manager.set_key_state(PhysicalKey::Code(KeyCode::KeyW), true),
                            KeyCode::KeyS => self.input_manager.set_key_state(PhysicalKey::Code(KeyCode::KeyS), true),
                            KeyCode::KeyA => self.input_manager.set_key_state(PhysicalKey::Code(KeyCode::KeyA), true),
                            KeyCode::KeyD => self.input_manager.set_key_state(PhysicalKey::Code(KeyCode::KeyD), true),
                            KeyCode::Space => self.input_manager.set_key_state(PhysicalKey::Code(KeyCode::Space), true),
                            KeyCode::ShiftLeft => self.input_manager.set_key_state(PhysicalKey::Code(KeyCode::ShiftLeft), true),
                            _ => {}
                        }
                    }
                } else if event.state == ElementState::Released {
                    if let PhysicalKey::Code(key_code) = event.physical_key {
                        match key_code {
                            KeyCode::KeyW => self.input_manager.set_key_state(PhysicalKey::Code(KeyCode::KeyW), false),
                            KeyCode::KeyS => self.input_manager.set_key_state(PhysicalKey::Code(KeyCode::KeyS), false),
                            KeyCode::KeyA => self.input_manager.set_key_state(PhysicalKey::Code(KeyCode::KeyA), false),
                            KeyCode::KeyD => self.input_manager.set_key_state(PhysicalKey::Code(KeyCode::KeyD), false),
                            KeyCode::Space => self.input_manager.set_key_state(PhysicalKey::Code(KeyCode::Space), false),
                            KeyCode::ShiftLeft => self.input_manager.set_key_state(PhysicalKey::Code(KeyCode::ShiftLeft), false),
                            _ => {}
                        }
                    }
                }
            }
            WinitWindowEvent::CursorMoved { position, .. } => {
                self.mouse_x = position.x as f32;
                self.mouse_y = position.y as f32;
            }
            WinitWindowEvent::MouseInput { state, button, .. } => {
                if button == winit::event::MouseButton::Left {
                    self.input_manager.set_mouse_button_state(MouseButton::Left.into(), state == ElementState::Pressed);
                } else if button == winit::event::MouseButton::Right {
                    self.input_manager.set_mouse_button_state(MouseButton::Right.into(), state == ElementState::Pressed);
                }
            }
            WinitWindowEvent::Resized(new_size) => {
                if let Err(e) = self.graphics_context.resize(new_size.width, new_size.height) {
                    eprintln!("ERROR: Resize error: {}", e);
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        let now = std::time::Instant::now();
        let dt = (now - self.last_frame_time).as_secs_f32();
        self.last_frame_time = now;

        if let Err(e) = self.update(dt) {
            eprintln!("ERROR: Update error: {}", e);
            eprintln!("ERROR: Backtrace: {:?}", std::backtrace::Backtrace::force_capture());
        }

        // Пропускаем рендеринг если контекст ещё не инициализирован
        if self.graphics_context.is_initialized() {
            if let Err(e) = self.render() {
                eprintln!("ERROR: Render error: {}", e);
                eprintln!("ERROR: Backtrace: {:?}", std::backtrace::Backtrace::force_capture());
            }
        }
    }
}
