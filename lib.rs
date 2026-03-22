pub mod assets;
pub mod audio;
pub mod config;
pub mod ecs;
pub use ecs::job_system::*; // Re-export job_system types
pub mod error;
pub mod graphics;
pub mod input;
pub mod physics;
pub mod ui;
pub mod profiler;
pub mod engine;
pub mod world;
pub mod game;
pub mod network;

// Re-export only specific types to avoid glob conflicts
// Each module's public API should be accessed via module::Type syntax
// pub use assets::*;  // Use assets::Type instead
// pub use config::*;  // Use config::Type instead
// pub use ecs::*;     // Use ecs::Type instead (except job_system which is re-exported above)
// pub use error::*;   // Use error::Type instead
// pub use physics::*; // Use physics::Type instead
// pub use graphics::*;// Use graphics::Type instead
// pub use audio::*;   // Use audio::Type instead
// pub use ui::*;      // Use ui::Type instead
// pub use profiler::*;// Use profiler::Type instead
// pub use engine::*;  // Use engine::Type instead
// pub use world::*;   // Use world::Type instead
// pub use game::*;    // Use game::Type instead
// pub use network::*; // Use network::Type instead

// Core engine types re-export
pub use nalgebra;
pub use winit;