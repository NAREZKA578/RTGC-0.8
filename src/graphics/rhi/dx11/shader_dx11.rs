//! DirectX 11 Shader - stub

use tracing::info;

pub struct Dx11Shader {
    pub bytecode: Vec<u8>,
    pub pixel_bytecode: Vec<u8>,
}

impl Dx11Shader {
    pub fn from_hlsl(
        _device: &(),
        _vertex_hlsl: &str,
        _pixel_hlsl: &str,
    ) -> Result<Self, String> {
        info!(target: "dx11", "Shader compiled (stub)");
        Ok(Self {
            bytecode: vec![],
            pixel_bytecode: vec![],
        })
    }

    pub fn bind(&self, _context: &()) {}
    pub fn unbind(&self, _context: &()) {}
}
