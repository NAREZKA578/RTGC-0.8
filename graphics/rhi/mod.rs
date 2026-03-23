//! RHI (Render Hardware Interface) Module
//! Provides abstraction over different graphics APIs (Vulkan, DX12, OpenGL)

pub mod types;
pub mod device;
pub mod factory;
pub mod gl;
pub mod rhi_module;
pub mod resource_manager;

#[cfg(feature = "dx12")]
pub mod dx12;

#[cfg(feature = "vulkan")]
pub mod vulkan;

pub use rhi_module::*;
pub use resource_manager::*;
