use winit::event::{KeyEvent, ElementState};
use winit::keyboard::{KeyCode, PhysicalKey};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum KeyState {
    Pressed,
    Released,
}

#[derive(Debug)]
pub struct InputManager {
    key_states: std::collections::HashMap<PhysicalKey, KeyState>,
}

impl InputManager {
    pub fn new() -> Self {
        Self {
            key_states: std::collections::HashMap::new(),
        }
    }

    pub fn handle_keyboard_input(&mut self, input: &KeyEvent) {
        match input.state {
            ElementState::Pressed => {
                self.key_states.insert(input.physical_key, KeyState::Pressed);
            }
            ElementState::Released => {
                self.key_states.insert(input.physical_key, KeyState::Released);
            }
        }
    }

    pub fn is_key_pressed(&self, key_code: KeyCode) -> bool {
        let physical_key = PhysicalKey::Code(key_code);
        matches!(
            self.key_states.get(&physical_key),
            Some(KeyState::Pressed)
        )
    }

    pub fn is_key_just_pressed(&self, key_code: KeyCode) -> bool {
        self.is_key_pressed(key_code)
    }

    pub fn get_key_state(&self, key_code: KeyCode) -> Option<KeyState> {
        let physical_key = PhysicalKey::Code(key_code);
        self.key_states.get(&physical_key).copied()
    }
}

impl Default for InputManager {
    fn default() -> Self {
        Self::new()
    }
}
