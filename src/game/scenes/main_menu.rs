//! Main Menu Scene

use super::super::scene::{Scene, SceneType, SceneId};
use std::any::Any;

pub struct MainMenuScene {
    name: String,
}

impl MainMenuScene {
    pub fn new() -> Self {
        Self {
            name: "Main Menu".to_string(),
        }
    }
}

impl Default for MainMenuScene {
    fn default() -> Self {
        Self::new()
    }
}

impl Scene for MainMenuScene {
    fn scene_type(&self) -> SceneType {
        SceneType::MainMenu
    }

    fn on_enter(&mut self) {
        tracing::info!("Entering Main Menu");
    }

    fn on_exit(&mut self) {
        tracing::info!("Exiting Main Menu");
    }

    fn update(&mut self, _delta_time: f32) {
        // Handle menu input and animations
    }

    fn render(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Render main menu UI
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
