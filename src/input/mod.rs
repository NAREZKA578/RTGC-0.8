//! Input module for RTGC-0.7

pub mod mapping;
pub mod input_module;

pub use mapping::{InputAction, InputMapping, MouseButton};
pub use input_module::InputManager;
