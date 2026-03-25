//! Network Module for RTGC
//! Provides multiplayer synchronization infrastructure

pub mod protocol;

pub use protocol::{GameState, NetworkMessage, PlayerInput};
