//! Utils module for RTGC-0.7

pub mod math;
pub mod time;
pub mod logger;
pub mod random;
pub mod temp;

pub use math::*;
pub use time::{TimeManager, FpsCounter};
pub use logger::{init_logger, init_logger_with_level};
pub use random::Rng;
pub use temp::{temp_dir, create_temp_file, create_temp_dir};
