use winit::{
    event::{WindowEvent, ElementState, KeyEvent, MouseButton},
    keyboard::{KeyCode, PhysicalKey},
};
use std::sync::Arc;
use crate::graphics::GlContext;
use crate::graphics::material::MaterialManager;
use crate::graphics::mesh::Mesh;
use crate::input::InputManager;
use crate::audio::AudioSystem;
use crate::ecs::EcsManager;
use crate::physics;
use crate::graphics::renderer::MenuState;
use crate::game::{WeatherSystem, DayNightCycle, Cargo, Winch, MissionGenerator, Mission};
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
    pub fn new(event_loop: &winit::event_loop::EventLoop<()>) -> Result<Self, Box<dyn std::error::Error>> {
        let graphics_context = GlContext::new(event_loop)?;
        let input_manager = InputManager::new();
        let audio_system = AudioSystem::new()?;
        let ecs_manager = EcsManager::new();
        let physics_world = physics::PhysicsWorld::new();
        let hud_manager = HudManager::new();
        let material_manager = MaterialManager::new();
        let particle_system = ParticleSystem::new();
        let debug_renderer = DebugRenderer::new();
        let weather_system = WeatherSystem::new();
        let day_night_cycle = DayNightCycle::new();
        let winch = Winch::new();

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
        use winit::event::{Event, StartCause};
        use winit::event_loop::ControlFlow;
        
        let event_loop = winit::event_loop::EventLoop::new();
        
        // Инициализация мира
        self.open_world = Some(OpenWorld::new(self.world_seed));
        self.mission_generator = Some(MissionGenerator::new(self.world_seed));
        
        // Загрузка начальных данных
        self.load_initial_data()?;
        
        event_loop.run(|event, _, control_flow| {
            *control_flow = ControlFlow::Poll;
            
            match event {
                Event::NewEvents(StartCause::Init) => {
                    self.last_frame_time = std::time::Instant::now();
                }
                Event::WindowEvent { event, .. } => {
                    if let Err(e) = self.handle_event(&event) {
                        eprintln!("Event handling error: {}", e);
                        *control_flow = ControlFlow::Exit;
                    }
                }
                Event::MainEventsCleared => {
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
                _ => {}
            }
        });
        
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
        self.weather_system.update(dt);
        self.day_night_cycle.update(dt);
        self.particle_system.update(dt);
        self.hud_manager.update(dt);
        
        // Обновление вертолета
        if let Some(ref mut heli) = self.helicopter {
            heli.update(dt, &self.input_manager);
        }
        
        // Обновление транспортного средства
        if let Some(ref mut vehicle) = self.vehicle {
            vehicle.update(dt, self.vehicle_throttle, self.vehicle_steering, self.vehicle_brake);
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
        
        let proj_matrix = self.graphics_context.get_projection_matrix();
        let view_proj = proj_matrix * view_matrix;
        
        // Рендеринг мира
        if let Some(ref open_world) = self.open_world {
            self.graphics_context.render_terrain(open_world, &view_proj)?;
        }
        
        // Рендеринг транспортных средств
        if let Some(ref vehicle) = self.vehicle {
            self.graphics_context.render_vehicle(vehicle, &view_proj)?;
        }
        
        // Рендеринг вертолета
        if let Some(ref heli) = self.helicopter {
            self.graphics_context.render_helicopter(heli, &view_proj)?;
        }
        
        // Рендеринг частиц
        self.particle_system.render(&self.graphics_context, &view_proj)?;
        
        // Отладочный рендеринг
        if self.debug_mode {
            self.debug_renderer.render(&self.graphics_context, &view_proj)?;
        }
        
        // Рендеринг UI
        self.hud_manager.render(&mut self.graphics_context, self.vehicle_health, self.vehicle_fuel)?;
        
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
                            KeyCode::Escape => {
                                return Err("Escape pressed".into());
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
            self.settlements = Settlement::generate_settlements(open_world, 5);
        }

        // Создание дорожной сети
        if !self.settlements.is_empty() {
            self.road_network = Some(RoadNetwork::generate(&self.settlements));
        }

        // Создание начальной миссии
        if let Some(ref mut generator) = self.mission_generator {
            self.current_mission = generator.generate_mission(&self.settlements);
        }

        // Создание игрока (вертолет или транспортное средство)
        self.create_player_vehicle()?;

        Ok(())
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
