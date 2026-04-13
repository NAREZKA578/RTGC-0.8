//! Подсистемы движка - инкапсулированные модули для разделения ответственности
//! 
//! Этот модуль предоставляет структуру для хранения всех подсистем движка,
//! обеспечивая слабую связанность между компонентами.

use crate::audio::AudioSystem;
use crate::ecs::EcsManager;
use crate::game::asset_manager::AssetManager;
use crate::game::debug_menu::DebugMenu;
use crate::game::interaction::InteractionSystem;
use crate::game::loading_manager::LoadingManager;
use crate::game::save::SaveSystem;
use crate::game::ui::UIManager;
use crate::graphics::debug_renderer::DebugRenderer;
use crate::graphics::material::MaterialManager;
use crate::graphics::renderer::Renderer;
use crate::graphics::particles::ParticleSystem;
use crate::input::InputManager;
use crate::physics;
use crate::ui::HudManager;
use crate::world::DayNightCycle;

/// Контейнер для всех подсистем движка
/// 
/// Эта структура инкапсулирует все подсистемы, предоставляя контролируемый доступ
/// к ним через методы-геттеры. Это уменьшает связанность и упрощает тестирование.
pub struct EngineSubsystems {
    /// Графическая подсистема (рендеринг, материалы, частицы)
    pub graphics: GraphicsSubsystem,
    
    /// Физическая подсистема
    pub physics: PhysicsSubsystem,
    
    /// Подсистема ввода
    pub input: InputManager,
    
    /// Аудио подсистема
    pub audio: AudioSystem,
    
    /// ECS менеджер
    pub ecs: EcsManager,
    
    /// UI подсистема
    pub ui: UISubsystem,
    
    /// Подсистема игрового мира
    pub world: WorldSubsystem,
    
    /// Подсистема загрузки ресурсов
    pub loading: LoadingManager,
    
    /// Подсистема сохранения
    pub save: SaveSystem,
}

impl EngineSubsystems {
    /// Создаёт новый контейнер подсистем
    pub fn new(
        graphics: GraphicsSubsystem,
        physics: PhysicsSubsystem,
        input: InputManager,
        audio: AudioSystem,
        ecs: EcsManager,
        ui: UISubsystem,
        world: WorldSubsystem,
        loading: LoadingManager,
        save: SaveSystem,
    ) -> Self {
        Self {
            graphics,
            physics,
            input,
            audio,
            ecs,
            ui,
            world,
            loading,
            save,
        }
    }
    
    /// Обновляет все подсистемы
    pub fn update(&mut self, dt: f32) {
        self.graphics.update(dt);
        self.physics.update(dt);
        self.ui.update(dt);
        self.world.update(dt);
        // input, audio, ecs, loading, save обновляются по мере необходимости
    }
}

/// Графическая подсистема
pub struct GraphicsSubsystem {
    pub renderer: Option<Renderer>,
    pub material_manager: MaterialManager,
    pub particle_system: ParticleSystem,
    pub debug_renderer: DebugRenderer,
}

impl GraphicsSubsystem {
    pub fn new(
        renderer: Option<Renderer>,
        material_manager: MaterialManager,
        particle_system: ParticleSystem,
        debug_renderer: DebugRenderer,
    ) -> Self {
        Self {
            renderer,
            material_manager,
            particle_system,
            debug_renderer,
        }
    }
    
    pub fn update(&mut self, dt: f32) {
        self.particle_system.update(dt);
    }
    
    pub fn render(&mut self) -> Result<(), crate::error::EngineError> {
        if let Some(ref mut renderer) = self.renderer {
            renderer.render()?;
        }
        Ok(())
    }
}

/// Физическая подсистема
pub struct PhysicsSubsystem {
    pub physics_world: physics::PhysicsWorld,
}

impl PhysicsSubsystem {
    pub fn new(physics_world: physics::PhysicsWorld) -> Self {
        Self { physics_world }
    }
    
    pub fn update(&mut self, dt: f32) {
        // Базовое обновление физического мира
        // Детальная логика обновляется в специализированных методах
    }
    
    pub fn step_simulation(&mut self, dt: f32) {
        self.physics_world.step(dt);
    }
}

/// UI подсистема
pub struct UISubsystem {
    pub hud_manager: HudManager,
    pub ui_manager: UIManager,
    pub debug_menu: DebugMenu,
}

impl UISubsystem {
    pub fn new(
        hud_manager: HudManager,
        ui_manager: UIManager,
        debug_menu: DebugMenu,
    ) -> Self {
        Self {
            hud_manager,
            ui_manager,
            debug_menu,
        }
    }
    
    pub fn update(&mut self, dt: f32) {
        self.hud_manager.update(dt);
        self.ui_manager.update(dt);
    }
}

/// Подсистема игрового мира
pub struct WorldSubsystem {
    pub day_night_cycle: DayNightCycle,
}

impl WorldSubsystem {
    pub fn new(day_night_cycle: DayNightCycle) -> Self {
        Self { day_night_cycle }
    }
    
    pub fn update(&mut self, dt: f32) {
        self.day_night_cycle.update(dt);
    }
}
