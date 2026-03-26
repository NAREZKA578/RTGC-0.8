use winit::{
    event::{WindowEvent, ElementState, KeyboardInput, MouseButton},
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
use crate::graphics::renderer::MenuState;
use crate::game::{WeatherSystem, Cargo, Winch, MissionGenerator, Mission};
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
        let material_manager = MaterialManager::new();
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
        profiler::begin_frame();
        let _profile = profiler::ProfileScope::new("Engine::update");
        
        // Обновление ввода
        self.input_manager.update();
        
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
            heli.update(dt, &self.input_manager);
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
                    open_world.get_height(x, z)
                } else {
                    0.0
                }
            };
            let surface_getter = |_x: f32, _z: f32| -> crate::physics::SurfaceType {
                crate::physics::SurfaceType::Default
            };
            vehicle.update(dt, terrain_getter, surface_getter);
        }
        
        // Обновление лебедки
        if let Some(ref mut cargo) = self.cargo {
            self.winch.update(dt, cargo);
        }
        
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
        
        // Подготовка кадра
        self.graphics_context.begin_frame()?;
        
        // Получение матриц вида и проекции
        let view_matrix = if let Some(ref heli) = self.helicopter {
            heli.get_view_matrix()
        } else if let Some(ref vehicle) = self.vehicle {
            vehicle.get_view_matrix()
        } else {
            Matrix4::identity()
        };
        
        let proj_matrix = self.graphics_context.get_projection_matrix(std::f32::consts::PI / 4.0, 0.1, 1000.0);
        let view_proj = proj_matrix * view_matrix;
        
        // Рендеринг частиц
        let gl = &self.graphics_context.gl;
        self.particle_system.render(gl, view_proj);
        
        // Отладочный рендеринг
        if self.debug_mode {
            self.debug_renderer.flush_to_gl(gl, view_proj);
        }
        
        // Завершение кадра
        self.graphics_context.end_frame()?;
        
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
                        match key_code {
                            // Настройки: Открытие/закрытие настроек на Escape (если открыты настройки или инвентарь)
                            KeyCode::Escape => {
                                // Если открыты настройки или инвентарь - закрываем их
                                if self.hud_manager.is_settings_open() || self.hud_manager.is_inventory_open() {
                                    self.hud_manager.set_settings_open(false);
                                    self.hud_manager.set_inventory_open(false);
                                } else {
                                    // Иначе выход из игры
                                    return Err("Escape pressed".into());
                                }
                            }
                            KeyCode::KeyF3 => {
                                self.debug_mode = !self.debug_mode;
                            }
                            // Ф1.6: Открытие инвентаря на Tab
                            KeyCode::Tab => {
                                // Не открывать инвентарь если открыты настройки
                                if !self.hud_manager.is_settings_open() {
                                    self.hud_manager.toggle_inventory();
                                }
                            }
                            // Настройки: Открытие настроек на F1
                            KeyCode::F1 => {
                                // Не открывать настройки если открыт инвентарь
                                if !self.hud_manager.is_inventory_open() {
                                    self.hud_manager.toggle_settings();
                                }
                            }
                            KeyCode::KeyW => self.input_manager.set_key_state(KeyCode::KeyW, true),
                            KeyCode::KeyS => self.input_manager.set_key_state(KeyCode::KeyS, true),
                            KeyCode::KeyA => self.input_manager.set_key_state(KeyCode::KeyA, true),
                            KeyCode::KeyD => self.input_manager.set_key_state(KeyCode::KeyD, true),
                            KeyCode::Space => self.input_manager.set_key_state(KeyCode::Space, true),
                            KeyCode::ShiftLeft => self.input_manager.set_key_state(KeyCode::ShiftLeft, true),
                            _ => {}
                        }
                    }
                } else if event.state == ElementState::Released {
                    if let PhysicalKey::Code(key_code) = event.physical_key {
                        match key_code {
                            KeyCode::KeyW => self.input_manager.set_key_state(KeyCode::KeyW, false),
                            KeyCode::KeyS => self.input_manager.set_key_state(KeyCode::KeyS, false),
                            KeyCode::KeyA => self.input_manager.set_key_state(KeyCode::KeyA, false),
                            KeyCode::KeyD => self.input_manager.set_key_state(KeyCode::KeyD, false),
                            KeyCode::Space => self.input_manager.set_key_state(KeyCode::Space, false),
                            KeyCode::ShiftLeft => self.input_manager.set_key_state(KeyCode::ShiftLeft, false),
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
                if *button == MouseButton::Left {
                    self.input_manager.set_mouse_button_state(0, *state == ElementState::Pressed);
                } else if *button == MouseButton::Right {
                    self.input_manager.set_mouse_button_state(1, *state == ElementState::Pressed);
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn physics_step(&mut self, dt: f32) -> Result<(), Box<dyn std::error::Error>> {
        // Обновление физики транспорта
        if let Some(ref mut vehicle) = self.vehicle {
            vehicle.physics_update(dt, &mut self.physics_world);
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

    fn load_initial_data(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Загрузка начальных данных игры
        // Генерация мира
        if let Some(ref mut open_world) = self.open_world {
            open_world.generate_terrain();
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
        if let Some(ref road_network) = self.road_network {
            let settlements_clone = self.settlements.clone();
            self.mission_generator = Some(MissionGenerator::new(settlements_clone, road_network.clone(), self.world_seed));
        }

        // Создание начальной миссии
        if let Some(ref mut generator) = self.mission_generator {
            self.current_mission = generator.generate_mission(&self.settlements);
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
                self.graphics_context = ctx;
                
                // Инициализация мира после создания контекста
                if let Some(ref mut open_world) = self.open_world {
                    open_world.generate_terrain();
                }
                
                // Загрузка начальных данных
                if let Err(e) = self.load_initial_data() {
                    eprintln!("Failed to load initial data: {}", e);
                }
            }
            Err(e) => {
                eprintln!("Failed to initialize window: {}", e);
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WinitWindowEvent) {
        match event {
            WinitWindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WinitWindowEvent::KeyboardInput { event, .. } => {
                if event.state == ElementState::Pressed {
                    if let PhysicalKey::Code(key_code) = event.physical_key {
                        match key_code {
                            KeyCode::Escape => {
                                event_loop.exit();
                            }
                            KeyCode::KeyF3 => {
                                self.debug_mode = !self.debug_mode;
                            }
                            KeyCode::KeyW => self.input_manager.set_key_state(KeyCode::KeyW, true),
                            KeyCode::KeyS => self.input_manager.set_key_state(KeyCode::KeyS, true),
                            KeyCode::KeyA => self.input_manager.set_key_state(KeyCode::KeyA, true),
                            KeyCode::KeyD => self.input_manager.set_key_state(KeyCode::KeyD, true),
                            KeyCode::Space => self.input_manager.set_key_state(KeyCode::Space, true),
                            KeyCode::ShiftLeft => self.input_manager.set_key_state(KeyCode::ShiftLeft, true),
                            _ => {}
                        }
                    }
                } else if event.state == ElementState::Released {
                    if let PhysicalKey::Code(key_code) = event.physical_key {
                        match key_code {
                            KeyCode::KeyW => self.input_manager.set_key_state(KeyCode::KeyW, false),
                            KeyCode::KeyS => self.input_manager.set_key_state(KeyCode::KeyS, false),
                            KeyCode::KeyA => self.input_manager.set_key_state(KeyCode::KeyA, false),
                            KeyCode::KeyD => self.input_manager.set_key_state(KeyCode::KeyD, false),
                            KeyCode::Space => self.input_manager.set_key_state(KeyCode::Space, false),
                            KeyCode::ShiftLeft => self.input_manager.set_key_state(KeyCode::ShiftLeft, false),
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
                if button == MouseButton::Left {
                    self.input_manager.set_mouse_button_state(0, state == ElementState::Pressed);
                } else if button == MouseButton::Right {
                    self.input_manager.set_mouse_button_state(1, state == ElementState::Pressed);
                }
            }
            WinitWindowEvent::Resized(new_size) => {
                if let Err(e) = self.graphics_context.resize(new_size.width, new_size.height) {
                    eprintln!("Resize error: {}", e);
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
            eprintln!("Update error: {}", e);
        }
        
        if let Err(e) = self.render() {
            eprintln!("Render error: {}", e);
        }
    }
}
