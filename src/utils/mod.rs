//! Utils module for RTGC-0.8

pub mod math;
pub mod time;
pub mod logger;
pub mod random;
pub mod console;
pub mod hot_reload;

pub use math::*;
pub use time::{TimeManager, FpsCounter};
pub use logger::{init_logger, init_logger_with_level};
pub use random::Rng;
pub use console::{Console, ConsoleKey};
pub use hot_reload::{HotReloadManager, HotReloadConfig};
