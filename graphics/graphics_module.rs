use winit::window::Window;
use std::sync::Arc;
use glow::Context;
use crate::graphics::renderer::{Renderer, MenuState};
use crate::graphics::rhi::{RhiFactory, RhiConfig, IDevice, GraphicsBackend};

pub struct GraphicsContext {
    pub renderer: Renderer,
    gl: Context,
    window: Arc<Window>,
    rhi_config: RhiConfig,
}

impl GraphicsContext {
    pub fn new(window: Arc<Window>, gl: Context) -> Result<Self, Box<dyn std::error::Error>> {
        let renderer = Renderer::new(gl.clone())?;
        
        Ok(Self {
            renderer,
            gl,
            window,
            rhi_config: RhiConfig::default(),
        })
    }
    
    /// Initialize the RHI with Vulkan or DirectX 12 backend
    pub fn initialize_rhi(&mut self) -> Result<Box<dyn IDevice>, Box<dyn std::error::Error>> {
        let config = RhiConfig {
            backend: RhiFactory::get_preferred_backend(),
            enable_validation: cfg!(debug_assertions),
            enable_debug_layers: cfg!(debug_assertions),
            max_frames_in_flight: 3,
            descriptor_pool_size: 1024,
        };
        
        log::info!("Initializing RHI with {} backend", config.backend.as_str());
        
        let device = RhiFactory::create_device(config.backend)?;
        log::info!("RHI device created: {}", device.get_device_name());
        
        Ok(device)
    }
    
    pub fn render(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        unsafe {
            self.gl.clear_color(0.1, 0.2, 0.3, 1.0);
            self.gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
        }
        
        self.renderer.render()?;
        
        Ok(())
    }
    
    pub fn get_rhi_config(&self) -> &RhiConfig {
        &self.rhi_config
    }
    
    pub fn set_rhi_config(&mut self, config: RhiConfig) {
        self.rhi_config = config;
    }
    
    pub fn resize(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        // Update OpenGL viewport
        unsafe {
            self.gl.viewport(0, 0, size.width as i32, size.height as i32);
        }
        
        // Update renderer camera aspect ratio
        self.renderer.camera.update_aspect_ratio(size.width as f32 / size.height as f32);
    }
    
    pub fn get_gl(&self) -> &Context {
        &self.gl
    }
}