//! Utils module for RTGC-0.7

pub mod math;
pub mod time;
pub mod logger;

pub use math::*;
pub use time::{TimeManager, FpsCounter};
pub use logger::{init_logger, init_logger_with_level};
