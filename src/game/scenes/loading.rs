//! Loading Scene

use super::super::scene::{Scene, SceneType, TransitionEffect};
use std::any::Any;

pub struct LoadingScene {
    name: String,
    progress: f32,
    loading_complete: bool,
}

impl LoadingScene {
    pub fn new() -> Self {
        Self {
            name: "Loading".to_string(),
            progress: 0.0,
            loading_complete: false,
        }
    }

    pub fn set_progress(&mut self, progress: f32) {
        self.progress = progress.clamp(0.0, 1.0);
        if self.progress >= 1.0 {
            self.loading_complete = true;
        }
    }

    pub fn is_loading_complete(&self) -> bool {
        self.loading_complete
    }
}

impl Default for LoadingScene {
    fn default() -> Self {
        Self::new()
    }
}

impl Scene for LoadingScene {
    fn scene_type(&self) -> SceneType {
        SceneType::Loading
    }

    fn on_enter(&mut self) {
        tracing::info!("Entering Loading Screen");
        self.progress = 0.0;
        self.loading_complete = false;
    }

    fn on_exit(&mut self) {
        tracing::info!("Exiting Loading Screen");
    }

    fn update(&mut self, _delta_time: f32) {
        // Simulate loading progress or check async loading tasks
    }

    fn render(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Render loading screen with progress bar
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
