//! DirectX 11 Context - Compatibility Wrapper
//! Re-exports from rhi/dx11/context_dx11.rs for backwards compatibility

pub use crate::graphics::rhi::dx11::context_dx11::{Dx11Config, Dx11Context as Dx11GraphicsContext};

/// Legacy re-export for compatibility
pub type Dx11GraphicsContext = crate::graphics::rhi::dx11::context_dx11::Dx11Context;

/// Legacy config re-export  
pub type Dx11Config = crate::graphics::rhi::dx11::context_dx11::Dx11Config;
