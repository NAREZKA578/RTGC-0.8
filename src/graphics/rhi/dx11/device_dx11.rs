//! DirectX 11 Device - RHI implementation

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tracing::{error, info};

use crate::graphics::rhi::device::*;
use crate::graphics::rhi::types::*;

pub struct Dx11Device {
    #[cfg(target_os = "windows")]
    device: windows::Win32::Graphics::Direct3D11::ID3D11Device,
    #[cfg(target_os = "windows")]
    context: windows::Win32::Graphics::Direct3D11::ID3D11DeviceContext,
    #[cfg(target_os = "windows")]
    factory: windows::Win32::Graphics::Dxgi::IDXGIFactory,
    name: String,
    resource_counter: AtomicU64,
}

#[cfg(target_os = "windows")]
unsafe impl Send for Dx11Device {}

#[cfg(target_os = "windows")]
unsafe impl Sync for Dx11Device {}

impl Dx11Device {
    pub fn new(_debug: bool, _validation: bool) -> RhiResult<Self> {
        info!(target: "dx11", "=== Dx11Device::new START ===");

        #[cfg(target_os = "windows")]
        {
            use windows::Win32::Graphics::Direct3D::{
                D3D_DRIVER_TYPE_HARDWARE, D3D_FEATURE_LEVEL_11_0,
            };
            use windows::Win32::Graphics::Direct3D11::{
                D3D11CreateDevice, D3D11_CREATE_DEVICE_BGRA_SUPPORT, D3D11_CREATE_DEVICE_FLAG,
                D3D11_SDK_VERSION,
            };
            use windows::Win32::Graphics::Dxgi::CreateDXGIFactory1;

            let factory: windows::Win32::Graphics::Dxgi::IDXGIFactory = unsafe {
                CreateDXGIFactory1().map_err(|e| {
                    error!(target: "dx11", "Failed to create DXGI factory: {:?}", e);
                    RhiError::InitializationFailed(format!("DXGI factory: {:?}", e))
                })?
            };

            let mut device: Option<windows::Win32::Graphics::Direct3D11::ID3D11Device> = None;
            let mut context: Option<windows::Win32::Graphics::Direct3D11::ID3D11DeviceContext> =
                None;
            let mut feature_level = D3D_FEATURE_LEVEL_11_0;

            let hr = unsafe {
                D3D11CreateDevice(
                    None,
                    D3D_DRIVER_TYPE_HARDWARE,
                    None,
                    D3D11_CREATE_DEVICE_FLAG(D3D11_CREATE_DEVICE_BGRA_SUPPORT.0),
                    Some(&[D3D_FEATURE_LEVEL_11_0]),
                    D3D11_SDK_VERSION,
                    Some(&mut device),
                    Some(&mut feature_level),
                    Some(&mut context),
                )
            };

            if hr.is_err() {
                error!(target: "dx11", "D3D11CreateDevice failed: {:?}", hr);
                return Err(RhiError::InitializationFailed(
                    "D3D11CreateDevice".to_string(),
                ));
            }

            info!(target: "dx11", "DX11 Device created: {:?}", feature_level);

            Ok(Self {
                device: device.unwrap(),
                context: context.unwrap(),
                factory,
                name: "DirectX 11".to_string(),
                resource_counter: AtomicU64::new(1),
            })
        }

        #[cfg(not(target_os = "windows"))]
        {
            Ok(Self {
                name: "DirectX 11 (stub)".to_string(),
                resource_counter: AtomicU64::new(1),
            })
        }
    }

    #[cfg(target_os = "windows")]
    pub fn get_device(&self) -> &windows::Win32::Graphics::Direct3D11::ID3D11Device {
        &self.device
    }

    #[cfg(target_os = "windows")]
    pub fn get_context(&self) -> &windows::Win32::Graphics::Direct3D11::ID3D11DeviceContext {
        &self.context
    }
}

impl IDevice for Dx11Device {
    fn get_device_name(&self) -> &str {
        &self.name
    }

    fn get_features(&self) -> DeviceFeatures {
        DeviceFeatures::default()
    }

    fn get_limits(&self) -> DeviceLimits {
        DeviceLimits::default()
    }

    fn create_buffer(&self, _desc: &BufferDescription) -> RhiResult<ResourceHandle> {
        Ok(ResourceHandle(
            self.resource_counter.fetch_add(1, Ordering::Relaxed),
        ))
    }

    fn create_texture(&self, _desc: &TextureDescription) -> RhiResult<ResourceHandle> {
        Ok(ResourceHandle(
            self.resource_counter.fetch_add(1, Ordering::Relaxed),
        ))
    }

    fn create_texture_view(
        &self,
        _texture: ResourceHandle,
        _desc: &TextureViewDescription,
    ) -> RhiResult<ResourceHandle> {
        Ok(ResourceHandle(
            self.resource_counter.fetch_add(1, Ordering::Relaxed),
        ))
    }

    fn create_sampler(&self, _desc: &SamplerDescription) -> RhiResult<ResourceHandle> {
        Ok(ResourceHandle(
            self.resource_counter.fetch_add(1, Ordering::Relaxed),
        ))
    }

    fn create_shader(&self, _desc: &ShaderDescription) -> RhiResult<ResourceHandle> {
        Ok(ResourceHandle(
            self.resource_counter.fetch_add(1, Ordering::Relaxed),
        ))
    }

    fn create_pipeline_state(&self, _desc: &PipelineStateObject) -> RhiResult<ResourceHandle> {
        Ok(ResourceHandle(
            self.resource_counter.fetch_add(1, Ordering::Relaxed),
        ))
    }

    fn create_descriptor_heap(
        &self,
        _desc: &DescriptorHeapDescription,
    ) -> RhiResult<ResourceHandle> {
        Ok(ResourceHandle(
            self.resource_counter.fetch_add(1, Ordering::Relaxed),
        ))
    }

    fn create_command_list(&self, _cmd_type: CommandListType) -> RhiResult<Arc<dyn ICommandList>> {
        Err(RhiError::Unsupported("DX11 command lists".to_string()))
    }

    fn create_command_queue(
        &self,
        _cmd_type: CommandListType,
    ) -> RhiResult<Arc<dyn ICommandQueue>> {
        Err(RhiError::Unsupported("DX11 command queues".to_string()))
    }

    fn create_fence(&self, _initial_value: u64) -> RhiResult<Arc<dyn IFence>> {
        Err(RhiError::Unsupported("DX11 fence".to_string()))
    }

    fn create_semaphore(&self) -> RhiResult<Arc<dyn ISemaphore>> {
        Err(RhiError::Unsupported("DX11 semaphore".to_string()))
    }

    fn create_swap_chain(
        &self,
        _window_handle: *mut std::ffi::c_void,
        _width: u32,
        _height: u32,
        _format: TextureFormat,
        _vsync: bool,
    ) -> RhiResult<Arc<dyn ISwapChain>> {
        Err(RhiError::Unsupported("DX11 swap chain via RHI".to_string()))
    }

    fn update_buffer(&self, _buffer: ResourceHandle, _offset: u64, _data: &[u8]) -> RhiResult<()> {
        Ok(())
    }

    fn map_buffer(&self, _buffer: ResourceHandle) -> RhiResult<*mut u8> {
        Err(RhiError::Unsupported("DX11 map".to_string()))
    }

    fn unmap_buffer(&self, _buffer: ResourceHandle) {}

    fn read_back_texture(&self, _texture: ResourceHandle) -> RhiResult<Vec<u8>> {
        Err(RhiError::Unsupported("DX11 readback".to_string()))
    }

    fn destroy_resource(&self, _handle: ResourceHandle) {}

    fn wait_idle(&self) -> RhiResult<()> {
        Ok(())
    }

    fn get_memory_stats(&self) -> MemoryStats {
        MemoryStats::default()
    }
}
