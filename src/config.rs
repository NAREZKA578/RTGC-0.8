//! Configuration module for RTGC engine
//! Provides centralized configuration for all engine subsystems

use serde::{Deserialize, Serialize};
use std::path::Path;
use tracing::{warn, info};

/// Main configuration structure containing all subsystem configs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub graphics: GraphicsConfig,
    pub physics: PhysicsConfig,
    pub world: WorldConfig,
    pub input: InputConfig,
    pub audio: AudioConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            graphics: GraphicsConfig::default(),
            physics: PhysicsConfig::default(),
            world: WorldConfig::default(),
            input: InputConfig::default(),
            audio: AudioConfig::default(),
        }
    }
}

impl Config {
    /// Load configuration from a JSON file
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = serde_json::from_str(&content)?;
        info!("Configuration loaded successfully");
        Ok(config)
    }

    /// Save configuration to a JSON file
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        info!("Configuration saved to {:?}", path.as_ref());
        Ok(())
    }
}

/// Graphics configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphicsConfig {
    pub window_width: u32,
    pub window_height: u32,
    pub fullscreen: bool,
    pub vsync: bool,
    pub max_fps: Option<u32>,
    pub msaa_samples: u32,
    pub shadow_resolution: u32,
    pub max_anisotropy: f32,
    pub lod_bias: f32,
    pub texture_streaming_budget_mb: u32,
    pub backend: String, // "vulkan", "dx12", "opengl"
    pub enable_validation: bool,
}

impl Default for GraphicsConfig {
    fn default() -> Self {
        Self {
            window_width: 1920,
            window_height: 1080,
            fullscreen: false,
            vsync: true,
            max_fps: Some(60),
            msaa_samples: 4,
            shadow_resolution: 2048,
            max_anisotropy: 16.0,
            lod_bias: 0.0,
            texture_streaming_budget_mb: 512,
            backend: "vulkan".to_string(),
            enable_validation: cfg!(debug_assertions),
        }
    }
}

/// Physics configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicsConfig {
    pub substeps: u32,
    pub gravity: [f32; 3],
    pub solver_iterations: u32,
    pub contact_offset: f32,
    pub rest_offset: f32,
    pub max_depenetration_velocity: f32,
    pub enable_ccd: bool,
    pub thread_count: u32,
    pub async_physics: bool,
}

impl Default for PhysicsConfig {
    fn default() -> Self {
        Self {
            substeps: 4,
            gravity: [0.0, -9.81, 0.0],
            solver_iterations: 8,
            contact_offset: 0.01,
            rest_offset: 0.0,
            max_depenetration_velocity: 100.0,
            enable_ccd: true,
            thread_count: num_cpus::get() as u32,
            async_physics: true,
        }
    }
}

/// World configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldConfig {
    pub chunk_size: u32,
    pub render_distance: u32,
    pub terrainlod_distances: Vec<f32>,
    pub max_entities: u32,
    pub streaming_enabled: bool,
    pub save_directory: String,
}

impl Default for WorldConfig {
    fn default() -> Self {
        Self {
            chunk_size: 64,
            render_distance: 10,
            terrainlod_distances: vec![50.0, 100.0, 200.0, 500.0],
            max_entities: 10000,
            streaming_enabled: true,
            save_directory: "saves".to_string(),
        }
    }
}

/// Input configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputConfig {
    pub mouse_sensitivity: f32,
    pub invert_y: bool,
    pub gamepad_enabled: bool,
    pub vibration_enabled: bool,
    pub vibration_strength: f32,
}

impl Default for InputConfig {
    fn default() -> Self {
        Self {
            mouse_sensitivity: 0.1,
            invert_y: false,
            gamepad_enabled: true,
            vibration_enabled: true,
            vibration_strength: 0.5,
        }
    }
}

/// Audio configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfig {
    pub master_volume: f32,
    pub music_volume: f32,
    pub sfx_volume: f32,
    pub voice_volume: f32,
    pub environmental_audio: bool,
    pub doppler_effect: bool,
    pub max_audio_sources: u32,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            master_volume: 1.0,
            music_volume: 0.7,
            sfx_volume: 0.8,
            voice_volume: 0.9,
            environmental_audio: true,
            doppler_effect: true,
            max_audio_sources: 64,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.graphics.window_width, 1920);
        assert_eq!(config.graphics.window_height, 1080);
        assert_eq!(config.physics.substeps, 4);
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let json = serde_json::to_string(&config).expect("Failed to serialize config");
        let loaded: Config = serde_json::from_str(&json).expect("Failed to deserialize config");
        assert_eq!(config.graphics.window_width, loaded.graphics.window_width);
    }
}
