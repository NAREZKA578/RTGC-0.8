//! DirectX 11 Buffer

use tracing::info;
use windows::Win32::Graphics::Direct3D11::{
    ID3D11Buffer, ID3D11Device, ID3D11DeviceContext, D3D11_BIND_CONSTANT_BUFFER,
    D3D11_BIND_INDEX_BUFFER, D3D11_BIND_VERTEX_BUFFER, D3D11_BUFFER_DESC, D3D11_CPU_ACCESS_WRITE,
    D3D11_PRIMITIVE_TOPOLOGY_TRIANGLELIST, D3D11_SUBRESOURCE_DATA, D3D11_USAGE_DEFAULT,
    D3D11_USAGE_DYNAMIC,
};

use windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT_R32_UINT;

pub struct Dx11Buffer {
    pub buffer: ID3D11Buffer,
    pub vertex_count: u32,
    pub index_count: u32,
    pub stride: u32,
    pub is_indexed: bool,
}

impl Dx11Buffer {
    pub fn create_vertex_buffer<T: bytemuck::Pod>(
        device: &ID3D11Device,
        data: &[T],
    ) -> Result<Self, String> {
        let stride = std::mem::size_of::<T>() as u32;
        let byte_size = (stride * data.len() as u32) as u32;

        let buffer_desc = D3D11_BUFFER_DESC {
            ByteWidth: byte_size,
            Usage: D3D11_USAGE_DEFAULT,
            BindFlags: D3D11_BIND_VERTEX_BUFFER,
            CPUAccessFlags: windows::Win32::Graphics::Direct3D11::D3D11_CPU_ACCESS_FLAG(0),
            MiscFlags: windows::Win32::Graphics::Direct3D11::D3D11_RESOURCE_FLAGS(0),
            StructureByteStride: stride,
        };

        let data_ptr = data.as_ptr() as *const std::ffi::c_void;
        let subresource = D3D11_SUBRESOURCE_DATA {
            pSysMem: data_ptr,
            SysMemPitch: 0,
            SysMemSlicePitch: 0,
        };

        let buffer = unsafe {
            device
                .CreateBuffer(&buffer_desc, Some(&subresource))
                .map_err(|e| format!("Failed to create vertex buffer: {:?}", e))?
        };

        info!(target: "dx11", "Vertex buffer created with {} vertices", data.len());

        Ok(Self {
            buffer,
            vertex_count: data.len() as u32,
            index_count: 0,
            stride,
            is_indexed: false,
        })
    }

    pub fn create_index_buffer(device: &ID3D11Device, data: &[u32]) -> Result<Self, String> {
        let byte_size = (std::mem::size_of::<u32>() * data.len()) as u32;

        let buffer_desc = D3D11_BUFFER_DESC {
            ByteWidth: byte_size,
            Usage: D3D11_USAGE_DEFAULT,
            BindFlags: D3D11_BIND_INDEX_BUFFER,
            CPUAccessFlags: windows::Win32::Graphics::Direct3D11::D3D11_CPU_ACCESS_FLAG(0),
            MiscFlags: windows::Win32::Graphics::Direct3D11::D3D11_RESOURCE_FLAGS(0),
            StructureByteStride: std::mem::size_of::<u32>() as u32,
        };

        let data_ptr = data.as_ptr() as *const std::ffi::c_void;
        let subresource = D3D11_SUBRESOURCE_DATA {
            pSysMem: data_ptr,
            SysMemPitch: 0,
            SysMemSlicePitch: 0,
        };

        let buffer = unsafe {
            device
                .CreateBuffer(&buffer_desc, Some(&subresource))
                .map_err(|e| format!("Failed to create index buffer: {:?}", e))?
        };

        info!(target: "dx11", "Index buffer created with {} indices", data.len());

        Ok(Self {
            buffer,
            vertex_count: 0,
            index_count: data.len() as u32,
            stride: std::mem::size_of::<u32>() as u32,
            is_indexed: true,
        })
    }

    pub fn create_constant_buffer<T: bytemuck::Pod>(device: &ID3D11Device) -> Result<Self, String> {
        let byte_size = std::mem::size_of::<T>() as u32;
        let aligned_size = (byte_size + 15) & !15;

        let buffer_desc = D3D11_BUFFER_DESC {
            ByteWidth: aligned_size,
            Usage: D3D11_USAGE_DYNAMIC,
            BindFlags: D3D11_BIND_CONSTANT_BUFFER,
            CPUAccessFlags: D3D11_CPU_ACCESS_WRITE,
            MiscFlags: windows::Win32::Graphics::Direct3D11::D3D11_RESOURCE_FLAGS(0),
            StructureByteStride: aligned_size,
        };

        let buffer = unsafe {
            device
                .CreateBuffer(&buffer_desc, None)
                .map_err(|e| format!("Failed to create constant buffer: {:?}", e))?
        };

        info!(target: "dx11", "Constant buffer created");

        Ok(Self {
            buffer,
            vertex_count: 0,
            index_count: 0,
            stride: aligned_size,
            is_indexed: false,
        })
    }

    pub fn set_topology(context: &ID3D11DeviceContext) {
        unsafe {
            context.IASetPrimitiveTopology(D3D11_PRIMITIVE_TOPOLOGY_TRIANGLELIST);
        }
    }

    pub fn bind(&self, context: &ID3D11DeviceContext, slot: u32) {
        if self.is_indexed {
            unsafe {
                context.IASetIndexBuffer(Some(&self.buffer), DXGI_FORMAT_R32_UINT, 0);
            }
        } else {
            unsafe {
                let buffers = [Some(&self.buffer)];
                let strides = [self.stride];
                let offsets = [0u32];
                context.IASetVertexBuffers(
                    slot,
                    1,
                    Some(buffers.as_slice()),
                    Some(strides.as_slice()),
                    Some(offsets.as_slice()),
                );
            }
        }
    }
}
