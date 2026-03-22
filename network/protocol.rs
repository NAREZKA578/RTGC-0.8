// Network Protocol Module for RTGC
// Defines serializable structures for multiplayer game state synchronization
// Note: This file only contains data structures - no network implementation yet

use serde::{Serialize, Deserialize};
use nalgebra::{Vector3, Quaternion};

/// Complete game state for network synchronization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    /// World seed for deterministic generation
    pub world_seed: u64,
    
    /// Player vehicle position
    pub vehicle_position: [f32; 3],
    
    /// Player vehicle rotation (quaternion)
    pub vehicle_rotation: [f32; 4],
    
    /// Vehicle velocity
    pub vehicle_velocity: [f32; 3],
    
    /// Vehicle angular velocity
    pub vehicle_angular_velocity: [f32; 3],
    
    /// Vehicle fuel level (0.0 - 1.0)
    pub vehicle_fuel: f32,
    
    /// Vehicle health (0.0 - 1.0)
    pub vehicle_health: f32,
    
    /// Vehicle engine RPM
    pub vehicle_rpm: f32,
    
    /// Current gear (-1 = reverse, 0 = neutral, 1+ = forward)
    pub vehicle_gear: i32,
    
    /// Active cargo mission ID (if any)
    pub current_mission_id: Option<String>,
    
    /// Cargo weight in kg (if attached)
    pub cargo_weight_kg: Option<f32>,
    
    /// Time of day (0.0 - 24.0)
    pub time_of_day: f32,
    
    /// Weather seed for deterministic weather
    pub weather_seed: u64,
    
    /// Player reputation/score
    pub reputation: i32,
    
    /// Completed mission IDs
    pub completed_missions: Vec<String>,
    
    /// Helicopter position (if active)
    pub helicopter_position: Option<[f32; 3]>,
    
    /// Helicopter rotation (if active)
    pub helicopter_rotation: Option<[f32; 4]>,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            world_seed: 0,
            vehicle_position: [0.0; 3],
            vehicle_rotation: [0.0, 0.0, 0.0, 1.0], // identity quaternion
            vehicle_velocity: [0.0; 3],
            vehicle_angular_velocity: [0.0; 3],
            vehicle_fuel: 1.0,
            vehicle_health: 1.0,
            vehicle_rpm: 0.0,
            vehicle_gear: 0,
            current_mission_id: None,
            cargo_weight_kg: None,
            time_of_day: 12.0,
            weather_seed: 0,
            reputation: 0,
            completed_missions: Vec::new(),
            helicopter_position: None,
            helicopter_rotation: None,
        }
    }
}

/// Network message types for client-server communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkMessage {
    /// Client requests to join server
    JoinRequest {
        player_name: String,
    },
    
    /// Server accepts client join
    JoinAccepted {
        client_id: u64,
        initial_state: GameState,
    },
    
    /// Client sends input state
    InputUpdate {
        throttle: f32,
        steering: f32,
        brake: f32,
        handbrake: bool,
    },
    
    /// Server broadcasts game state update
    StateUpdate {
        game_state: GameState,
        tick: u64,
    },
    
    /// Client acknowledges state
    StateAck {
        tick: u64,
    },
    
    /// Mission started
    MissionStart {
        mission_id: String,
        pickup_location: [f32; 3],
        delivery_location: [f32; 3],
        cargo_type: String,
        reward: i32,
    },
    
    /// Mission completed
    MissionComplete {
        mission_id: String,
        success: bool,
        reward_earned: i32,
    },
}

/// Player input state for network transmission
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerInput {
    pub throttle: f32,
    pub brake: f32,
    pub steering: f32,
    pub handbrake: bool,
    pub diff_lock_rear: bool,
    pub diff_lock_front: bool,
    pub low_range: bool,
    pub winch_active: bool,
}

impl Default for PlayerInput {
    fn default() -> Self {
        Self {
            throttle: 0.0,
            brake: 0.0,
            steering: 0.0,
            handbrake: false,
            diff_lock_rear: false,
            diff_lock_front: false,
            low_range: false,
            winch_active: false,
        }
    }
}
