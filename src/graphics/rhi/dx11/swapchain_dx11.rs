//! DirectX 11 SwapChain

use tracing::info;
use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Direct3D11::{
    ID3D11DepthStencilView, ID3D11Device, ID3D11RenderTargetView,
};
use windows::Win32::Graphics::Dxgi::{
    ID3D10Multithread, IDXGIFactory, IDXGISwapChain, DXGI_FORMAT_R8G8B8A8_UNORM, DXGI_MODE_DESC,
    DXGI_MODE_ROTATION_UNSPECIFIED, DXGI_MODE_SCALING_UNSPECIFIED, DXGI_RATIONAL, DXGI_SAMPLE_DESC,
    DXGI_SWAP_CHAIN_DESC, DXGI_SWAP_EFFECT_DISCARD, DXGI_USAGE_RENDER_TARGET_OUTPUT,
};

pub struct Dx11SwapChain {
    pub swap_chain: IDXGISwapChain,
    pub render_target_view: ID3D11RenderTargetView,
    pub depth_stencil_view: ID3D11DepthStencilView,
    pub device: ID3D11Device,
    pub width: u32,
    pub height: u32,
}

impl Dx11SwapChain {
    pub fn new(
        factory: &IDXGIFactory,
        device: &ID3D11Device,
        hwnd: isize,
        width: u32,
        height: u32,
    ) -> Result<Self, String> {
        info!(target: "dx11", "=== Dx11SwapChain::new START ===");

        let hwnd = HWND(hwnd as *mut std::ffi::c_void);

        unsafe {
            let swap_desc = DXGI_SWAP_CHAIN_DESC {
                BufferDesc: DXGI_MODE_DESC {
                    Width: width,
                    Height: height,
                    RefreshRate: DXGI_RATIONAL {
                        Numerator: 60,
                        Denominator: 1,
                    },
                    Format: DXGI_FORMAT_R8G8B8A8_UNORM,
                    ScanlineOrdering: DXGI_MODE_ROTATION_UNSPECIFIED,
                    Scaling: DXGI_MODE_SCALING_UNSPECIFIED,
                },
                SampleDesc: DXGI_SAMPLE_DESC {
                    Count: 1,
                    Quality: 0,
                },
                BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
                BufferCount: 2,
                OutputWindow: hwnd,
                Windowed: windows::Win32::Foundation::TRUE,
                SwapEffect: DXGI_SWAP_EFFECT_DISCARD,
                Flags: windows::Win32::Graphics::Dxgi::DXGI_SWAP_CHAIN_FLAG(0),
            };

            let swap_chain = factory
                .CreateSwapChain(device, &swap_desc)
                .map_err(|e| format!("Failed to create SwapChain: {:?}", e))?;

            info!(target: "dx11", "SwapChain created");

            // Create render target view
            let buffer = swap_chain
                .GetBuffer(0)
                .map_err(|e| format!("Failed to get buffer: {:?}", e))?;

            let render_target_view = device
                .CreateRenderTargetView(&buffer, None)
                .map_err(|e| format!("Failed to create RTV: {:?}", e))?;

            info!(target: "dx11", "RenderTargetView created");

            // Create depth stencil
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
                Usage: windows::Win32::Graphics::Direct3D11::D3D11_USAGE_DEFAULT,
                BindFlags: windows::Win32::Graphics::Direct3D11::D3D11_BIND_DEPTH_STENCIL,
                CPUAccessFlags: windows::Win32::Graphics::Direct3D11::D3D11_CPU_ACCESS_FLAG(0),
                MiscFlags: windows::Win32::Graphics::Direct3D11::D3D11_RESOURCE_FLAGS(0),
            };

            let depth_texture = device
                .CreateTexture2D(&depth_desc, None)
                .map_err(|e| format!("Failed to create depth texture: {:?}", e))?;

            let depth_stencil_view = device
                .CreateDepthStencilView(&depth_texture, None)
                .map_err(|e| format!("Failed to create DSV: {:?}", e))?;

            info!(target: "dx11", "DepthStencilView created");
            info!(target: "dx11", "=== Dx11SwapChain::new END ===");

            Ok(Self {
                swap_chain,
                render_target_view,
                depth_stencil_view,
                device: device.clone(),
                width,
                height,
            })
        }
    }

    pub fn present(&self, vsync: bool) -> Result<(), String> {
        unsafe {
            let interval = if vsync { 1 } else { 0 };
            self.swap_chain
                .Present(interval, 0)
                .map_err(|e| format!("Present failed: {:?}", e))
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
                Usage: windows::Win32::Graphics::Direct3D11::D3D11_USAGE_DEFAULT,
                BindFlags: windows::Win32::Graphics::Direct3D11::D3D11_BIND_DEPTH_STENCIL,
                CPUAccessFlags: windows::Win32::Graphics::Direct3D11::D3D11_CPU_ACCESS_FLAG(0),
                MiscFlags: windows::Win32::Graphics::Direct3D11::D3D11_RESOURCE_FLAGS(0),
            };

            let depth_texture = self
                .device
                .CreateTexture2D(&depth_desc, None)
                .map_err(|e| format!("Failed to create depth texture: {:?}", e))?;

            self.depth_stencil_view = self
                .device
                .CreateDepthStencilView(&depth_texture, None)
                .map_err(|e| format!("Failed to create DSV: {:?}", e))?;
        }

        Ok(())
    }

    pub fn clear(&self, color: [f32; 4], clear_depth: bool) {
        unsafe {
            self.context
                .ClearRenderTargetView(&self.render_target_view, &color);
            if clear_depth {
                self.context.ClearDepthStencilView(
                    &self.depth_stencil_view,
                    windows::Win32::Graphics::Direct3D11::D3D11_CLEAR_DEPTH,
                    1.0,
                    0,
                );
            }
        }
    }

    pub fn set_viewport(&self) {
        use windows::Win32::Graphics::Direct3D11::D3D11_VIEWPORT;

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

    pub fn begin_frame(&self) {
        self.set_viewport();
        self.clear([0.02, 0.02, 0.05, 1.0], true);
    }

    pub fn end_frame(&self) {
        let _ = self.present(true);
    }
}
