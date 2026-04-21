//! DirectX 11 Context - Full RHI Integration
//! Wraps Dx11Device + SwapChain for easy use

use std::sync::Arc;
use tracing::{info, error, warn};

use crate::graphics::rhi::device::*;
use crate::graphics::rhi::types::*;
use crate::graphics::rhi::dx11::device_dx11::Dx11Device;
use crate::graphics::rhi::dx11::swapchain_dx11::Dx11SwapChain;

#[cfg(target_os = "windows")]
use windows::Win32::Foundation::HWND;

/// DX11 Configuration
pub struct Dx11Config {
    pub prefer_discrete_gpu: bool,
    pub debug_enabled: bool,
    pub validation_enabled: bool,
    pub vsync: bool,
}

impl Default for Dx11Config {
    fn default() -> Self {
        Self {
            prefer_discrete_gpu: true,
            debug_enabled: cfg!(debug_assertions),
            validation_enabled: cfg!(debug_assertions),
            vsync: true,
        }
    }
}

pub struct Dx11Context {
    pub device: Arc<Dx11Device>,
    pub swapchain: Option<Arc<Dx11SwapChain>>,
    pub hwnd: isize,
    pub width: u32,
    pub height: u32,
    pub config: Dx11Config,
    pub depth_stencil_view: Option<ResourceHandle>,
    pub render_target_view: Option<ResourceHandle>,
}

impl Dx11Context {
    pub fn new(hwnd: isize, width: u32, height: u32) -> Result<Self, String> {
        Self::new_with_config(hwnd, width, height, Dx11Config::default())
    }

    pub fn new_with_config(hwnd: isize, width: u32, height: u32, config: Dx11Config) -> Result<Self, String> {
        info!(target: "dx11", "=== Dx11Context::new_with_config START ===");
        info!(target: "dx11", "HWND: {:?}, Size: {}x{}", hwnd, width, height);
        info!(target: "dx11", "Config: debug={}, validation={}, prefer_discrete={}, vsync={}", 
              config.debug_enabled, config.validation_enabled, config.prefer_discrete_gpu, config.vsync);

        // Create DX11 device
        let device = Dx11Device::new(config.debug_enabled, config.validation_enabled)
            .map_err(|e| format!("Failed to create DX11 device: {:?}", e))?;
        
        info!(target: "dx11", "DX11 Device created: {}", device.get_device_name());

        let mut context = Self {
            device: Arc::new(device),
            swapchain: None,
            hwnd,
            width,
            height,
            config,
            depth_stencil_view: None,
            render_target_view: None,
        };

        // Create swapchain
        context.create_swapchain(width, height)?;
        
        info!(target: "dx11", "<<< Dx11Context::new_with_config COMPLETE ===");
        Ok(context)
    }

    fn create_swapchain(&mut self, width: u32, height: u32) -> Result<(), String> {
        info!(target: "dx11", "Creating swapchain: {}x{}", width, height);

        #[cfg(target_os = "windows")]
        {
            use windows::Win32::Foundation::HWND;
            
            // Get factory and device from Dx11Device
            let factory = self.device.get_factory();
            let d3d_device = self.device.get_device();
            let hwnd = HWND(self.hwnd);
            
            // Create swapchain using factory and device
            let swapchain = Dx11SwapChain::new(
                factory,
                d3d_device,
                hwnd,
                width,
                height,
                TextureFormat::Bgra8Unorm, // Default backbuffer format
                self.config.vsync,
            ).map_err(|e| format!("Failed to create swapchain: {:?}", e))?;

            let rtv_handle = ResourceHandle(swapchain.get_back_buffer().0);
            info!(target: "dx11", "Render target view created: {:?}", rtv_handle);

            // Create depth stencil buffer
            let depth_stencil = self.create_depth_stencil(width, height);
            
            self.swapchain = Some(Arc::new(swapchain));
            self.render_target_view = Some(rtv_handle);
            self.depth_stencil_view = depth_stencil;

            info!(target: "dx11", "Swapchain created successfully");
            Ok(())
        }

        #[cfg(not(target_os = "windows"))]
        {
            Err("DX11 is only available on Windows".to_string())
        }
    }

    fn create_depth_stencil(&self, width: u32, height: u32) -> Option<ResourceHandle> {
        info!(target: "dx11", "Creating depth stencil buffer: {}x{}", width, height);

        let texture_desc = TextureDescription {
            dimension: TextureDimension::D2,
            texture_type: TextureType::Texture2D,
            width,
            height,
            depth: 1,
            depth_or_array_layers: 1,
            mip_levels: 1,
            format: TextureFormat::D32Float,
            usage: TextureUsage::DEPTH_STENCIL,
            initial_state: ResourceState::DepthWrite,
        };

        match self.device.create_texture(&texture_desc) {
            Ok(texture) => {
                info!(target: "dx11", "Depth texture created: {:?}", texture);
                
                let view_desc = TextureViewDescription {
                    view_type: TextureViewType::DepthStencil,
                    format: TextureFormat::D32Float,
                    most_detailed_mip: 0,
                    mip_level_count: 1,
                    first_array_slice: 0,
                    array_slice_count: 1,
                };

                match self.device.create_texture_view(texture, &view_desc) {
                    Ok(dsv) => {
                        info!(target: "dx11", "Depth stencil view created: {:?}", dsv);
                        Some(dsv)
                    }
                    Err(e) => {
                        error!(target: "dx11", "Failed to create depth stencil view: {:?}", e);
                        None
                    }
                }
            }
            Err(e) => {
                error!(target: "dx11", "Failed to create depth texture: {:?}", e);
                None
            }
        }
    }

    pub fn begin_frame(&self) {
        info!(target: "dx11", "begin_frame");
    }

    pub fn end_frame(&self) {
        info!(target: "dx11", "end_frame");
        if let Some(ref swapchain) = self.swapchain {
            if let Err(e) = swapchain.present() {
                error!(target: "dx11", "Failed to present: {:?}", e);
            }
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) -> Result<(), String> {
        info!(target: "dx11", "Resizing to: {}x{}", width, height);
        
        self.width = width;
        self.height = height;

        // Recreate swapchain and depth buffer
        self.create_swapchain(width, height)?;
        
        Ok(())
    }

    pub fn set_viewport(&self, x: i32, y: i32, width: u32, height: u32) {
        info!(target: "dx11", "set_viewport: {}x{} at {}x{}", width, height, x, y);
        // Viewport will be set via command list
    }

    pub fn clear(&self, color: Option<[f32; 4]>, depth: Option<f32>) {
        info!(target: "dx11", "clear: color={:?}, depth={:?}", color, depth);
        // Clear will be done via command list
    }

    pub fn get_device(&self) -> &Arc<Dx11Device> {
        &self.device
    }

    pub fn get_swapchain(&self) -> Option<&Arc<Dx11SwapChain>> {
        self.swapchain.as_ref()
    }

    pub fn get_render_target_view(&self) -> Option<ResourceHandle> {
        self.render_target_view
    }

    pub fn get_depth_stencil_view(&self) -> Option<ResourceHandle> {
        self.depth_stencil_view
    }

    pub fn get_device_name(&self) -> &str {
        self.device.get_device_name()
    }
}
