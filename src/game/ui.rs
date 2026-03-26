//! UI System for RTGC-0.8
//! Handles HUD, menus, tooltips, and all user interface elements

use crate::game::player::{PlayerState, CameraMode};
use crate::game::skills::PlayerSkills;
use crate::game::interaction::{InteractableType, InteractionResult};
use crate::game::weather::WeatherState;
use nalgebra::Vector2;

/// UI visibility flags
#[derive(Debug, Clone, Copy)]
pub struct UIVisibility {
    pub hud: bool,
    pub crosshair: bool,
    pub interaction_prompt: bool,
    pub minimap: bool,
    pub speedometer: bool,
    pub fuel_gauge: bool,
    pub compass: bool,
    pub clock: bool,
    pub notifications: bool,
    pub debug_overlay: bool,
}

impl Default for UIVisibility {
    fn default() -> Self {
        Self {
            hud: true,
            crosshair: true,
            interaction_prompt: true,
            minimap: true,
            speedometer: false,
            fuel_gauge: false,
            compass: true,
            clock: true,
            notifications: true,
            debug_overlay: false,
        }
    }
}

/// HUD state data
#[derive(Debug, Clone)]
pub struct HUDData {
    /// Player health (0.0 - 100.0)
    pub health: f32,
    /// Player stamina (0.0 - 100.0)
    pub stamina: f32,
    /// Current speed (km/h)
    pub speed_kmh: f32,
    /// Fuel level (0.0 - 1.0)
    pub fuel: f32,
    /// Money (rubles)
    pub money: u32,
    /// Current time (hours, 0-24)
    pub time_hours: f32,
    /// Weather description
    pub weather: String,
    /// Location name
    pub location: String,
    /// Player state
    pub player_state: PlayerState,
    /// Camera mode
    pub camera_mode: CameraMode,
    /// Current gear (for vehicles)
    pub gear: i8,
    /// RPM (for vehicles)
    pub rpm: f32,
    /// Engine temperature (0.0 - 1.0, normal is 0.3-0.6)
    pub engine_temp: f32,
    /// Compass heading (0-359 degrees)
    pub heading: f32,
    /// Coordinates (X, Y, Z)
    pub position: Vec2,
    /// Altitude (meters)
    pub altitude: f32,
}

impl Default for HUDData {
    fn default() -> Self {
        Self {
            health: 100.0,
            stamina: 100.0,
            speed_kmh: 0.0,
            fuel: 1.0,
            money: 50000,
            time_hours: 12.0,
            weather: "Clear".to_string(),
            location: "Novosibirsk".to_string(),
            player_state: PlayerState::OnFoot,
            camera_mode: CameraMode::ThirdPerson,
            gear: 0,
            rpm: 0.0,
            engine_temp: 0.5,
            heading: 0.0,
            position: Vec2::ZERO,
            altitude: 0.0,
        }
    }
}

/// Notification message
#[derive(Debug, Clone)]
pub struct Notification {
    pub message: String,
    pub notification_type: NotificationType,
    pub duration: f32,
    pub age: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NotificationType {
    Info,
    Success,
    Warning,
    Error,
    SkillUp,
    Achievement,
}

/// Interaction prompt data
#[derive(Debug, Clone)]
pub struct InteractionPrompt {
    pub visible: bool,
    pub text: String,
    pub key: String,
    pub distance: f32,
}

/// Minimap data
#[derive(Debug, Clone)]
pub struct MinimapData {
    /// Player position on map (normalized 0-1)
    pub player_pos: Vec2,
    /// Player rotation (radians)
    pub player_rotation: f32,
    /// Zoom level (1.0 = max zoom)
    pub zoom: f32,
    /// Marked waypoints
    pub waypoints: Vec<Waypoint>,
    /// Visible vehicles
    pub vehicles: Vec<VehicleMarker>,
    /// Visible NPCs
    pub npcs: Vec<NPCMarker>,
}

#[derive(Debug, Clone)]
pub struct Waypoint {
    pub name: String,
    pub position: Vec2,
    pub waypoint_type: WaypointType,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WaypointType {
    Mission,
    Custom,
    Home,
    Work,
    FuelStation,
    Shop,
}

#[derive(Debug, Clone)]
pub struct VehicleMarker {
    pub vehicle_id: u32,
    pub position: Vec2,
    pub vehicle_type: String,
    pub is_player_owned: bool,
}

#[derive(Debug, Clone)]
pub struct NPCMarker {
    pub npc_id: u32,
    pub position: Vec2,
    pub name: String,
}

impl Default for MinimapData {
    fn default() -> Self {
        Self {
            player_pos: Vec2::new(0.5, 0.5),
            player_rotation: 0.0,
            zoom: 1.0,
            waypoints: Vec::new(),
            vehicles: Vec::new(),
            npcs: Vec::new(),
        }
    }
}

/// Main UI manager
pub struct UIManager {
    visibility: UIVisibility,
    hud_data: HUDData,
    notifications: Vec<Notification>,
    interaction_prompt: Option<InteractionPrompt>,
    minimap_data: MinimapData,
    /// Active skill notifications
    skill_notifications: Vec<(String, u32)>, // (skill_name, new_rank)
}

impl UIManager {
    pub fn new() -> Self {
        Self {
            visibility: UIVisibility::default(),
            hud_data: HUDData::default(),
            notifications: Vec::new(),
            interaction_prompt: None,
            minimap_data: MinimapData::default(),
            skill_notifications: Vec::new(),
        }
    }

    /// Update UI systems
    pub fn update(&mut self, dt: f32) {
        // Update notifications
        for notification in &mut self.notifications {
            notification.age += dt;
        }
        self.notifications.retain(|n| n.age < n.duration);

        // Update skill notifications
        // (would fade out over time in actual rendering)

        // Update interaction prompt visibility based on distance
        if let Some(prompt) = &mut self.interaction_prompt {
            if prompt.distance > 3.0 {
                prompt.visible = false;
            }
        }
    }

    /// Add a notification
    pub fn add_notification(&mut self, message: String, notification_type: NotificationType) {
        let duration = match notification_type {
            NotificationType::Info => 3.0,
            NotificationType::Success => 4.0,
            NotificationType::Warning => 5.0,
            NotificationType::Error => 6.0,
            NotificationType::SkillUp => 5.0,
            NotificationType::Achievement => 8.0,
        };

        self.notifications.push(Notification {
            message,
            notification_type,
            duration,
            age: 0.0,
        });
    }

    /// Add skill up notification
    pub fn notify_skill_up(&mut self, skill_name: String, new_rank: u32) {
        self.skill_notifications.push((skill_name.clone(), new_rank));
        self.add_notification(
            format!("{} increased to Rank {}!", skill_name, new_rank),
            NotificationType::SkillUp,
        );
    }

    /// Set interaction prompt
    pub fn set_interaction_prompt(&mut self, text: String, distance: f32) {
        self.interaction_prompt = Some(InteractionPrompt {
            visible: true,
            text,
            key: "F".to_string(),
            distance,
        });
        self.visibility.interaction_prompt = true;
    }

    /// Clear interaction prompt
    pub fn clear_interaction_prompt(&mut self) {
        self.interaction_prompt = None;
        self.visibility.interaction_prompt = false;
    }

    /// Update HUD data
    pub fn update_hud(&mut self, data: HUDData) {
        self.hud_data = data;
        
        // Auto-show speedometer when in vehicle
        if matches!(data.player_state, PlayerState::InVehicle { .. }) {
            self.visibility.speedometer = true;
            self.visibility.fuel_gauge = true;
        } else {
            self.visibility.speedometer = false;
            self.visibility.fuel_gauge = false;
        }
    }

    /// Get current HUD data
    pub fn get_hud_data(&self) -> &HUDData {
        &self.hud_data
    }

    /// Update minimap data
    pub fn update_minimap(&mut self, data: MinimapData) {
        self.minimap_data = data;
    }

    /// Get minimap data
    pub fn get_minimap_data(&self) -> &MinimapData {
        &self.minimap_data
    }

    /// Toggle HUD visibility
    pub fn toggle_hud(&mut self) {
        self.visibility.hud = !self.visibility.hud;
    }

    /// Toggle minimap
    pub fn toggle_minimap(&mut self) {
        self.visibility.minimap = !self.visibility.minimap;
    }

    /// Toggle debug overlay
    pub fn toggle_debug_overlay(&mut self) {
        self.visibility.debug_overlay = !self.visibility.debug_overlay;
    }

    /// Get UI visibility
    pub fn get_visibility(&self) -> UIVisibility {
        self.visibility
    }

    /// Get active notifications
    pub fn get_notifications(&self) -> &[Notification] {
        &self.notifications
    }

    /// Get interaction prompt
    pub fn get_interaction_prompt(&self) -> Option<&InteractionPrompt> {
        self.interaction_prompt.as_ref()
    }

    /// Handle weather change notification
    pub fn notify_weather_change(&mut self, weather: &WeatherState) {
        self.add_notification(
            format!("Weather changed: {}", weather.description()),
            NotificationType::Info,
        );
        self.hud_data.weather = weather.description().to_string();
    }

    /// Handle money change
    pub fn notify_money_change(&mut self, amount: i32, reason: &str) {
        if amount > 0 {
            self.add_notification(
                format!("+{} ₽ ({})", amount, reason),
                NotificationType::Success,
            );
        } else if amount < 0 {
            self.add_notification(
                format!("-{} ₽ ({})", -amount, reason),
                NotificationType::Warning,
            );
        }
    }

    /// Reset UI state (on scene change)
    pub fn reset(&mut self) {
        self.notifications.clear();
        self.interaction_prompt = None;
        self.skill_notifications.clear();
        self.visibility = UIVisibility::default();
    }
}

impl Default for UIManager {
    fn default() -> Self {
        Self::new()
    }
}

// Helper methods for weather description
impl WeatherState {
    pub fn description(&self) -> &str {
        match self {
            WeatherState::Clear => "Clear",
            WeatherState::Cloudy => "Cloudy",
            WeatherState::Rain { .. } => "Rain",
            WeatherState::Snow { .. } => "Snow",
            WeatherState::Fog { .. } => "Fog",
            WeatherState::Storm { .. } => "Storm",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ui_manager_creation() {
        let ui = UIManager::new();
        assert!(ui.get_visibility().hud);
        assert!(ui.get_notifications().is_empty());
    }

    #[test]
    fn test_add_notification() {
        let mut ui = UIManager::new();
        ui.add_notification("Test".to_string(), NotificationType::Info);
        assert_eq!(ui.get_notifications().len(), 1);
    }

    #[test]
    fn test_toggle_hud() {
        let mut ui = UIManager::new();
        assert!(ui.get_visibility().hud);
        ui.toggle_hud();
        assert!(!ui.get_visibility().hud);
    }

    #[test]
    fn test_interaction_prompt() {
        let mut ui = UIManager::new();
        ui.set_interaction_prompt("Enter Vehicle".to_string(), 2.0);
        assert!(ui.get_interaction_prompt().is_some());
        
        ui.clear_interaction_prompt();
        assert!(ui.get_interaction_prompt().is_none());
    }
}
