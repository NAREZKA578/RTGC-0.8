//! DirectX 11 Context - основной контекст для рендеринга

use std::sync::Arc;
use tracing::{error, info};
use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::Graphics::Direct3D11::{D3D11CreateDevice, ID3D11Device, ID3D11DeviceContext};
use windows::Win32::Graphics::Dxgi::{
    CreateDXGIFactory, IDXGIFactory, IDXGIOutput, DXGI_MODE_DESC, DXGI_MODE_ROTATION_UNSPECIFIED,
    DXGI_MODE_SCALING_UNSPECIFIED, DXGI_MODE_SYNC_INTERVAL, DXGI_OUTPUT_DESC, DXGI_RATIONAL,
    DXGI_SAMPLE_DESC, DXGI_SWAP_CHAIN_DESC, DXGI_SWAP_EFFECT, DXGI_USAGE_RENDER_TARGET_OUTPUT,
};

use super::device_dx11::Dx11Device;
use super::swapchain_dx11::Dx11SwapChain;

pub struct Dx11Context {
    pub device: ID3D11Device,
    pub context: ID3D11DeviceContext,
    pub swap_chain: Dx11SwapChain,
    pub hwnd: HWND,
    pub width: u32,
    pub height: u32,
}

impl Dx11Context {
    pub fn new(hwnd: isize, width: u32, height: u32) -> Result<Self, String> {
        info!(target: "dx11", "=== Dx11Context::new START ===");
        info!(target: "dx11", "HWND: {:?}, Size: {}x{}", hwnd, width, height);

        let hwnd = HWND(hwnd as *mut std::ffi::c_void);

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
                windows::Win32::Graphics::Direct3D11::D3D_DRIVER_TYPE_HARDWARE,
                None,
                0,
                None,
                windows::Win32::Graphics::Direct3D11::D3D11_SDK_VERSION,
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

            let swap_chain = Dx11SwapChain::new(&factory, &device, hwnd, width, height)?;

            info!(target: "dx11", "SwapChain created successfully!");
            info!(target: "dx11", "=== Dx11Context::new END ===");

            Ok(Self {
                device,
                context,
                swap_chain,
                hwnd,
                width,
                height,
            })
        }
    }

    pub fn begin_frame(&self) {
        unsafe {
            let color = [0.02, 0.02, 0.05, 1.0];
            self.context
                .ClearRenderTargetView(&self.swap_chain.render_target_view, &color);
            self.context.ClearDepthStencilView(
                &self.swap_chain.depth_stencil_view,
                windows::Win32::Graphics::Direct3D11::D3D11_CLEAR_DEPTH,
                1.0,
                0,
            );
        }
    }

    pub fn end_frame(&self) {
        unsafe {
            let _ = self.swap_chain.present(1);
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) -> Result<(), String> {
        self.width = width;
        self.height = height;
        self.swap_chain.resize(width, height)
    }

    pub fn set_viewport(&self, x: i32, y: i32, width: u32, height: u32) {
        use windows::Win32::Graphics::Direct3D11::D3D11_VIEWPORT;

        let viewport = D3D11_VIEWPORT {
            TopLeftX: x as f32,
            TopLeftY: y as f32,
            Width: width as f32,
            Height: height as f32,
            MinDepth: 0.0,
            MaxDepth: 1.0,
        };

        unsafe {
            self.context.RSSetViewports(&[viewport]);
        }
    }
}
