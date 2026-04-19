//! DirectX 11 Buffer - stub

use bytemuck::Pod;
use tracing::info;

pub struct Dx11Buffer {
    pub vertex_count: u32,
    pub index_count: u32,
    pub stride: u32,
    pub is_indexed: bool,
}

impl Dx11Buffer {
    pub fn create_vertex_buffer<T: Pod>(_device: &(), data: &[T]) -> Result<Self, String> {
        let stride = std::mem::size_of::<T>() as u32;
        info!(target: "dx11", "Vertex buffer: {} verts", data.len());
        Ok(Self {
            vertex_count: data.len() as u32,
            index_count: 0,
            stride,
            is_indexed: false,
        })
    }

    pub fn create_index_buffer(_device: &(), data: &[u32]) -> Result<Self, String> {
        info!(target: "dx11", "Index buffer: {} indices", data.len());
        Ok(Self {
            vertex_count: 0,
            index_count: data.len() as u32,
            stride: 4,
            is_indexed: true,
        })
    }

    pub fn create_constant_buffer<T: Pod>(_device: &()) -> Result<Self, String> {
        let stride = std::mem::size_of::<T>() as u32;
        Ok(Self {
            vertex_count: 0,
            index_count: 0,
            stride,
            is_indexed: false,
        })
    }

    pub fn bind(&self, _context: &(), _slot: u32) {}
    pub fn bind_index(&self, _context: &()) {}
    pub fn bind_constant(&self, _context: &(), _slot: u32, _stage: u32) {}
}

pub fn set_topology(_context: &()) {}
