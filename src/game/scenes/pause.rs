//! Pause Scene

use super::super::scene::{Scene, SceneType};
use std::any::Any;

pub struct PauseScene {
    name: String,
}

impl PauseScene {
    pub fn new() -> Self {
        Self {
            name: "Pause Menu".to_string(),
        }
    }
}

impl Default for PauseScene {
    fn default() -> Self {
        Self::new()
    }
}

impl Scene for PauseScene {
    fn scene_type(&self) -> SceneType {
        SceneType::Pause
    }

    fn on_enter(&mut self) {
        tracing::info!("Entering Pause Menu");
    }

    fn on_exit(&mut self) {
        tracing::info!("Exiting Pause Menu");
    }

    fn update(&mut self, _delta_time: f32) {
        // Handle pause menu input
    }

    fn render(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Render pause menu overlay
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
