//! Player module for RTGC-0.8
//! Handles player character, creation, appearance, and skills

pub mod player;
pub mod appearance;
pub mod skills;
pub mod character_creation;

pub use player::{Player, PlayerState};
pub use appearance::Appearance;
pub use skills::Skills;
pub use character_creation::CharacterCreation;
