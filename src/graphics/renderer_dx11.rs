//! DX11 Renderer - полный рендерер через DirectX 11

use crate::graphics::camera::Camera;
use crate::graphics::mesh::Mesh;
use crate::graphics::texture::Texture;
use nalgebra::{Matrix4, UnitQuaternion, Vector3};
use std::collections::HashMap;
use tracing::{info, warn};
use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Direct3D11::{
    D3D11CreateDevice, ID3D11Device, ID3D11DeviceContext, D3D11_CREATE_DEVICE_BGRA_SUPPORT,
    D3D11_CREATE_DEVICE_FLAG, D3D11_SDK_VERSION, D3D_DRIVER_TYPE_HARDWARE, D3D_FEATURE_LEVEL,
};
use windows::Win32::Graphics::Dxgi::{
    CreateDXGIFactory, ID3D10Multithread, IDXGIFactory, DXGI_MODE_DESC,
    DXGI_MODE_ROTATION_UNSPECIFIED, DXGI_MODE_SCALING_UNSPECIFIED, DXGI_RATIONAL, DXGI_SAMPLE_DESC,
    DXGI_SWAP_CHAIN_DESC, DXGI_SWAP_EFFECT_DISCARD, DXGI_USAGE_RENDER_TARGET_OUTPUT,
};

use crate::graphics::rhi::dx11::buffer_dx11::Dx11Buffer;
use crate::graphics::rhi::dx11::device_dx11::Dx11Device;
use crate::graphics::rhi::dx11::shader_dx11::Dx11Shader;
use crate::graphics::rhi::dx11::swapchain_dx11::Dx11SwapChain;

pub struct Dx11Renderer {
    pub device: Dx11Device,
    pub swap_chain: Option<Dx11SwapChain>,
    pub shader: Option<Dx11Shader>,
    pub camera: Camera,
    pub width: u32,
    pub height: u32,
    pub menu_state: crate::graphics::renderer::MenuState,
    meshes: HashMap<String, Mesh>,
}

impl Dx11Renderer {
    pub fn new(hwnd: isize, width: u32, height: u32) -> Result<Self, String> {
        info!(target: "dx11", "=== Dx11Renderer::new START ===");
        info!(target: "dx11", "HWND: {:?}, Size: {}x{}", hwnd, width, height);

        // Create device
        let device = Dx11Device::new(false)?;

        info!(target: "dx11", "Device created");

        // Get factory
        let factory = device.get_factory();

        // Create swap chain
        let swap_chain = Dx11SwapChain::new(factory, device.get_device(), hwnd, width, height)?;

        info!(target: "dx11", "SwapChain created");

        // Create shaders
        let shader = create_default_shaders(device.get_device())?;

        info!(target: "dx11", "Shaders created");
        info!(target: "dx11", "=== Dx11Renderer::new END ===");

        Ok(Self {
            device,
            swap_chain: Some(swap_chain),
            shader: Some(shader),
            camera: Camera::new(),
            width,
            height,
            menu_state: crate::graphics::renderer::MenuState::Loading,
            meshes: HashMap::new(),
        })
    }

    pub fn begin_frame(&mut self) {
        if let Some(ref swap_chain) = self.swap_chain {
            swap_chain.begin_frame();
        }
    }

    pub fn end_frame(&mut self) {
        if let Some(ref swap_chain) = self.swap_chain {
            swap_chain.end_frame();
        }
    }

    pub fn render_menu(&mut self) -> Result<(), String> {
        // Simple colored quad for menu background
        let vertices: [f32; 24] = [
            // Position (x, y, z), Normal (x, y, z), TexCoord (u, v)
            -1.0, -1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, -1.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 1.0,
            1.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, -1.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 1.0,
        ];

        let indices: [u32; 6] = [0, 1, 2, 0, 2, 3];

        let device = self.device.get_device();
        let context = self.device.get_context();

        // Create vertex buffer
        let vb = Dx11Buffer::create_vertex_buffer(device, &vertices)?;
        let ib = Dx11Buffer::create_index_buffer(device, &indices)?;

        // Bind shader
        if let Some(ref shader) = self.shader {
            shader.bind(context);
        }

        // Set topology
        Dx11Buffer::set_topology(context);

        // Bind vertex buffer
        vb.bind(context, 0);

        // Bind index buffer
        let _ = ib.bind(context, 0);

        // Create constant buffer for MVP matrix
        let mvp = Matrix4::identity();

        // Draw
        unsafe {
            context.DrawIndexed(6, 0, 0);
        }

        // Unbind
        if let Some(ref shader) = self.shader {
            shader.unbind(context);
        }

        Ok(())
    }

    pub fn resize(&mut self, width: u32, height: u32) -> Result<(), String> {
        self.width = width;
        self.height = height;

        if let Some(ref mut swap_chain) = self.swap_chain {
            swap_chain.resize(width, height)?;
        }

        Ok(())
    }

    pub fn get_projection_matrix(&self) -> Matrix4<f32> {
        let aspect = self.width as f32 / self.height as f32;
        *nalgebra::Perspective3::new(aspect, 1.0, 0.01, 1000.0).as_matrix()
    }
}

fn create_default_shaders(device: &ID3D11Device) -> Result<Dx11Shader, String> {
    let vertex_shader = r#"
        cbuffer Constants : register(b0) {
            float4x4 g_mvp;
        };
        
        struct VS_INPUT {
            float3 Position : POSITION;
            float3 Normal : NORMAL;
            float2 TexCoord : TEXCOORD;
        };
        
        struct VS_OUTPUT {
            float4 Position : SV_POSITION;
            float3 Normal : NORMAL;
            float2 TexCoord : TEXCOORD;
        };
        
        VS_OUTPUT main(VS_INPUT input) {
            VS_OUTPUT output;
            output.Position = mul(float4(input.Position, 1.0), g_mvp);
            output.Normal = input.Normal;
            output.TexCoord = input.TexCoord;
            return output;
        }
    "#;

    let pixel_shader = r#"
        struct PS_INPUT {
            float4 Position : SV_POSITION;
            float3 Normal : NORMAL;
            float2 TexCoord : TEXCOORD;
        };
        
        float4 main(PS_INPUT input) : SV_TARGET {
            float3 light_dir = normalize(float3(0.5, 1.0, 0.5));
            float ndotl = max(dot(normalize(input.Normal), light_dir), 0.0);
            float3 color = float3(0.02, 0.02, 0.05);
            color += ndotl * 0.3;
            return float4(color, 1.0);
        }
    "#;

    Dx11Shader::from_hlsl(device, vertex_shader, pixel_shader)
}
