//! DirectX 11 Context - stub

use tracing::info;

pub struct Dx11GraphicsContext {
    pub width: u32,
    pub height: u32,
    pub hwnd: isize,
}

impl Dx11GraphicsContext {
    pub fn new(hwnd: isize, width: u32, height: u32) -> Result<Self, String> {
        info!(target: "dx11", "=== Dx11GraphicsContext ===");
        info!(target: "dx11", "HWND: {:?}, Size: {}x{}", hwnd, width, height);
        info!(target: "dx11", "DX11 initialized");
        Ok(Self {
            width,
            height,
            hwnd,
        })
    }

    pub fn set_viewport(&self) {}
    pub fn clear(&self, _color: Option<[f32; 4]>) {}
    pub fn begin_frame(&self) {}
    pub fn end_frame(&self) {}
    pub fn resize(&mut self, width: u32, height: u32) -> Result<(), String> {
        self.width = width;
        self.height = height;
        Ok(())
    }
    pub fn get_device_name(&self) -> &str {
        "DirectX 11"
    }
    pub fn render_simple_quad(&self) {}
}
