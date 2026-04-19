//! DirectX 11 SwapChain - stub

use tracing::info;

pub struct Dx11SwapChain {
    pub width: u32,
    pub height: u32,
    vsync: bool,
}

impl Dx11SwapChain {
    pub fn new(
        _factory: &(),
        _device: &(),
        hwnd: isize,
        width: u32,
        height: u32,
    ) -> Result<Self, String> {
        info!(target: "dx11", "SwapChain {}x{} HWND: {:?}", width, height, hwnd);
        Ok(Self {
            width,
            height,
            vsync: true,
        })
    }

    pub fn set_vsync(&mut self, vsync: bool) {
        self.vsync = vsync;
    }
    pub fn get_vsync(&self) -> bool {
        self.vsync
    }
    pub fn present(&self) -> Result<(), String> {
        Ok(())
    }
    pub fn get_back_buffer(&self) -> Option<&()> {
        None
    }
    pub fn resize(&mut self, width: u32, height: u32) -> Result<(), String> {
        self.width = width;
        self.height = height;
        Ok(())
    }
    pub fn begin_frame(&mut self) {}
    pub fn end_frame(&mut self) {}
}
