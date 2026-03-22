//! Physics Module for RTGC-0.7
//! Provides rigid body dynamics, collision detection, constraints, and vehicle physics

pub mod physics_module;
pub mod arena_allocator;
pub mod spatial_hash;
pub mod async_physics;
pub mod thread_pool;
pub mod fracture_component;
pub mod helicopter;
pub mod advanced_vehicle;
pub mod vehicle;
pub mod deformable_terrain;
pub mod constraints;

// Re-export collision layer constants
pub use physics_module::{LAYER_WORLD, LAYER_VEHICLE, LAYER_CARGO, LAYER_TRIGGER};
pub use physics_module::{PhysicsWorld, RigidBody, Ray, RaycastHit, Aabb};
pub use arena_allocator::ArenaAllocator;
pub use spatial_hash::SpatialHash;
pub use async_physics::AsyncPhysicsEngine;
pub use thread_pool::ThreadPool;
pub use fracture_component::FractureComponent;
pub use helicopter::{Helicopter, HelicopterConfig, HelicopterControls, HelicopterState};
pub use vehicle::{Vehicle, VehicleControls};
pub use advanced_vehicle::AdvancedVehicle;
pub use deformable_terrain::DeformableTerrainComponent;
pub use constraints::{SpringConstraint, RaycastSuspension};
