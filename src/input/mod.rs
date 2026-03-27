//! Input module for RTGC-0.8

pub mod mapping;
pub mod input_module;
pub mod gamepad;
pub mod action_map;

pub use mapping::{InputAction, InputMapping, MouseButton};
pub use input_module::InputManager;
pub use gamepad::{GamepadButton, GamepadAxis, GamepadState, GamepadManager, GamepadConfig};
pub use action_map::{ActionMap, ActionState};
