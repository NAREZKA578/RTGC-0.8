//! Core - Центральный управляющий модуль приложения
//! 
//! Этот файл является точкой входа и координации всех подсистем движка.
//! Он импортирует Engine из модуля engine и предоставляет удобный интерфейс
//! для запуска и управления приложением.

// Импорт основного движка
pub use crate::engine::Engine;

// Ре-экспорт ключевых типов для удобного доступа
pub use crate::engine::EngineState;
pub use crate::engine::PauseReason;
pub use crate::engine::MenuState;

// Ре-экспорт менеджеров
pub use crate::engine::PhysicsManager;
pub use crate::engine::WorldManager;
pub use crate::engine::VehicleManager;
pub use crate::engine::InputManagerWrapper;
pub use crate::engine::RenderManager;
pub use crate::engine::GameLoopManager;

// Ре-экспорт подсистем
pub use crate::engine::EngineSubsystems;
pub use crate::engine::GraphicsSubsystem;
pub use crate::engine::PhysicsSubsystem;
pub use crate::engine::UISubsystem;
pub use crate::engine::WorldSubsystem;

/// Тип результата для операций ядра
pub type CoreResult<T> = Result<T, Box<dyn std::error::Error>>;

/// Центральная функция для создания и запуска движка
/// 
/// # Пример использования
/// ```rust
/// fn main() -> CoreResult<()> {
///     tracing_subscriber::fmt::init();
///     core::run()?;
///     Ok(())
/// }
/// ```
pub fn run() -> CoreResult<()> {
    // Инициализация логгера
    tracing_subscriber::fmt::init();
    
    // Создание движка
    let mut engine = Engine::new()?;
    
    // Запуск игрового цикла
    engine.run()?;
    
    Ok(())
}

/// Создание нового экземпляра движка
/// 
/// # Возвращает
/// * `CoreResult<Engine>` - Успешно созданный движок или ошибка
pub fn create_engine() -> CoreResult<Engine> {
    Engine::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_core_exports() {
        // Проверяем, что все ключевые типы доступны
        let _ = std::any::type_name::<Engine>();
        let _ = std::any::type_name::<EngineState>();
        let _ = std::any::type_name::<PhysicsManager>();
        let _ = std::any::type_name::<WorldManager>();
        let _ = std::any::type_name::<VehicleManager>();
    }
}
