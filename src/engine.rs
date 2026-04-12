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
use crate::game::loading_manager::{LoadingManager, ResourceType, LoadingProgress, LoadingStats};
use crate::game::interaction::InteractionSystem;
use crate::game::debug_menu::DebugMenu;
use crate::game::asset_manager::AssetManager;
use crate::game::save::SaveSystem;
use crate::game::ui::UIManager;
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
    CharacterCreation,
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
    tracked_vehicle: Option<crate::physics::TrackedVehicle>,
    tracked_vehicle_throttle: f32,
    tracked_vehicle_brake: f32,
    tracked_vehicle_turn: f32,
    save_timer: f32,
    mouse_x: f32,
    mouse_y: f32,
    // Проблема 8: Interaction system
    interaction_system: InteractionSystem,
    // Проблема 6: Debug menu
    debug_menu: DebugMenu,
    // Проблема 4: Asset manager
    asset_manager: AssetManager,
    // Проблема 3: UI Manager
    ui_manager: UIManager,
    // Проблема 9: Save system
    save_system: SaveSystem,
    // Менеджер загрузки
    loading_manager: LoadingManager,
    // Персонаж игрока
    player: Option<crate::game::player::Player>,
    // Менеджер создания персонажа
    character_creation: crate::game::character_creation::CharacterCreationManager,
}

impl Engine {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Загрузка конфигурации
        let config = Config::load("config.json").unwrap_or_else(|_| {
            tracing::warn!("Не удалось загрузить config.json, используются настройки по умолчанию");
            Config::default()
        });

        // GlContext будет создан в resumed() через init_window
        let graphics_context = GlContext::new_placeholder();
        let input_manager = InputManager::new();

        let audio_system = AudioSystem::new()?;

        let ecs_manager = EcsManager::new();
        let physics_world = physics::PhysicsWorld::new();
        let hud_manager = HudManager::new();
        let material_manager = MaterialManager::new(crate::graphics::material::TextureQuality::Medium);
        let particle_system = ParticleSystem::new(1000);
        let debug_renderer = DebugRenderer::new();
        let weather_system = WeatherSystem::new(42);  // seed для погоды
        let day_night_cycle = DayNightCycle::new(55.0, 82.9);  // широта и долгота (Новосибирск)
        let winch = Winch::new(0);  // индекс тела транспортного средства
        // Проблема 8: Interaction system
        let interaction_system = InteractionSystem::new();
        // Проблема 6: Debug menu
        let debug_menu = DebugMenu::new();
        // Проблема 4: Asset manager
        let asset_manager = AssetManager::default();
        // Проблема 3: UI manager
        let ui_manager = UIManager::new();
        // Проблема 9: Save system
        let save_system = SaveSystem::default();
        // Менеджер загрузки
        let mut loading_manager = LoadingManager::new("assets");
        // Персонаж игрока (пока None)
        let player: Option<crate::game::player::Player> = None;
        // Менеджер создания персонажа
        let character_creation = crate::game::character_creation::CharacterCreationManager::new();

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
            tracked_vehicle: None,
            tracked_vehicle_throttle: 0.0,
            tracked_vehicle_brake: 0.0,
            tracked_vehicle_turn: 0.0,
            save_timer: 0.0,
            mouse_x: 0.0,
            mouse_y: 0.0,
            interaction_system,
            debug_menu,
            asset_manager,
            ui_manager,
            save_system,
            loading_manager,
            player,
            character_creation,
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
        profiler::begin_frame();
        let _profile = profiler::ProfileScope::new("Engine::update");

        // Обновление ввода
        self.input_manager.update();

        // Обновление main_menu
        if self.game_state == GameState::MainMenu {
            self.main_menu.update(dt);
            self.update_menu_hover();
        }

        // Обновление создания персонажа
        if self.game_state == GameState::CharacterCreation {
            self.update_character_creation_input();
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

        // Обновление персонажа игрока
        if let Some(ref mut player) = self.player {
            if player.state == crate::game::player::PlayerState::OnFoot {
                // Обновляем стамина
                player.update_stamina(dt);
                
                // Обрабатываем ввод для персонажа
                if let Some(action_map) = &self.input_manager.action_map() {
                    player.process_input(action_map, &mut self.physics_world, dt);
                }
                
                // Синхронизируем позицию с физическим телом
                if let Some(body_idx) = player.body_index {
                    if let Some(body) = self.physics_world.get_body(body_idx) {
                        player.position = body.position;
                    }
                }
            }
        }

        // Проблема 8: Обновление interaction system
        if self.game_state == GameState::Playing {
            // Получаем позицию игрока и направление камеры
            let player_pos = if let Some(ref player) = self.player {
                if let Some(body_idx) = player.body_index {
                    if let Some(body) = self.physics_world.get_body(body_idx) {
                        body.position
                    } else {
                        Vector3::zeros()
                    }
                } else {
                    Vector3::zeros()
                }
            } else if let Some(ref heli) = self.helicopter {
                heli.position
            } else {
                Vector3::zeros()
            };
            let player_forward = Vector3::z(); // Helicopter не имеет метода forward()

            // Обновляем interaction system
            self.interaction_system.update(dt, player_pos, player_forward, 4.0);

            // Обработка взаимодействия по клавише F
            if self.input_manager.state().is_key_held(winit::keyboard::KeyCode::KeyF) {
                // Получаем состояние игрока из helicopter или vehicle
                let mut player_state = crate::game::player::PlayerState::OnFoot;
                if self.helicopter.is_some() {
                    player_state = crate::game::player::PlayerState::InVehicle {
                        vehicle_index: 0,
                        vehicle_id: 1,
                        seat_index: 0,
                    };
                }

                // Пробуем взаимодействовать
                let result = self.interaction_system.try_interact(&mut player_state);
                if result.success {
                    tracing::info!("Interaction: {}", result.message);
                }
            }
        }

        // Проблема 6: Обновление debug menu статистики
        if self.debug_mode {
            self.debug_menu.update_fps(1.0 / dt.max(0.0001), dt * 1000.0);
            // self.debug_menu.update_physics_stats(self.physics_world.stats()); // stats() не существует

            // Обновление chunk count - chunks не существует в OpenWorld
            // if let Some(ref open_world) = self.open_world {
            //     self.debug_menu.update_chunks(open_world.chunks.len());
            // }

            // RAM usage - заглушка (требует feature в windows crate)
            self.debug_menu.update_ram_usage(0.0);
        }

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
                    open_world.get_height_at(nalgebra::Vector3::new(x, 0.0, z))
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
        let _profile = profiler::ProfileScope::new("Engine::render");

        // Рендеринг через Renderer — один вызов, всё внутри
        if let Some(ref mut renderer) = self.renderer {
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
                GameState::MainMenu => MenuState::MainMenu,
                GameState::Loading => MenuState::Loading,
                GameState::Playing => MenuState::InGame,
                GameState::Paused => MenuState::Paused,
            };

            // Передача debug_mode в renderer
            renderer.debug_mode = self.debug_mode;

            // Один вызов render() — всё внутри, включая flush
            renderer.render()?;
            
            // Завершение кадра (swap buffers)
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
                                    tracing::info!("Starting new game - character creation");
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
        // Terrain getter для получения высоты местности через get_height_at
        let terrain_getter = |x: f32, z: f32| -> f32 {
            if let Some(ref open_world) = self.open_world {
                open_world.get_height_at(nalgebra::Vector3::new(x, 0.0, z))
            } else {
                0.0
            }
        };
        
        // Surface getter для получения типа поверхности
        let surface_getter = |x: f32, z: f32| -> crate::world::SurfaceType {
            if let Some(ref open_world) = self.open_world {
                open_world.get_surface_type_at(x, z)
            } else {
                crate::world::SurfaceType::Grass
            }
        };
        
        // Deformable terrain - создаём компонент один раз при загрузке
        // В реальном проекте это должно храниться в Engine и передаваться по ссылке
        let mut deformable_terrain = crate::physics::DeformableTerrainComponent::new(0, 64, 64);
        let deformable_terrain_ref: Option<&mut crate::physics::DeformableTerrainComponent> = Some(&mut deformable_terrain);

        // Обновление физики транспорта
        if let Some(ref mut vehicle) = self.vehicle {
            vehicle.physics_update(dt, &mut self.physics_world, &terrain_getter, &surface_getter, deformable_terrain_ref);
        }

        // Обновление физики гусеничного транспорта
        if let Some(ref mut tracked_vehicle) = self.tracked_vehicle {
            let controls = crate::physics::tracked_vehicle::TrackedControls::from_input(
                self.tracked_vehicle_throttle,
                self.tracked_vehicle_turn,
                self.tracked_vehicle_brake > 0.5,
            );
            tracked_vehicle.controls = controls;
            tracked_vehicle.physics_update(dt, &mut self.physics_world, &terrain_getter, &surface_getter, deformable_terrain_ref);
        }

        // Обновление физики вертолета
        if let Some(ref mut heli) = self.helicopter {
            heli.physics_update(dt, &mut self.physics_world);
        }

        // Шаг физического мира
        self.physics_world.step(dt);

        // Обновление лебедки и груза
        if let Some(ref mut cargo) = self.cargo {
            self.winch.physics_update(dt, cargo, &mut self.physics_world);
        }

        Ok(())
    }

    /// Обработка кликов по кнопкам главного меню
    fn handle_menu_click(&mut self) {
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
        
        // "Новая игра" - переход к созданию персонажа
        if click_x >= center_x - button_width / 2.0 && click_x <= center_x + button_width / 2.0
            && click_y >= new_game_y && click_y <= new_game_y + button_height {
            tracing::info!("Menu click - NEW GAME - Starting character creation");
            self.game_state = GameState::CharacterCreation;
            self.character_creation.is_active = true;
            return;
        }
        
        // "Продолжить"
        if click_x >= center_x - button_width / 2.0 && click_x <= center_x + button_width / 2.0
            && click_y >= continue_y && click_y <= continue_y + button_height {
            tracing::info!("Menu click - CONTINUE");
            // Загрузка последнего сохранения
            if let Some(save_path) = self.main_menu.continue_game() {
                tracing::info!("Loading save from: {:?}", save_path);
                self.game_state = GameState::Playing;
            } else {
                tracing::warn!("No saves found");
            }
            return;
        }
        
        // "Настройки"
        if click_x >= center_x - button_width / 2.0 && click_x <= center_x + button_width / 2.0
            && click_y >= settings_y && click_y <= settings_y + button_height {
            tracing::info!("Menu click - SETTINGS (not implemented)");
            return;
        }
        
        // "Выход"
        if click_x >= center_x - button_width / 2.0 && click_x <= center_x + button_width / 2.0
            && click_y >= exit_y && click_y <= exit_y + button_height {
            tracing::info!("Menu click - EXIT");
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

        // Проверка наведения на кнопки и обновление main_menu
        if mouse_x >= center_x - button_width / 2.0 && mouse_x <= center_x + button_width / 2.0 {
            if mouse_y >= new_game_y && mouse_y <= new_game_y + button_height {
                self.main_menu.hover_button(crate::game::main_menu::MenuButton::NewGame);
            } else if mouse_y >= continue_y && mouse_y <= continue_y + button_height {
                self.main_menu.hover_button(crate::game::main_menu::MenuButton::Continue);
            } else if mouse_y >= settings_y && mouse_y <= settings_y + button_height {
                self.main_menu.hover_button(crate::game::main_menu::MenuButton::Options);
            } else if mouse_y >= exit_y && mouse_y <= exit_y + button_height {
                self.main_menu.hover_button(crate::game::main_menu::MenuButton::Exit);
            } else {
                self.main_menu.hover_button(crate::game::main_menu::MenuButton::NewGame); // Сброс
            }
        } else {
            self.main_menu.hover_button(crate::game::main_menu::MenuButton::NewGame); // Сброс
        }
    }

    /// Обработка ввода в режиме создания персонажа
    fn update_character_creation_input(&mut self) {
        use crate::game::character_creation::{CreationStep, Gender};
        use winit::keyboard::KeyCode;

        // Получаем состояние клавиш
        let input_state = self.input_manager.state();
        
        // Переход к следующему шагу по Enter или Space
        if input_state.is_key_just_pressed(KeyCode::Enter) || input_state.is_key_just_pressed(KeyCode::Space) {
            self.character_creation.next_step();
            
            // Если создание завершено - создаём игрока и начинаем игру
            if self.character_creation.current_step == CreationStep::Complete {
                self.finalize_character_creation();
            }
        }
        
        // Возврат к предыдущему шагу по Escape (кроме первого шага)
        if input_state.is_key_just_pressed(KeyCode::Escape) {
            match self.character_creation.current_step {
                CreationStep::Gender => {
                    // На первом шаге Escape возвращает в главное меню
                    self.game_state = GameState::MainMenu;
                    self.character_creation.is_active = false;
                }
                _ => {
                    self.character_creation.prev_step();
                }
            }
        }
        
        // Навигация по шагам создания персонажа
        match self.character_creation.current_step {
            CreationStep::Gender => {
                // Переключение пола стрелками влево/вправо
                if input_state.is_key_just_pressed(KeyCode::ArrowLeft) {
                    self.character_creation.data.set_gender(Gender::Male);
                }
                if input_state.is_key_just_pressed(KeyCode::ArrowRight) {
                    self.character_creation.data.set_gender(Gender::Female);
                }
            }
            CreationStep::Height => {
                // Изменение роста стрелками вверх/вниз
                if input_state.is_key_held(KeyCode::ArrowUp) {
                    self.character_creation.data.adjust_height(0.01);
                }
                if input_state.is_key_held(KeyCode::ArrowDown) {
                    self.character_creation.data.adjust_height(-0.01);
                }
            }
            CreationStep::SkinColor => {
                // Выбор цвета кожи стрелками
                if input_state.is_key_just_pressed(KeyCode::ArrowLeft) {
                    self.character_creation.data.cycle_skin_tone(-1);
                }
                if input_state.is_key_just_pressed(KeyCode::ArrowRight) {
                    self.character_creation.data.cycle_skin_tone(1);
                }
            }
            CreationStep::Face => {
                // Выбор лица стрелками
                if input_state.is_key_just_pressed(KeyCode::ArrowLeft) {
                    self.character_creation.data.cycle_face(-1);
                }
                if input_state.is_key_just_pressed(KeyCode::ArrowRight) {
                    self.character_creation.data.cycle_face(1);
                }
            }
            CreationStep::HairStyle => {
                // Выбор причёски стрелками
                if input_state.is_key_just_pressed(KeyCode::ArrowLeft) {
                    self.character_creation.data.cycle_hair_style(-1);
                }
                if input_state.is_key_just_pressed(KeyCode::ArrowRight) {
                    self.character_creation.data.cycle_hair_style(1);
                }
            }
            CreationStep::HairColor => {
                // Выбор цвета волос стрелками
                if input_state.is_key_just_pressed(KeyCode::ArrowLeft) {
                    self.character_creation.data.cycle_hair_color(-1);
                }
                if input_state.is_key_just_pressed(KeyCode::ArrowRight) {
                    self.character_creation.data.cycle_hair_color(1);
                }
            }
            CreationStep::Education | CreationStep::VehicleColor | CreationStep::StartingLocation => {
                // Упрощённая обработка для этих шагов
            }
            CreationStep::Summary => {
                // Экран подтверждения - только Enter для завершения
            }
            CreationStep::Complete => {
                // Завершено
            }
        }
    }

    /// Завершение создания персонажа и начало игры
    fn finalize_character_creation(&mut self) {
        tracing::info!("Finalizing character creation");
        
        // Создаём игрока из данных создания персонажа
        let mut player = self.character_creation.data.build_player();
        
        // Создаём физическое тело игрока (капсула)
        let start_pos = self.character_creation.data.start_location.position;
        let body = player.create_physics_body(start_pos);
        let body_idx = self.physics_world.add_body(body);
        player.body_index = Some(body_idx);
        player.position = start_pos;
        
        self.player = Some(player);
        
        // Создаём транспортные средства
        if let Err(e) = self.create_player_vehicles() {
            tracing::error!("Failed to create player vehicles: {}", e);
        }
        
        // Переходим в режим игры
        self.game_state = GameState::Playing;
        self.character_creation.is_active = false;
        
        tracing::info!("Character creation complete - starting game");
    }

    /// Создание транспортных средств игрока (обновлённая версия create_player_vehicle)
    fn create_player_vehicles(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Создаём вертолёт
        let mut heli = crate::physics::Helicopter::new(
            nalgebra::Vector3::new(0.0, 10.0, 0.0),
            0.0,
        );
        heli.body_index = Some(self.physics_world.add_body(heli.create_physics_body()));
        self.helicopter = Some(heli);

        // Создаём гусеничную технику (GTS-M)
        let tracked_chassis = crate::physics::RigidBody::new_box(
            nalgebra::Vector3::new(5.0, 2.0, 0.0),
            100.0,
            nalgebra::Vector3::new(4.0, 1.5, 2.5),
        );
        let tracked_chassis_id = self.physics_world.add_body(tracked_chassis);
        
        let mut tracked_vehicle = crate::physics::TrackedVehicle::new(
            nalgebra::Vector3::new(5.0, 2.0, 0.0),
            0.0,
        );
        tracked_vehicle.set_chassis_body_id(tracked_chassis_id);
        self.tracked_vehicle = Some(tracked_vehicle);

        // Создаём автомобиль (UAZ)
        let uaz_chassis = crate::physics::RigidBody::new_box(
            nalgebra::Vector3::new(-5.0, 2.0, 0.0),
            80.0,
            nalgebra::Vector3::new(3.5, 1.2, 1.8),
        );
        let uaz_chassis_id = self.physics_world.add_body(uaz_chassis);
        
        let mut vehicle = crate::physics::Vehicle::new(
            nalgebra::Vector3::new(-5.0, 2.0, 0.0),
            0.0,
        );
        vehicle.set_chassis_body_id(uaz_chassis_id);
        self.vehicle = Some(vehicle);

        tracing::info!("Created player vehicles: Helicopter, GTS-M (tracked), UAZ (vehicle)");
        Ok(())
    }

    /// Инициализация менеджера загрузки - регистрация всех ресурсов
    fn init_loading_manager(&mut self) {
        // Регистрация мешей
        self.loading_manager.add_resource("meshes/truck.obj", ResourceType::Mesh, 1);
        self.loading_manager.add_resource("meshes/terrain.obj", ResourceType::Mesh, 2);
        self.loading_manager.add_resource("meshes/building_low.obj", ResourceType::Mesh, 3);
        
        // Регистрация текстур
        self.loading_manager.add_resource("textures/ground.png", ResourceType::Texture, 1);
        self.loading_manager.add_resource("textures/sky.png", ResourceType::Texture, 2);
        self.loading_manager.add_resource("textures/building.png", ResourceType::Texture, 3);
        self.loading_manager.add_resource("textures/ui/hud.png", ResourceType::Texture, 4);
        
        // Регистрация шейдеров
        self.loading_manager.add_resource("shaders/basic.vert", ResourceType::Shader, 1);
        self.loading_manager.add_resource("shaders/basic.frag", ResourceType::Shader, 1);
        self.loading_manager.add_resource("shaders/terrain.vert", ResourceType::Shader, 2);
        self.loading_manager.add_resource("shaders/terrain.frag", ResourceType::Shader, 2);
        
        // Регистрация конфигов
        self.loading_manager.add_resource("config.json", ResourceType::Config, 0);
        self.loading_manager.add_resource("settings.json", ResourceType::Config, 0);
        
        // Проверка файлов
        let progress = self.loading_manager.check_all_files();
        tracing::info!(
            "Loading manager initialized: {}/{} files found",
            progress.loaded_resources + progress.failed_resources,
            progress.total_resources
        );
    }

    /// Загрузить все зарегистрированные ресурсы
    pub fn load_all_resources(&mut self) -> LoadingProgress {
        tracing::info!("Loading all resources...");
        self.loading_manager.load_all()
    }

    /// Получить текущий прогресс загрузки
    pub fn get_loading_progress(&self) -> LoadingProgress {
        self.loading_manager.get_progress()
    }

    /// Получить статистику загрузки
    pub fn get_loading_stats(&self) -> &LoadingStats {
        self.loading_manager.stats()
    }

    fn load_initial_data(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Инициализация менеджера загрузки
        self.init_loading_manager();
        
        // Загрузка начальных данных игры
        // Генерация мира и создание terrain mesh для рендерера
        if let Some(ref mut open_world) = self.open_world {
            // Генерируем первый чанк вокруг спавна
            let spawn_chunk_id = crate::world::ChunkId::new(0, 0);
            let chunk_data = open_world.generator.generate_chunk(spawn_chunk_id);
            
            // Создаём меш территории из данных чанка
            let (vertices, indices) = crate::world::chunk::generate_chunk_mesh(&chunk_data, 0);
            
            // Конвертируем TerrainVertex в формат для Mesh (flat array)
            let mut vertex_data: Vec<f32> = Vec::with_capacity(vertices.len() * 18);
            for v in &vertices {
                vertex_data.extend_from_slice(&v.position);
                vertex_data.extend_from_slice(&v.normal);
                vertex_data.extend_from_slice(&v.tangent);
                vertex_data.extend_from_slice(&v.bitangent);
                vertex_data.extend_from_slice(&v.texcoord);
                vertex_data.extend_from_slice(&v.splat_weights);
            }
            
            // Создаём меш и передаём в рендерер
            if let Some(ref mut renderer) = self.renderer {
                if let Ok(gl) = &self.graphics_context.gl {
                    match crate::graphics::mesh::Mesh::new_terrain(gl, &vertex_data, &indices) {
                        Ok(terrain_mesh) => {
                            renderer.set_terrain_mesh(terrain_mesh);
                            tracing::info!("Terrain mesh created with {} vertices", vertices.len());
                        }
                        Err(e) => {
                            tracing::error!("Failed to create terrain mesh: {}", e);
                        }
                    }
                }
            }
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
        // Создаем начальные транспортные средства для игрока - все три типа для тестирования
        let spawn_pos = nalgebra::Vector3::new(0.0, 100.0, 0.0);

        // 1. Создаем вертолет
        let mut helicopter = crate::physics::Helicopter::new(spawn_pos);
        
        // Добавляем тело вертолета в физический мир
        let chassis_body = crate::physics::RigidBody::new_capsule(
            spawn_pos,
            1100.0, // масса вертолета
            1.5,    // радиус
            4.0,    // высота
        );
        let chassis_id = self.physics_world.add_body(chassis_body);
        helicopter.set_chassis_body_id(chassis_id);

        self.helicopter = Some(helicopter);

        // 2. Создаем гусеничное транспортное средство ГТ-СМ
        let tracked_spawn_pos = nalgebra::Vector3::new(10.0, 2.0, 0.0);
        let mut tracked_vehicle = crate::physics::TrackedVehicle::new(
            crate::physics::TrackedVehicleType::GTS_M,
            tracked_spawn_pos
        );
        
        // Добавляем тело гусеничного транспорта в физический мир
        let tracked_chassis_body = crate::physics::RigidBody::new_box(
            tracked_spawn_pos,
            4500.0, // масса ГТ-СМ
            nalgebra::Vector3::new(2.0, 1.5, 4.0), // размеры
        );
        let tracked_chassis_id = self.physics_world.add_body(tracked_chassis_body);
        tracked_vehicle.set_chassis_body_id(tracked_chassis_id);
        
        // Сохраняем ссылку на гусеничный транспорт в Engine
        self.tracked_vehicle = Some(tracked_vehicle);

        // 3. Создаем автомобиль UAZ
        let vehicle_spawn_pos = nalgebra::Vector3::new(-10.0, 2.0, 0.0);
        let mut vehicle = crate::physics::Vehicle::new(vehicle_spawn_pos);
        
        // Добавляем тело автомобиля в физический мир
        let vehicle_chassis_body = crate::physics::RigidBody::new_box(
            vehicle_spawn_pos,
            2000.0, // масса UAZ
            nalgebra::Vector3::new(2.0, 1.8, 4.5), // размеры
        );
        let vehicle_chassis_id = self.physics_world.add_body(vehicle_chassis_body);
        vehicle.set_chassis_body_id(vehicle_chassis_id);
        
        self.vehicle = Some(vehicle);
        self.vehicle_chassis_id = Some(vehicle_chassis_id);

        // Добавляем модель UAZ Patriot из GLB файла для рендеринга
        if let Some(ref mut renderer) = self.renderer {
            // Загружаем модель UAZ Patriot из assets/models/uaz_patriot.glb
            match crate::assets::VehicleLoader::load_gltf("assets/models/uaz_patriot.glb") {
                Ok(model) => {
                    renderer.load_model("uaz_patriot".to_string(), model);
                    
                    // Ставим машину на сцену
                    renderer.set_vehicle_transform(
                        nalgebra::Vector3::new(0.0, 2.0, -15.0),
                        nalgebra::UnitQuaternion::from_axis_angle(&nalgebra::Vector3::y_axis(), 0.0)
                    );
                    tracing::info!("Loaded UAZ Patriot from GLB");
                }
                Err(e) => {
                    tracing::warn!("Failed to load UAZ Patriot GLB: {}, using fallback box", e);
                    // Fallback: создаём простую коробку если загрузка не удалась
                    let _ = renderer.create_vehicle_box_mesh(nalgebra::Vector3::new(2.5, 1.8, 5.5));
                    renderer.set_vehicle_transform(
                        nalgebra::Vector3::new(0.0, 2.0, -10.0),
                        nalgebra::UnitQuaternion::from_axis_angle(&nalgebra::Vector3::y_axis(), 0.0)
                    );
                }
            }

            // Включаем HUD
            renderer.set_hud_data(crate::ui::hud::VehicleHudData {
                speed_kmh: 45.0,
                engine_rpm: 2200.0,
                ..Default::default()
            });
        }

        tracing::info!("Created player vehicles: Helicopter, GTS-M (tracked), UAZ (vehicle)");

        Ok(())
    }

    fn save_game(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Проблема 9: Использование SaveSystem для сохранения
        use crate::game::save::{SaveData, SaveMetadata, WorldStateData, PlayerData};
        use std::time::{SystemTime, UNIX_EPOCH};
        
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        // Получаем позицию игрока
        let position = if let Some(ref heli) = self.helicopter {
            [heli.position.x, heli.position.y, heli.position.z]
        } else {
            [0.0, 0.0, 0.0]
        };
        
        // Создаем метаданные сохранения
        let metadata = SaveMetadata {
            slot: 0,
            player_name: "Player".to_string(),
            game_time_hours: self.day_night_cycle.get_hour(),
            timestamp,
            location_name: "Open World".to_string(),
            position,
            money_rub: 50000.0, // TODO: get from player
            playtime_hours: 0.0, // TODO: track playtime
        };
        
        // Создаем данные игрока
        let player_data = PlayerData {
            name: "Player".to_string(),
            is_male: true,
            height: 1.93,
            skin_color: [0.8, 0.65, 0.55],
            face_variant: 0,
            hair_style: 0,
            hair_color: [0.25, 0.18, 0.12],
            skills: crate::game::save::PlayerSkillsData {
                mechanics: crate::game::save::SkillData { rank: 1, mastery: 0.0, total_hours: 0.0 },
                electrics: crate::game::save::SkillData { rank: 1, mastery: 0.0, total_hours: 0.0 },
                welding: crate::game::save::SkillData { rank: 1, mastery: 0.0, total_hours: 0.0 },
                construction: crate::game::save::SkillData { rank: 1, mastery: 0.0, total_hours: 0.0 },
                road_building: crate::game::save::SkillData { rank: 1, mastery: 0.0, total_hours: 0.0 },
                driving: crate::game::save::SkillData { rank: 1, mastery: 0.0, total_hours: 0.0 },
                tracked: crate::game::save::SkillData { rank: 1, mastery: 0.0, total_hours: 0.0 },
                piloting: crate::game::save::SkillData { rank: 1, mastery: 0.0, total_hours: 0.0 },
                flying: crate::game::save::SkillData { rank: 1, mastery: 0.0, total_hours: 0.0 },
                crane: crate::game::save::SkillData { rank: 1, mastery: 0.0, total_hours: 0.0 },
                geology: crate::game::save::SkillData { rank: 1, mastery: 0.0, total_hours: 0.0 },
                drilling: crate::game::save::SkillData { rank: 1, mastery: 0.0, total_hours: 0.0 },
                logging: crate::game::save::SkillData { rank: 1, mastery: 0.0, total_hours: 0.0 },
                mining: crate::game::save::SkillData { rank: 1, mastery: 0.0, total_hours: 0.0 },
                business: crate::game::save::SkillData { rank: 1, mastery: 0.0, total_hours: 0.0 },
                logistics: crate::game::save::SkillData { rank: 1, mastery: 0.0, total_hours: 0.0 },
                trading: crate::game::save::SkillData { rank: 1, mastery: 0.0, total_hours: 0.0 },
                navigation: crate::game::save::SkillData { rank: 1, mastery: 0.0, total_hours: 0.0 },
                medicine: crate::game::save::SkillData { rank: 1, mastery: 0.0, total_hours: 0.0 },
                fitness: crate::game::save::SkillData { rank: 1, mastery: 0.0, total_hours: 0.0 },
            },
            money: crate::game::save::PlayerMoneyData {
                rub: 50000.0,
                cny: 0.0,
                usd: 0.0,
            },
            inventory: vec![],
            inventory_weight: 0.0,
            position,
            rotation: [0.0, 0.0, 0.0, 1.0],
            state: crate::game::save::PlayerStateData::OnFoot,
            camera_mode: crate::game::save::CameraModeData::ThirdPerson { distance: 4.0, yaw: 0.0, pitch: 0.3 },
            stamina: 1.0,
        };
        
        // Создаем данные мира
        let world_state = WorldStateData {
            time_hours: self.day_night_cycle.get_hour(),
            day: 1,
            weather: self.weather_system.get_state().description().to_string(),
            weather_intensity: self.weather_system.get_state().intensity,
            discovered_locations: vec![],
            completed_missions: vec![],
            active_missions: vec![],
            reputation: std::collections::HashMap::new(),
        };
        
        // Создаем данные сохранения
        let save_data = SaveData {
            metadata,
            player: player_data,
            world_state,
            vehicles: vec![],
        };
        
        // Сохраняем через SaveSystem
        self.save_system.save_game(0, &save_data)
            .map_err(|e| format!("Failed to save game: {}", e))?;
        
        tracing::info!("Game saved successfully");
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
                            // Настраиваем размеры и состояние меню
                            let mut r = renderer;
                            r.width = 1280;
                            r.height = 720;
                            r.menu_state = MenuState::MainMenu;
                            
                            // Добавляем LOD объекты для тестирования (лес, дороги, здания)
                            use crate::graphics::lod_system::{LodObject, LodModel};
                            
                            // Тестовый объект - машина игрока
                            let lod_obj = LodObject::new(
                                nalgebra::Vector3::new(0.0, 2.0, -10.0),
                                5.0, // radius
                            );
                            r.lod_manager.add_object(lod_obj);
                            
                            // Лес вдалеке
                            for i in 0..5 {
                                let lod_obj = LodObject::new(
                                    nalgebra::Vector3::new(-50.0 + i as f32 * 20.0, 0.0, -50.0),
                                    10.0,
                                );
                                r.lod_manager.add_object(lod_obj);
                            }
                            
                            // Дорога/здания
                            for i in 0..3 {
                                let lod_obj = LodObject::new(
                                    nalgebra::Vector3::new(50.0, 0.0, -30.0 + i as f32 * 30.0),
                                    15.0,
                                );
                                r.lod_manager.add_object(lod_obj);
                            }
                            
                            self.renderer = Some(r);
                            tracing::info!("Renderer initialized successfully");
                        }
                        Err(e) => {
                            tracing::error!("Failed to initialize Renderer: {}", e);
                        }
                    }
                }
                
                self.graphics_context = ctx;

                // Загрузка начальных данных
                if let Err(e) = self.load_initial_data() {
                    tracing::error!("Failed to load initial data: {}", e);
                }
            }
            Err(e) => {
                tracing::error!("Failed to initialize window: {}", e);
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
                    tracing::error!("Resize error: {}", e);
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
            tracing::error!("Update error: {}", e);
        }

        // Пропускаем рендеринг если контекст ещё не инициализирован
        if self.graphics_context.is_initialized() {
            if let Err(e) = self.render() {
                tracing::error!("Render error: {}", e);
            }
        }
    }
}
