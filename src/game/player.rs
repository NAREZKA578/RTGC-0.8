//! Player Module - Pedestrian Character System
//! Implements player character with physics capsule, inventory, skills, and vehicle interaction

use nalgebra::{Vector3, UnitQuaternion, Point3};
use crate::physics::{RigidBody, Shape, LAYER_WORLD, LAYER_VEHICLE, LAYER_CARGO, LAYER_TRIGGER};
use crate::game::skills::PlayerSkills;
use crate::game::InventoryItem;

/// Collision layer for player character
pub const LAYER_PLAYER: u32 = 0b10000;

/// Player state - either on foot or in a vehicle
#[derive(Debug, Clone, PartialEq)]
pub enum PlayerState {
    OnFoot,
    InVehicle { vehicle_index: usize, vehicle_id: u64, seat_index: usize },
}

impl PlayerState {
    /// Get vehicle id if in vehicle
    pub fn vehicle_id(&self) -> Option<u64> {
        match self {
            PlayerState::OnFoot => None,
            PlayerState::InVehicle { vehicle_id, .. } => Some(*vehicle_id),
        }
    }

    /// Get vehicle index if in vehicle
    pub fn vehicle_index(&self) -> Option<usize> {
        match self {
            PlayerState::OnFoot => None,
            PlayerState::InVehicle { vehicle_index, .. } => Some(*vehicle_index),
        }
    }

    /// Get seat index if in vehicle
    pub fn seat_index(&self) -> Option<usize> {
        match self {
            PlayerState::OnFoot => None,
            PlayerState::InVehicle { seat_index, .. } => Some(*seat_index),
        }
    }
}

/// Camera mode for player
#[derive(Debug, Clone, PartialEq)]
pub enum CameraMode {
    FirstPerson,
    ThirdPerson { distance: f32, yaw: f32, pitch: f32 },
}

impl Default for CameraMode {
    fn default() -> Self {
        CameraMode::ThirdPerson {
            distance: 4.0,
            yaw: 0.0,
            pitch: 0.3,
        }
    }
}

/// Player character structure
pub struct Player {
    /// Player name
    pub name: String,
    
    /// Gender: true = male, false = female
    pub is_male: bool,
    
    /// Height in meters (default 1.93, range 1.50 - 2.10)
    pub height: f32,
    
    /// Skin color [r, g, b]
    pub skin_color: [f32; 3],
    
    /// Face variant index
    pub face_variant: u8,
    
    /// Hair style index
    pub hair_style: u8,
    
    /// Hair color [r, g, b]
    pub hair_color: [f32; 3],
    
    /// Player skills
    pub skills: PlayerSkills,
    
    /// Money: RUB, CNY, USD
    pub money: PlayerWallet,
    
    /// Inventory (max 60kg)
    pub inventory: Vec<InventoryItem>,
    pub inventory_weight: f32,
    pub max_inventory_weight: f32,
    
    /// Physics body index (capsule RigidBody)
    pub body_index: Option<usize>,
    
    /// Current state
    pub state: PlayerState,
    
    /// Camera mode
    pub camera_mode: CameraMode,
    
    /// Stamina (0.0 - 1.0)
    pub stamina: f32,
    pub max_stamina: f32,
    
    /// Is sprinting
    pub is_sprinting: bool,
    
    /// Is jumping
    pub is_jumping: bool,
    
    /// Can jump (on ground)
    pub can_jump: bool,
}

/// Player wallet with multiple currencies
#[derive(Debug, Clone)]
pub struct PlayerWallet {
    pub rub: f64,
    pub cny: f64,
    pub usd: f64,
}

impl Default for PlayerWallet {
    fn default() -> Self {
        Self {
            rub: 0.0,
            cny: 0.0,
            usd: 0.0,
        }
    }
}

impl Player {
    /// Create a new player with default values
    pub fn new(name: String) -> Self {
        let height = 1.93;
        let radius = 0.35;
        
        Self {
            name,
            is_male: true,
            height,
            skin_color: [0.8, 0.65, 0.55],
            face_variant: 0,
            hair_style: 0,
            hair_color: [0.25, 0.18, 0.12],
            skills: PlayerSkills::new(),
            money: PlayerWallet::default(),
            inventory: Vec::new(),
            inventory_weight: 0.0,
            max_inventory_weight: 60.0,
            body_index: None,
            state: PlayerState::OnFoot,
            camera_mode: CameraMode::default(),
            stamina: 1.0,
            max_stamina: 1.0,
            is_sprinting: false,
            is_jumping: false,
            can_jump: true,
        }
    }
    
    /// Create player capsule RigidBody
    pub fn create_physics_body(&self, position: Vector3<f32>) -> RigidBody {
        let radius = 0.35;
        let height = self.height;
        
        let mut body = RigidBody::new_capsule(position, 80.0, radius, height);
        body.collision_layer = LAYER_PLAYER;
        body.collision_mask = LAYER_WORLD | LAYER_VEHICLE | LAYER_CARGO | LAYER_TRIGGER;
        body.linear_damping = 0.95;
        body.angular_damping = 0.9;
        
        body
    }
    
    /// Apply movement force to player capsule
    pub fn apply_movement_force(&mut self, direction: Vector3<f32>, force: f32, physics: &mut crate::physics::PhysicsWorld) {
        if let Some(idx) = self.body_index {
            if let Some(body) = physics.get_body_mut(idx) {
                let move_force = direction * force;
                body.forces += move_force;
            }
        }
    }
    
    /// Jump action
    pub fn jump(&mut self, physics: &mut crate::physics::PhysicsWorld) {
        if !self.can_jump || self.state != PlayerState::OnFoot {
            return;
        }
        
        if let Some(idx) = self.body_index {
            if let Some(body) = physics.get_body_mut(idx) {
                // Jump impulse (not parkour - limited jump)
                body.velocity.y = 4.0;
                self.is_jumping = true;
                self.can_jump = false;
                self.stamina -= 0.1;
            }
        }
    }
    
    /// Sprint toggle
    pub fn toggle_sprint(&mut self) {
        if self.stamina > 0.2 {
            self.is_sprinting = !self.is_sprinting;
        } else {
            self.is_sprinting = false;
        }
    }
    
    /// Update player stamina
    pub fn update_stamina(&mut self, dt: f32) {
        if self.is_sprinting && self.state == PlayerState::OnFoot {
            self.stamina -= 0.3 * dt;
            if self.stamina <= 0.0 {
                self.stamina = 0.0;
                self.is_sprinting = false;
            }
        } else {
            self.stamina += 0.2 * dt;
            if self.stamina > self.max_stamina {
                self.stamina = self.max_stamina;
            }
        }
    }
    
    /// Enter vehicle
    pub fn enter_vehicle(&mut self, vehicle_index: usize, vehicle_id: u64, seat_index: usize) {
        self.state = PlayerState::InVehicle {
            vehicle_index,
            vehicle_id,
            seat_index,
        };

        // Disable physics body when in vehicle
        if let Some(idx) = self.body_index {
            self.body_index = None;
            // Note: physics body should be removed/disabled by engine
        }
    }
    
    /// Exit vehicle
    pub fn exit_vehicle(&mut self, exit_position: Vector3<f32>, physics: &mut crate::physics::PhysicsWorld) {
        if let PlayerState::InVehicle { .. } = self.state {
            self.state = PlayerState::OnFoot;
            
            // Re-enable physics body at exit position
            let body = self.create_physics_body(exit_position);
            self.body_index = Some(physics.add_body(body));
        }
    }
    
    /// Switch camera mode
    pub fn toggle_camera(&mut self) {
        match self.camera_mode {
            CameraMode::FirstPerson => {
                self.camera_mode = CameraMode::ThirdPerson {
                    distance: 4.0,
                    yaw: 0.0,
                    pitch: 0.3,
                };
            }
            CameraMode::ThirdPerson { .. } => {
                self.camera_mode = CameraMode::FirstPerson;
            }
        }
    }
    
    /// Zoom camera
    pub fn zoom_camera(&mut self, delta: f32) {
        if let CameraMode::ThirdPerson { distance, yaw, pitch } = &mut self.camera_mode {
            *distance += delta;
            *distance = (*distance).clamp(1.0, 8.0);
        }
    }
    
    /// Rotate camera around player
    pub fn rotate_camera(&mut self, yaw_delta: f32, pitch_delta: f32) {
        if let CameraMode::ThirdPerson { yaw, pitch, .. } = &mut self.camera_mode {
            *yaw += yaw_delta;
            *pitch += pitch_delta;
            *pitch = (*pitch).clamp(-0.5, 0.8);
        }
    }
    
    /// Get camera position based on mode
    pub fn get_camera_position(&self, player_pos: Vector3<f32>, player_rotation: UnitQuaternion<f32>) -> Vector3<f32> {
        match &self.camera_mode {
            CameraMode::FirstPerson => {
                // Eye level
                player_pos + Vector3::new(0.0, self.height - 0.15, 0.0)
            }
            CameraMode::ThirdPerson { distance, yaw, pitch } => {
                let cos_yaw = yaw.cos();
                let sin_yaw = yaw.sin();
                let cos_pitch = pitch.cos();
                let sin_pitch = pitch.sin();
                
                let offset = Vector3::new(
                    -distance * cos_pitch * sin_yaw,
                    distance * sin_pitch,
                    -distance * cos_pitch * cos_yaw,
                );
                
                player_pos + Vector3::new(0.0, self.height * 0.8, 0.0) + offset
            }
        }
    }
    
    /// Add item to inventory
    pub fn add_to_inventory(&mut self, item: InventoryItem) -> bool {
        let total_weight = self.inventory_weight + item.total_weight();
        if total_weight > self.max_inventory_weight {
            return false;
        }

        self.inventory.push(item);
        self.inventory_weight = total_weight;
        true
    }

    /// Remove item from inventory
    pub fn remove_from_inventory(&mut self, index: usize) -> Option<InventoryItem> {
        if index < self.inventory.len() {
            let item = self.inventory.remove(index);
            self.inventory_weight -= item.total_weight();
            Some(item)
        } else {
            None
        }
    }
    
    /// Check if player can interact with something at ray hit
    pub fn can_interact(&self, distance: f32) -> bool {
        distance <= 3.0
    }
}

/// Player input structure for network synchronization
#[derive(Debug, Clone, Default)]
pub struct PlayerInput {
    pub move_forward: f32,
    pub move_right: f32,
    pub jump: bool,
    pub sprint: bool,
    pub interact: bool,
    pub camera_yaw: f32,
    pub camera_pitch: f32,
    pub camera_zoom: f32,
    pub toggle_camera: bool,
    pub exit_vehicle: bool,
}
