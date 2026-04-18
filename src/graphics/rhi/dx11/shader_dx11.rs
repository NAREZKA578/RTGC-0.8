//! DirectX 11 Shader - компиляция HLSL

use tracing::{error, info};
use windows::Win32::Foundation::HLOCAL;
use windows::Win32::Graphics::Direct3D11::{
    ID3D11Device, ID3D11PixelShader, ID3D11VertexShader, ID3DBlob,
};
use windows::Win32::System::Com::{
    CoCreateInstance, ID3DBlob as ComID3DBlob, CLSCTX_INPROC_SERVER,
};

pub struct Dx11Shader {
    pub vertex_shader: Option<ID3D11VertexShader>,
    pub pixel_shader: Option<ID3D11PixelShader>,
    pub input_layout: Option<windows::Win32::Graphics::Direct3D11::ID3D11InputLayout>,
    pub bytecode: Vec<u8>,
    pub pixel_bytecode: Vec<u8>,
}

impl Dx11Shader {
    pub fn from_hlsl(
        device: &ID3D11Device,
        vertex_hlsl: &str,
        pixel_hlsl: &str,
    ) -> Result<Self, String> {
        info!(target: "dx11", "Compiling HLSL shaders...");

        // Compile vertex shader
        let (vertex_blob, vertex_bytecode) = compile_hlsl(device, vertex_hlsl, "vs_5_0")?;

        let vertex_shader = unsafe {
            device
                .CreateVertexShader(Some(&vertex_blob), None, None, None)
                .map_err(|e| format!("Failed to create vertex shader: {:?}", e))?
        };

        // Create input layout
        let input_layout = create_input_layout(device, &vertex_blob)?;

        info!(target: "dx11", "Vertex shader compiled");

        // Compile pixel shader
        let (pixel_blob, pixel_bytecode) = compile_hlsl(device, pixel_hlsl, "ps_5_0")?;

        let pixel_shader = unsafe {
            device
                .CreatePixelShader(Some(&pixel_blob), None, None, None)
                .map_err(|e| format!("Failed to create pixel shader: {:?}", e))?
        };

        info!(target: "dx11", "Pixel shader compiled");

        Ok(Self {
            vertex_shader: Some(vertex_shader),
            pixel_shader: Some(pixel_shader),
            input_layout: Some(input_layout),
            bytecode: vertex_bytecode,
            pixel_bytecode,
        })
    }

    pub fn bind(&self, context: &windows::Win32::Graphics::Direct3D11::ID3D11DeviceContext) {
        unsafe {
            if let Some(ref vs) = self.vertex_shader {
                context.VSSetShader(Some(vs), None, None);
            }
            if let Some(ref ps) = self.pixel_shader {
                context.PSSetShader(Some(ps), None, None);
            }
            if let Some(ref layout) = self.input_layout {
                context.IASetInputLayout(Some(&layout));
            }
        }
    }

    pub fn unbind(&self, context: &windows::Win32::Graphics::Direct3D11::ID3D11DeviceContext) {
        unsafe {
            context.VSSetShader(None, None, None);
            context.PSSetShader(None, None, None);
            context.IASetInputLayout(None);
        }
    }
}

fn compile_hlsl(
    device: &ID3D11Device,
    source: &str,
    target: &str,
) -> Result<(windows::Win32::System::Com::ID3DBlob, Vec<u8>), String> {
    unsafe {
        let hr = windows::Win32::Foundation::D3DCompile(
            source.as_ptr() as *const std::ffi::c_void,
            source.len(),
            None,
            None,
            None,
            "main",
            target,
            0,
            0,
            Some(&mut std::ptr::null_mut()),
            Some(&mut std::ptr::null_mut()),
        );

        if hr != windows::Win32::Foundation::S_OK {
            return Err(format!("D3DCompile failed with HRESULT: {:?}", hr));
        }

        // Get compiled bytecode
        let mut blob: Option<windows::Win32::System::Com::ID3DBlob> = None;
        let hr = windows::Win32::Foundation::D3DCompile(
            source.as_ptr() as *const std::ffi::c_void,
            source.len(),
            None,
            None,
            None,
            "main",
            target,
            0,
            0,
            Some(&mut std::ptr::null_mut()),
            Some(&mut blob),
        );

        if hr != windows::Win32::Foundation::S_OK {
            return Err(format!("D3DCompile failed: {:?}", hr));
        }

        let bytecode = blob
            .as_ref()
            .map(|b| {
                let size = b.GetBufferSize();
                let ptr = b.GetBufferPointer();
                std::slice::from_raw_parts(ptr as *const u8, size).to_vec()
            })
            .unwrap_or_default();

        let blob = blob.ok_or("Failed to get blob")?;

        Ok((blob, bytecode))
    }
}

fn create_input_layout(
    device: &ID3D11Device,
    vertex_shader_blob: &windows::Win32::System::Com::ID3DBlob,
) -> Result<windows::Win32::Graphics::Direct3D11::ID3D11InputLayout, String> {
    // Standard vertex layout: position (vec3), normal (vec3), texcoord (vec2)
    let layout = [
        windows::Win32::Graphics::Direct3D11::D3D11_INPUT_ELEMENT_DESC {
            SemanticName: b"POSITION\0".as_ptr() as *const i8,
            SemanticIndex: 0,
            Format: windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT_R32G32B32_FLOAT,
            InputSlot: 0,
            AlignedByteOffset: 0,
            InputSlotClass: windows::Win32::Graphics::Direct3D11::D3D11_INPUT_PER_VERTEX_DATA,
            InstanceDataStepRate: 0,
        },
        windows::Win32::Graphics::Direct3D11::D3D11_INPUT_ELEMENT_DESC {
            SemanticName: b"NORMAL\0".as_ptr() as *const i8,
            SemanticIndex: 0,
            Format: windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT_R32G32B32_FLOAT,
            InputSlot: 0,
            AlignedByteOffset: 12,
            InputSlotClass: windows::Win32::Graphics::Direct3D11::D3D11_INPUT_PER_VERTEX_DATA,
            InstanceDataStepRate: 0,
        },
        windows::Win32::Graphics::Direct3D11::D3D11_INPUT_ELEMENT_DESC {
            SemanticName: b"TEXCOORD\0".as_ptr() as *const i8,
            SemanticIndex: 0,
            Format: windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT_R32G32_FLOAT,
            InputSlot: 0,
            AlignedByteOffset: 24,
            InputSlotClass: windows::Win32::Graphics::Direct3D11::D3D11_INPUT_PER_VERTEX_DATA,
            InstanceDataStepRate: 0,
        },
    ];

    let layout = unsafe {
        device
            .CreateInputLayout(&layout, 1, Some(vertex_shader_blob), None, None)
            .map_err(|e| format!("Failed to create input layout: {:?}", e))?
    };

    Ok(layout)
}
