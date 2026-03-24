// RHI Factory - Backend selection and device creation
// Provides unified interface for creating GPU devices across platforms

use crate::graphics::rhi::device::IDevice;
use crate::graphics::rhi::types::RhiResult;
use std::sync::Arc;
use tracing;

#[cfg(feature = "dx12")]
use crate::graphics::rhi::dx12_module::Dx12Device;

#[cfg(feature = "vulkan")]
use crate::graphics::rhi::vulkan_module::VkDevice;

/// Graphics API backend selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RhiBackend {
    /// Auto-select best available backend
    Auto,
    /// DirectX 12 (Windows only)
    Dx12,
    /// Vulkan (Cross-platform)
    Vulkan,
}

/// RHI configuration
#[derive(Debug, Clone)]
pub struct RhiConfig {
    pub backend: RhiBackend,
    pub debug_enabled: bool,
    pub validation_enabled: bool,
    pub preferred_adapter_index: Option<usize>,
}

impl Default for RhiConfig {
    fn default() -> Self {
        Self {
            backend: RhiBackend::Auto,
            debug_enabled: false,
            validation_enabled: false,
            preferred_adapter_index: None,
        }
    }
}

/// RHI Factory - creates and manages GPU devices
pub struct RhiFactory;

impl RhiFactory {
    /// Create a new RHI device with the specified configuration
    pub fn create_device(config: &RhiConfig) -> RhiResult<Arc<dyn IDevice>> {
        let selected_backend = Self::select_backend(config.backend)?;
        
        tracing::info!("Creating RHI device with backend: {:?}", selected_backend);
        
        match selected_backend {
            #[cfg(feature = "dx12")]
            RhiBackend::Dx12 => {
                #[cfg(target_os = "windows")]
                {
                    let device = Dx12Device::new(config.debug_enabled, config.validation_enabled)?;
                    Ok(Arc::new(device))
                }
                #[cfg(not(target_os = "windows"))]
                {
                    Err(crate::graphics::rhi::types::RhiError::Unsupported(
                        "DirectX 12 is only available on Windows".to_string(),
                    ))
                }
            }
            
            #[cfg(feature = "vulkan")]
            RhiBackend::Vulkan => {
                let device = VkDevice::new(config.debug_enabled, config.validation_enabled)?;
                Ok(Arc::new(device))
            }
            
            _ => Err(crate::graphics::rhi::types::RhiError::Unsupported(
                "No suitable RHI backend available".to_string(),
            )),
        }
    }
    
    /// Select the best available backend based on configuration and platform
    fn select_backend(requested: RhiBackend) -> RhiResult<RhiBackend> {
        match requested {
            RhiBackend::Auto => Self::detect_best_backend(),
            RhiBackend::Dx12 => {
                #[cfg(all(feature = "dx12", target_os = "windows"))]
                {
                    Ok(RhiBackend::Dx12)
                }
                #[cfg(not(all(feature = "dx12", target_os = "windows")))]
                {
                    Err(crate::graphics::rhi::types::RhiError::Unsupported(
                        "DirectX 12 is not available on this platform".to_string(),
                    ))
                }
            }
            RhiBackend::Vulkan => {
                #[cfg(feature = "vulkan")]
                {
                    Ok(RhiBackend::Vulkan)
                }
                #[cfg(not(feature = "vulkan"))]
                {
                    Err(crate::graphics::rhi::types::RhiError::Unsupported(
                        "Vulkan support is not compiled in".to_string(),
                    ))
                }
            }
        }
    }
    
    /// Detect the best available backend for the current platform
    fn detect_best_backend() -> RhiResult<RhiBackend> {
        // Priority order: Vulkan > DX12
        // Vulkan is preferred due to cross-platform support
        
        #[cfg(feature = "vulkan")]
        {
            if Self::is_vulkan_available() {
                tracing::info!("Vulkan backend detected as available");
                return Ok(RhiBackend::Vulkan);
            }
        }
        
        #[cfg(all(feature = "dx12", target_os = "windows"))]
        {
            if Self::is_dx12_available() {
                tracing::info!("DirectX 12 backend detected as available");
                return Ok(RhiBackend::Dx12);
            }
        }
        
        Err(crate::graphics::rhi::types::RhiError::InitializationFailed(
            "No suitable RHI backend found".to_string(),
        ))
    }
    
    /// Check if Vulkan is available on the system
    #[cfg(feature = "vulkan")]
    fn is_vulkan_available() -> bool {
        use ash::vk;
        use ash::Entry;
        
        let entry = match Entry::new() {
            Ok(e) => e,
            Err(_) => return false,
        };
        
        match entry.enumerate_instance_extension_properties(None) {
            Ok(extensions) => {
                extensions.iter().any(|ext| {
                    let name = unsafe { std::ffi::CStr::from_ptr(ext.extension_name.as_ptr()) };
                    name.to_str().unwrap_or("").contains("VK_KHR_surface")
                })
            }
            Err(_) => false,
        }
    }
    
    /// Check if DX12 is available on the system
    #[cfg(all(feature = "dx12", target_os = "windows"))]
    fn is_dx12_available() -> bool {
        // DX12 is available on Windows 10+ with compatible hardware
        // For now, assume it's available if we're on Windows
        true
    }
    
    /// Get list of available backends for this platform
    pub fn get_available_backends() -> Vec<RhiBackend> {
        let mut backends = Vec::new();
        
        #[cfg(feature = "vulkan")]
        {
            backends.push(RhiBackend::Vulkan);
        }
        
        #[cfg(all(feature = "dx12", target_os = "windows"))]
        {
            backends.push(RhiBackend::Dx12);
        }
        
        backends
    }
}

/// Helper function to create a default RHI device
pub fn create_default_device() -> RhiResult<Arc<dyn IDevice>> {
    RhiFactory::create_device(&RhiConfig::default())
}
