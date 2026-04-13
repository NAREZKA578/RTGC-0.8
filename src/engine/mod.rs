//! Модуль подсистем движка
//! 
//! Предоставляет инкапсулированные подсистемы для разделения ответственности
//! 
//! # Архитектура
//! 
//! Движок разделён на следующие подмодули:
//! 
//! - [`state`] - Управление состоянием приложения (единый источник истины)
//! - [`subsystems`] - Контейнеры для подсистем (графика, физика, UI, мир)
//! - [`physics_manager`] - Инкапсуляция физической симуляции
//! - [`world_manager`] - Управление открытым миром, погодой, миссиями
//! - [`vehicle_manager`] - Управление транспортными средствами

pub mod state;
pub mod subsystems;
pub mod physics_manager;
pub mod world_manager;
pub mod vehicle_manager;

pub use state::*;
pub use subsystems::*;
pub use physics_manager::*;
pub use world_manager::*;
pub use vehicle_manager::*;
