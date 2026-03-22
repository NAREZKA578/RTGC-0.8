//! Game Module for RTGC-0.7
//! Contains gameplay systems: missions, cargo, weather, day/night cycle

pub mod mission_save;
pub mod weather;
pub mod cargo;
pub mod winch;
pub mod mission_generator;

pub use mission_save::{SaveGame, MissionSaveManager};
pub use weather::{WeatherSystem, WeatherState, DayNightCycle, PrecipitationType};
pub use cargo::Cargo;
pub use winch::Winch;
pub use mission_generator::{MissionGenerator, Mission, CargoType};
