//! Open World Scene

use super::super::scene::{Scene, SceneType};
use std::any::Any;

pub struct OpenWorldScene {
    name: String,
    world_loaded: bool,
}

impl OpenWorldScene {
    pub fn new() -> Self {
        Self {
            name: "Open World".to_string(),
            world_loaded: false,
        }
    }

    pub fn load_world(&mut self) {
        self.world_loaded = true;
        tracing::info!("Open world loaded");
    }
}

impl Default for OpenWorldScene {
    fn default() -> Self {
        Self::new()
    }
}

impl Scene for OpenWorldScene {
    fn scene_type(&self) -> SceneType {
        SceneType::OpenWorld
    }

    fn on_enter(&mut self) {
        tracing::info!("Entering Open World");
        self.load_world();
    }

    fn on_exit(&mut self) {
        tracing::info!("Exiting Open World");
    }

    fn update(&mut self, _delta_time: f32) {
        // Update world entities, physics, etc.
    }

    fn render(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Render the open world
        Ok(())
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
