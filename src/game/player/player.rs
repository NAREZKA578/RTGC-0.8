//! Player character structure and logic

use nalgebra::{Vector3, UnitQuaternion};
use crate::game::vehicle::VehicleId;
use crate::save::PlayerMoneyData;
use crate::save::PlayerSkillsData;

/// Inventory item (placeholder for now)
#[derive(Debug, Clone)]
pub struct InventoryItem {
    pub name: String,
    pub count: u32,
    pub item_type: ItemType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ItemType {
    Tool,
    Consumable,
    Material,
    Other,
}

impl Default for InventoryItem {
    fn default() -> Self {
        Self {
            name: String::new(),
            count: 0,
            item_type: ItemType::Other,
        }
    }
}

/// Player state - where the player currently is
#[derive(Debug, Clone, PartialEq)]
pub enum PlayerState {
    OnFoot,
    InVehicle { vehicle_id: usize, seat: u8 },
    InHelicopter { heli_id: usize, seat: u8 },
    InCrane,
}

impl Default for PlayerState {
    fn default() -> Self {
        PlayerState::OnFoot
    }
}

/// Main player structure
#[derive(Debug, Clone)]
pub struct Player {
    pub id: usize,                    // generational ID
    pub name: String,
    pub is_male: bool,
    pub height: f32,                  // 1.6..2.0
    pub skin_color: [f32; 3],
    pub face_variant: u8,             // 0..7
    pub hair_style: u8,
    pub hair_color: [f32; 3],
    
    pub position: Vector3<f32>,
    pub rotation: UnitQuaternion<f32>,
    pub velocity: Vector3<f32>,
    
    pub state: PlayerState,
    
    pub stamina: f32,                 // 0.0..1.0
    pub health: f32,
    pub money: PlayerMoneyData,
    pub inventory: Vec<InventoryItem>,
    
    pub skills: PlayerSkillsData,
    
    // Physical body (for walking)
    pub capsule_body_id: Option<usize>,   // ID in PhysicsWorld
}

impl Default for Player {
    fn default() -> Self {
        Self {
            id: 0,
            name: String::from("Player"),
            is_male: true,
            height: 1.75,
            skin_color: [0.8, 0.65, 0.5],
            face_variant: 0,
            hair_style: 0,
            hair_color: [0.3, 0.2, 0.1],
            
            position: Vector3::new(0.0, 2.0, 0.0),
            rotation: UnitQuaternion::identity(),
            velocity: Vector3::zeros(),
            
            state: PlayerState::OnFoot,
            
            stamina: 1.0,
            health: 100.0,
            money: PlayerMoneyData::default(),
            inventory: Vec::new(),
            
            skills: PlayerSkillsData::default(),
            
            capsule_body_id: None,
        }
    }
}

impl Player {
    /// Create a new player with custom name
    pub fn new(name: String) -> Self {
        let mut player = Self::default();
        player.name = name;
        player.id = 1;
        player
    }
    
    /// Check if player is on foot
    pub fn is_on_foot(&self) -> bool {
        matches!(self.state, PlayerState::OnFoot)
    }
    
    /// Check if player is in any vehicle
    pub fn is_in_vehicle(&self) -> bool {
        matches!(self.state, PlayerState::InVehicle { .. } | PlayerState::InHelicopter { .. } | PlayerState::InCrane)
    }
    
    /// Get current vehicle ID if in vehicle
    pub fn get_vehicle_id(&self) -> Option<usize> {
        match self.state {
            PlayerState::InVehicle { vehicle_id, .. } => Some(vehicle_id),
            PlayerState::InHelicopter { heli_id, .. } => Some(heli_id),
            _ => None,
        }
    }
    
    /// Exit current vehicle
    pub fn exit_vehicle(&mut self) {
        self.state = PlayerState::OnFoot;
    }
    
    /// Enter a vehicle
    pub fn enter_vehicle(&mut self, vehicle_id: usize, seat: u8) {
        self.state = PlayerState::InVehicle { vehicle_id, seat };
    }
    
    /// Update player position from physics body
    pub fn sync_from_physics(&mut self, new_pos: Vector3<f32>, new_vel: Vector3<f32>) {
        self.position = new_pos;
        self.velocity = new_vel;
    }
}
