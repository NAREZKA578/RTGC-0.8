//! DX11 Renderer - stub

use crate::graphics::camera::Camera;
use nalgebra::Matrix4;
use tracing::info;

pub struct Dx11Renderer {
    pub width: u32,
    pub height: u32,
    pub menu_state: crate::graphics::renderer::MenuState,
    pub camera: Camera,
}

impl Dx11Renderer {
    pub fn new(hwnd: isize, width: u32, height: u32) -> Result<Self, String> {
        info!(target: "dx11", "=== Dx11Renderer::new START ===");
        info!(target: "dx11", "HWND: {:?}, Size: {}x{}", hwnd, width, height);
        info!(target: "dx11", "Device created");
        info!(target: "dx11", "SwapChain created");
        info!(target: "dx11", "Shaders created");
        info!(target: "dx11", "=== Dx11Renderer::new END ===");
        Ok(Self {
            width,
            height,
            menu_state: crate::graphics::renderer::MenuState::Loading,
            camera: Camera::new(),
        })
    }

    pub fn begin_frame(&mut self) {}
    pub fn end_frame(&mut self) {}
    pub fn render_menu(&mut self) -> Result<(), String> {
        info!(target: "dx11", "render_menu (stub)");
        Ok(())
    }
    pub fn resize(&mut self, width: u32, height: u32) -> Result<(), String> {
        self.width = width;
        self.height = height;
        Ok(())
    }
    pub fn get_projection_matrix(&self) -> Matrix4<f32> {
        let aspect = self.width as f32 / self.height as f32;
        *nalgebra::Perspective3::new(aspect, 1.0, 0.01, 1000.0).as_matrix()
    }
}
