//! DirectX 11 Backend - Stubs for compilation
//! Full implementation requires windows crate API work

pub mod buffer_dx11;
pub mod context_dx11;
pub mod device_dx11;
pub mod pipeline_dx11;
pub mod shader_dx11;
pub mod swapchain_dx11;
pub mod texture_dx11;

pub use context_dx11::Dx11Context;
pub use device_dx11::Dx11Device;
pub use swapchain_dx11::Dx11SwapChain;
