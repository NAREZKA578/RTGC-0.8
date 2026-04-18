//! DirectX 11 Context - заменяет GlContext для DX11 рендеринга

use std::sync::Arc;
use tracing::{error, info};
use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::Graphics::Direct3D11::{
    D3D11CreateDevice, ID3D11DepthStencilView, ID3D11Device, ID3D11DeviceContext,
    ID3D11RenderTargetView, D3D11_BIND_DEPTH_STENCIL, D3D11_CLEAR_DEPTH, D3D11_CLEAR_STENCIL,
    D3D11_CPU_ACCESS_NONE, D3D11_RESOURCE_FLAGS, D3D11_SDK_VERSION, D3D11_USAGE_DEFAULT,
    D3D11_VIEWPORT, D3D_DRIVER_TYPE_HARDWARE,
};
use windows::Win32::Graphics::Dxgi::{
    CreateDXGIFactory, IDXGIFactory, IDXGISwapChain, DXGI_FORMAT_R8G8B8A8_UNORM, DXGI_MODE_DESC,
    DXGI_MODE_SCALING_UNSPECIFIED, DXGI_MODE_SCANLINE_ORDER_UNSPECIFIED, DXGI_RATIONAL,
    DXGI_SAMPLE_DESC, DXGI_SWAP_CHAIN_DESC, DXGI_SWAP_EFFECT_FLIP_DISCARD,
    DXGI_USAGE_RENDER_TARGET_OUTPUT,
};

pub struct Dx11GraphicsContext {
    pub device: ID3D11Device,
    pub context: ID3D11DeviceContext,
    pub swap_chain: IDXGISwapChain,
    pub render_target_view: ID3D11RenderTargetView,
    pub depth_stencil_view: ID3D11DepthStencilView,
    pub hwnd: isize,
    pub width: u32,
    pub height: u32,
}

impl Dx11GraphicsContext {
    pub fn new(hwnd: isize, width: u32, height: u32) -> Result<Self, String> {
        info!(target: "dx11", "=== Dx11GraphicsContext::new START ===");
        info!(target: "dx11", "HWND: {:?}, Size: {}x{}", hwnd, width, height);

        let hwnd_raw = HWND(hwnd as *mut std::ffi::c_void);

        unsafe {
            let factory: IDXGIFactory = CreateDXGIFactory().map_err(|e| {
                error!("Failed to create DXGIFactory: {:?}", e);
                format!("Failed to create DXGIFactory: {:?}", e)
            })?;

            info!(target: "dx11", "DXGIFactory created");

            let adapter = factory.EnumAdapters(0).map_err(|e| {
                error!("Failed to enum adapters: {:?}", e);
                format!("Failed to enum adapters: {:?}", e)
            })?;

            info!(target: "dx11", "Adapter enumerated");

            let mut device: Option<ID3D11Device> = None;
            let mut context: Option<ID3D11DeviceContext> = None;

            D3D11CreateDevice(
                Some(&adapter),
                D3D_DRIVER_TYPE_HARDWARE,
                None,
                0,
                None,
                D3D11_SDK_VERSION,
                &mut device,
                Some(&mut context),
                None,
            )
            .map_err(|e| {
                error!("Failed to create D3D11 device: {:?}", e);
                format!("Failed to create D3D11 device: {:?}", e)
            })?;

            let device = device.unwrap();
            let context = context.unwrap();

            info!(target: "dx11", "D3D11 Device created successfully!");

            let swap_desc = DXGI_SWAP_CHAIN_DESC {
                BufferDesc: DXGI_MODE_DESC {
                    Width: width,
                    Height: height,
                    RefreshRate: DXGI_RATIONAL {
                        Numerator: 60,
                        Denominator: 1,
                    },
                    Format: DXGI_FORMAT_R8G8B8A8_UNORM,
                    ScanlineOrdering: DXGI_MODE_SCANLINE_ORDER_UNSPECIFIED,
                    Scaling: DXGI_MODE_SCALING_UNSPECIFIED,
                },
                SampleDesc: DXGI_SAMPLE_DESC {
                    Count: 1,
                    Quality: 0,
                },
                BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
                BufferCount: 2,
                OutputWindow: hwnd_raw,
                Windowed: windows::Win32::Foundation::TRUE,
                SwapEffect: DXGI_SWAP_EFFECT_FLIP_DISCARD,
                Flags: windows::Win32::Graphics::Dxgi::DXGI_SWAP_CHAIN_FLAG(0),
            };

            let swap_chain = factory.CreateSwapChain(&device, &swap_desc).map_err(|e| {
                error!("Failed to create SwapChain: {:?}", e);
                format!("Failed to create SwapChain: {:?}", e)
            })?;

            info!(target: "dx11", "SwapChain created");

            let buffer = swap_chain
                .GetBuffer(0)
                .map_err(|e| format!("Failed to get buffer: {:?}", e))?;

            let render_target_view = device
                .CreateRenderTargetView(&buffer, None)
                .map_err(|e| format!("Failed to create RTV: {:?}", e))?;

            info!(target: "dx11", "RenderTargetView created");

            let depth_format = windows::Win32::Graphics::Dxgi::DXGI_FORMAT_D24_UNORM_S8_UINT;

            let depth_desc = windows::Win32::Graphics::Direct3D11::D3D11_TEXTURE2D_DESC {
                Width: width,
                Height: height,
                MipLevels: 1,
                ArraySize: 1,
                Format: depth_format,
                SampleDesc: DXGI_SAMPLE_DESC {
                    Count: 1,
                    Quality: 0,
                },
                Usage: D3D11_USAGE_DEFAULT,
                BindFlags: D3D11_BIND_DEPTH_STENCIL,
                CPUAccessFlags: D3D11_CPU_ACCESS_NONE,
                MiscFlags: D3D11_RESOURCE_FLAGS(0),
            };

            let depth_texture = device
                .CreateTexture2D(&depth_desc, None)
                .map_err(|e| format!("Failed to create depth texture: {:?}", e))?;

            let depth_stencil_view = device
                .CreateDepthStencilView(&depth_texture, None)
                .map_err(|e| format!("Failed to create DSV: {:?}", e))?;

            info!(target: "dx11", "DepthStencilView created");
            info!(target: "dx11", "=== Dx11GraphicsContext::new END ===");

            Ok(Self {
                device,
                context,
                swap_chain,
                render_target_view,
                depth_stencil_view,
                hwnd,
                width,
                height,
            })
        }
    }

    pub fn set_viewport(&self) {
        let viewport = D3D11_VIEWPORT {
            TopLeftX: 0.0,
            TopLeftY: 0.0,
            Width: self.width as f32,
            Height: self.height as f32,
            MinDepth: 0.0,
            MaxDepth: 1.0,
        };

        unsafe {
            self.context.RSSetViewports(&[viewport]);
        }
    }

    pub fn clear(&self, color: Option<[f32; 4]>, clear_depth: bool) {
        let clear_color = color.unwrap_or([0.02, 0.02, 0.05, 1.0]);

        unsafe {
            self.context
                .ClearRenderTargetView(&self.render_target_view, &clear_color);

            if clear_depth {
                self.context.ClearDepthStencilView(
                    &self.depth_stencil_view,
                    D3D11_CLEAR_DEPTH,
                    1.0,
                    0,
                );
            }
        }
    }

    pub fn begin_frame(&self) {
        self.set_viewport();
        self.clear(Some([0.02, 0.02, 0.05, 1.0]), true);
    }

    pub fn end_frame(&self) {
        unsafe {
            let _ = self.swap_chain.Present(1, 0);
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) -> Result<(), String> {
        self.width = width;
        self.height = height;

        unsafe {
            self.swap_chain
                .ResizeBuffers(2, width, height, DXGI_FORMAT_R8G8B8A8_UNORM, 0)
                .map_err(|e| format!("ResizeBuffers failed: {:?}", e))?;

            let buffer = self
                .swap_chain
                .GetBuffer(0)
                .map_err(|e| format!("Failed to get buffer: {:?}", e))?;

            self.render_target_view = self
                .device
                .CreateRenderTargetView(&buffer, None)
                .map_err(|e| format!("Failed to create RTV: {:?}", e))?;

            let depth_format = windows::Win32::Graphics::Dxgi::DXGI_FORMAT_D24_UNORM_S8_UINT;

            let depth_desc = windows::Win32::Graphics::Direct3D11::D3D11_TEXTURE2D_DESC {
                Width: width,
                Height: height,
                MipLevels: 1,
                ArraySize: 1,
                Format: depth_format,
                SampleDesc: DXGI_SAMPLE_DESC {
                    Count: 1,
                    Quality: 0,
                },
                Usage: D3D11_USAGE_DEFAULT,
                BindFlags: D3D11_BIND_DEPTH_STENCIL,
                CPUAccessFlags: D3D11_CPU_ACCESS_NONE,
                MiscFlags: D3D11_RESOURCE_FLAGS(0),
            };

            let depth_texture = self
                .device
                .CreateTexture2D(&depth_desc, None)
                .map_err(|e| format!("Failed to create depth: {:?}", e))?;

            self.depth_stencil_view = self
                .device
                .CreateDepthStencilView(&depth_texture, None)
                .map_err(|e| format!("Failed to create DSV: {:?}", e))?;
        }

        Ok(())
    }

    pub fn get_projection_matrix(&self, fov: f32, near: f32, far: f32) -> nalgebra::Matrix4<f32> {
        let aspect = self.width as f32 / self.height as f32;
        *nalgebra::Perspective3::new(aspect, fov, near, far).as_matrix()
    }
}
