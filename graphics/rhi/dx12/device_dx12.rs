// DirectX 12 Backend - Device Implementation
// Implements IDevice trait for DX12

use crate::graphics::rhi::{
    types::*,
    device::*,
};
use std::sync::Arc;

#[cfg(target_os = "windows")]
use windows::{
    Win32::Foundation::*,
    Win32::Graphics::Direct3D12::*,
    Win32::Graphics::Dxgi::*,
    Win32::Graphics::Dxgi::Common::*,
    Win32::System::Threading::*,
};

/// DX12 Device implementation
pub struct Dx12Device {
    #[cfg(target_os = "windows")]
    device: ID3D12Device,
    
    #[cfg(target_os = "windows")]
    command_queue: ID3D12CommandQueue,
    
    #[cfg(target_os = "windows")]
    descriptor_srv_heap: Option<ID3D12DescriptorHeap>,
    
    #[cfg(target_os = "windows")]
    descriptor_rtv_heap: Option<ID3D12DescriptorHeap>,
    
    features: DeviceFeatures,
    limits: DeviceLimits,
    name: String,
    resource_counter: std::sync::atomic::AtomicU64,
}

unsafe impl Send for Dx12Device {}
unsafe impl Sync for Dx12Device {}

impl Dx12Device {
    /// Create a new DX12 device
    #[cfg(target_os = "windows")]
    pub fn new(enable_validation: bool) -> RhiResult<Self> {
        use windows::Win32::Graphics::Dxgi::*;
        
        // Enable debug layer if requested
        if enable_validation {
            unsafe {
                let debug_controller: ID3D12Debug = D3D12GetDebugInterface()
                    .map_err(|e| RhiError::InitializationFailed(format!("Failed to get debug interface: {:?}", e)))?;
                debug_controller.EnableDebugLayer();
            }
        }
        
        // Create DXGI factory
        let factory: IDXGIFactory4 = unsafe {
            CreateDXGIFactory1()
                .map_err(|e| RhiError::InitializationFailed(format!("Failed to create DXGI factory: {:?}", e)))?
        };
        
        // Find adapter
        let adapter = Self::find_adapter(&factory)?;
        
        // Get hardware feature levels
        let feature_levels = [
            D3D_FEATURE_LEVEL_12_2,
            D3D_FEATURE_LEVEL_12_1,
            D3D_FEATURE_LEVEL_12_0,
            D3D_FEATURE_LEVEL_11_1,
            D3D_FEATURE_LEVEL_11_0,
        ];
        
        // Create device
        let device: ID3D12Device = unsafe {
            D3D12CreateDevice(
                &adapter,
                D3D_FEATURE_LEVEL_12_0,
            )
            .map_err(|e| RhiError::InitializationFailed(format!("Failed to create D3D12 device: {:?}", e)))?
        };
        
        // Create command queue
        let queue_desc = D3D12_COMMAND_QUEUE_DESC {
            Type: D3D12_COMMAND_LIST_TYPE_DIRECT,
            Priority: 0,
            Flags: D3D12_COMMAND_QUEUE_FLAG_NONE,
            NodeMask: 0,
        };
        
        let command_queue: ID3D12CommandQueue = unsafe {
            device.CreateCommandQueue(&queue_desc)
                .map_err(|e| RhiError::InitializationFailed(format!("Failed to create command queue: {:?}", e)))?
        };
        
        // Create descriptor heaps
        let srv_heap_desc = D3D12_DESCRIPTOR_HEAP_DESC {
            Type: D3D12_DESCRIPTOR_HEAP_TYPE_CBV_SRV_UAV,
            NumDescriptors: 1024,
            Flags: D3D12_DESCRIPTOR_HEAP_FLAG_SHADER_VISIBLE,
            NodeMask: 0,
        };
        
        let rtv_heap_desc = D3D12_DESCRIPTOR_HEAP_DESC {
            Type: D3D12_DESCRIPTOR_HEAP_TYPE_RTV,
            NumDescriptors: 256,
            Flags: D3D12_DESCRIPTOR_HEAP_FLAG_NONE,
            NodeMask: 0,
        };
        
        let descriptor_srv_heap: ID3D12DescriptorHeap = unsafe {
            device.CreateDescriptorHeap(&srv_heap_desc)
                .map_err(|e| RhiError::InitializationFailed(format!("Failed to create SRV descriptor heap: {:?}", e)))?
        };
        
        let descriptor_rtv_heap: ID3D12DescriptorHeap = unsafe {
            device.CreateDescriptorHeap(&rtv_heap_desc)
                .map_err(|e| RhiError::InitializationFailed(format!("Failed to create RTV descriptor heap: {:?}", e)))?
        };
        
        // Query adapter info
        let adapter_desc = unsafe { adapter.GetDesc1() }
            .map_err(|e| RhiError::InitializationFailed(format!("Failed to get adapter desc: {:?}", e)))?;
        
        let name = String::from_utf16_lossy(&adapter_desc.Description)
            .trim_end_matches('\0')
            .to_string();
        
        Ok(Self {
            device,
            command_queue,
            descriptor_srv_heap: Some(descriptor_srv_heap),
            descriptor_rtv_heap: Some(descriptor_rtv_heap),
            features: Self::query_features(),
            limits: Self::query_limits(),
            name,
            resource_counter: std::sync::atomic::AtomicU64::new(0),
        })
    }
    
    #[cfg(not(target_os = "windows"))]
    pub fn new(_enable_validation: bool) -> RhiResult<Self> {
        Err(RhiError::Unsupported("DirectX 12 is only available on Windows".to_string()))
    }
    
    #[cfg(target_os = "windows")]
    fn find_adapter(factory: &IDXGIFactory4) -> RhiResult<IDXGIAdapter4> {
        let mut adapter_index = 0;
        
        loop {
            let adapter: IDXGIAdapter4 = unsafe { factory.EnumAdapterByGpuPreference(adapter_index, DXGI_GPU_PREFERENCE_HIGH_PERFORMANCE) }
                .map_err(|_| RhiError::InitializationFailed("No suitable GPU adapter found".to_string()))?;
            
            let desc = unsafe { adapter.GetDesc1() }.unwrap();
            
            // Skip software adapters
            if (desc.Flags & DXGI_ADAPTER_FLAG_SOFTWARE) == 0 {
                return Ok(adapter);
            }
            
            adapter_index += 1;
        }
    }
    
    fn query_features() -> DeviceFeatures {
        DeviceFeatures {
            anisotropic_filtering: true,
            bc_compression: true,
            compute_shaders: true,
            geometry_shaders: true,
            tessellation: true,
            conservative_rasterization: true,
            multi_draw_indirect: true,
            draw_indirect_first_instance: true,
            dual_source_blending: true,
            depth_bounds_test: false, // Not supported in DX12
            sample_rate_shading: true,
            texture_cube_map_array: true,
            texture_3d_as_2d_array: true,
            independent_blend: true,
            logic_op: true,
            occlusion_query: true,
            timestamp_query: true,
            pipeline_statistics_query: true,
            stream_output: true,
            variable_rate_shading: true,
            mesh_shaders: true,
            ray_tracing: true,
            sampler_lod_bias: true,
            border_color_clamp: true,
        }
    }
    
    fn query_limits() -> DeviceLimits {
        DeviceLimits {
            max_texture_dimension_1d: 16384,
            max_texture_dimension_2d: 16384,
            max_texture_dimension_3d: 2048,
            max_texture_array_layers: 2048,
            max_buffer_size: 128 * 1024 * 1024 * 1024, // 128 GB
            max_vertex_input_attributes: 32,
            max_vertex_input_bindings: 32,
            max_vertex_input_attribute_offset: 4095,
            max_vertex_input_binding_stride: 4095,
            max_vertex_output_components: 128,
            max_fragment_input_components: 128,
            max_fragment_output_attachments: 8,
            max_compute_work_group_count: [65535, 65535, 65535],
            max_compute_work_group_invocations: 1024,
            max_compute_shared_memory_size: 32768,
            max_uniform_buffer_range: 65536,
            max_storage_buffer_range: 128 * 1024 * 1024,
            max_sampler_anisotropy: 16.0,
            min_texel_buffer_offset_alignment: 1,
            min_uniform_buffer_offset_alignment: 256,
            min_storage_buffer_offset_alignment: 1,
            max_descriptor_set_samplers: 2048,
            max_descriptor_set_uniform_buffers: 256,
            max_descriptor_set_storage_buffers: 256,
            max_descriptor_set_textures: 256,
            max_descriptor_set_storage_images: 256,
            max_per_stage_descriptor_samplers: 2048,
            max_per_stage_descriptor_uniform_buffers: 64,
            max_per_stage_descriptor_storage_buffers: 64,
            max_per_stage_descriptor_textures: 128,
            max_per_stage_descriptor_storage_images: 64,
        }
    }
    
    #[cfg(target_os = "windows")]
    pub fn device(&self) -> &ID3D12Device {
        &self.device
    }
    
    #[cfg(target_os = "windows")]
    pub fn command_queue(&self) -> &ID3D12CommandQueue {
        &self.command_queue
    }
    
    #[cfg(target_os = "windows")]
    pub fn generate_handle(&self) -> ResourceHandle {
        let id = self.resource_counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        ResourceHandle(id)
    }
}

impl IDevice for Dx12Device {
    fn get_device_name(&self) -> &str {
        &self.name
    }
    
    fn get_features(&self) -> DeviceFeatures {
        self.features.clone()
    }
    
    fn get_limits(&self) -> DeviceLimits {
        self.limits.clone()
    }
    
    #[cfg(target_os = "windows")]
    fn create_buffer(&self, desc: &BufferDescription) -> RhiResult<ResourceHandle> {
        use windows::Win32::Graphics::Direct3D12::*;
        
        let handle = self.generate_handle();
        let buffer = Dx12Buffer::new(&self.device, desc, handle)?;
        
        // Store buffer in resource manager (TODO: implement proper resource tracking)
        Ok(handle)
    }
    
    #[cfg(not(target_os = "windows"))]
    fn create_buffer(&self, _desc: &BufferDescription) -> RhiResult<ResourceHandle> {
        Err(RhiError::Unsupported("DX12 is only available on Windows".to_string()))
    }
    
    #[cfg(target_os = "windows")]
    fn create_texture(&self, desc: &TextureDescription) -> RhiResult<ResourceHandle> {
        use windows::Win32::Graphics::Direct3D12::*;
        
        let handle = self.generate_handle();
        let texture = Dx12Texture::new(&self.device, desc, handle)?;
        
        Ok(handle)
    }
    
    #[cfg(not(target_os = "windows"))]
    fn create_texture(&self, _desc: &TextureDescription) -> RhiResult<ResourceHandle> {
        Err(RhiError::Unsupported("DX12 is only available on Windows".to_string()))
    }
    
    #[cfg(target_os = "windows")]
    fn create_texture_view(
        &self,
        texture: ResourceHandle,
        desc: &TextureViewDescription,
    ) -> RhiResult<ResourceHandle> {
        let handle = self.generate_handle();
        // TODO: Implement texture view creation
        Ok(handle)
    }
    
    #[cfg(not(target_os = "windows"))]
    fn create_texture_view(
        &self,
        _texture: ResourceHandle,
        _desc: &TextureViewDescription,
    ) -> RhiResult<ResourceHandle> {
        Err(RhiError::Unsupported("DX12 is only available on Windows".to_string()))
    }
    
    #[cfg(target_os = "windows")]
    fn create_sampler(&self, desc: &SamplerDescription) -> RhiResult<ResourceHandle> {
        use windows::Win32::Graphics::Direct3D12::*;
        
        let handle = self.generate_handle();
        // TODO: Implement sampler creation using descriptor heap
        Ok(handle)
    }
    
    #[cfg(not(target_os = "windows"))]
    fn create_sampler(&self, _desc: &SamplerDescription) -> RhiResult<ResourceHandle> {
        Err(RhiError::Unsupported("DX12 is only available on Windows".to_string()))
    }
    
    #[cfg(target_os = "windows")]
    fn create_shader(&self, desc: &ShaderDescription) -> RhiResult<ResourceHandle> {
        use windows::Win32::Graphics::Direct3D12::*;
        
        let handle = self.generate_handle();
        let shader = Dx12Shader::new(&self.device, desc, handle)?;
        
        Ok(handle)
    }
    
    #[cfg(not(target_os = "windows"))]
    fn create_shader(&self, _desc: &ShaderDescription) -> RhiResult<ResourceHandle> {
        Err(RhiError::Unsupported("DX12 is only available on Windows".to_string()))
    }
    
    #[cfg(target_os = "windows")]
    fn create_pipeline_state(&self, desc: &PipelineStateObject) -> RhiResult<ResourceHandle> {
        use windows::Win32::Graphics::Direct3D12::*;
        
        let handle = self.generate_handle();
        let pso = Dx12PipelineState::new(&self.device, desc, handle)?;
        
        Ok(handle)
    }
    
    #[cfg(not(target_os = "windows"))]
    fn create_pipeline_state(&self, _desc: &PipelineStateObject) -> RhiResult<ResourceHandle> {
        Err(RhiError::Unsupported("DX12 is only available on Windows".to_string()))
    }
    
    #[cfg(target_os = "windows")]
    fn create_descriptor_heap(&self, desc: &DescriptorHeapDescription) -> RhiResult<ResourceHandle> {
        use windows::Win32::Graphics::Direct3D12::*;
        
        let handle = self.generate_handle();
        let heap = Dx12DescriptorHeap::new(&self.device, desc, handle)?;
        
        Ok(handle)
    }
    
    #[cfg(not(target_os = "windows"))]
    fn create_descriptor_heap(&self, _desc: &DescriptorHeapDescription) -> RhiResult<ResourceHandle> {
        Err(RhiError::Unsupported("DX12 is only available on Windows".to_string()))
    }
    
    #[cfg(target_os = "windows")]
    fn create_command_list(&self, cmd_type: CommandListType) -> RhiResult<Arc<dyn ICommandList>> {
        use windows::Win32::Graphics::Direct3D12::*;
        
        let cmd_list = Dx12CommandList::new(
            &self.device,
            cmd_type,
            &self.descriptor_srv_heap,
            &self.descriptor_rtv_heap,
        )?;
        
        Ok(Arc::new(cmd_list))
    }
    
    #[cfg(not(target_os = "windows"))]
    fn create_command_list(&self, _cmd_type: CommandListType) -> RhiResult<Arc<dyn ICommandList>> {
        Err(RhiError::Unsupported("DX12 is only available on Windows".to_string()))
    }
    
    #[cfg(target_os = "windows")]
    fn create_command_queue(&self, cmd_type: CommandListType) -> RhiResult<Arc<dyn ICommandQueue>> {
        use windows::Win32::Graphics::Direct3D12::*;
        
        let queue_desc = D3D12_COMMAND_QUEUE_DESC {
            Type: match cmd_type {
                CommandListType::Direct => D3D12_COMMAND_LIST_TYPE_DIRECT,
                CommandListType::Compute => D3D12_COMMAND_LIST_TYPE_COMPUTE,
                CommandListType::Copy => D3D12_COMMAND_LIST_TYPE_COPY,
            },
            Priority: 0,
            Flags: D3D12_COMMAND_QUEUE_FLAG_NONE,
            NodeMask: 0,
        };
        
        let queue: ID3D12CommandQueue = unsafe {
            self.device.CreateCommandQueue(&queue_desc)
                .map_err(|e| RhiError::ResourceCreationFailed(format!("Failed to create command queue: {:?}", e)))?
        };
        
        let cmd_queue = Dx12CommandQueue::new(queue);
        
        Ok(Arc::new(cmd_queue))
    }
    
    #[cfg(not(target_os = "windows"))]
    fn create_command_queue(&self, _cmd_type: CommandListType) -> RhiResult<Arc<dyn ICommandQueue>> {
        Err(RhiError::Unsupported("DX12 is only available on Windows".to_string()))
    }
    
    #[cfg(target_os = "windows")]
    fn create_fence(&self, initial_value: u64) -> RhiResult<Arc<dyn IFence>> {
        use windows::Win32::Graphics::Direct3D12::*;
        
        let fence: ID3D12Fence = unsafe {
            self.device.CreateFence(initial_value, D3D12_FENCE_FLAG_NONE)
                .map_err(|e| RhiError::ResourceCreationFailed(format!("Failed to create fence: {:?}", e)))?
        };
        
        let dx_fence = Dx12Fence::new(fence, initial_value);
        
        Ok(Arc::new(dx_fence))
    }
    
    #[cfg(not(target_os = "windows"))]
    fn create_fence(&self, _initial_value: u64) -> RhiResult<Arc<dyn IFence>> {
        Err(RhiError::Unsupported("DX12 is only available on Windows".to_string()))
    }
    
    #[cfg(target_os = "windows")]
    fn create_semaphore(&self) -> RhiResult<Arc<dyn ISemaphore>> {
        // DX12 doesn't have semaphores like Vulkan, we use fences instead
        self.create_fence(0).map(|f| Arc::new(Dx12Semaphore::from_fence(f)))
    }
    
    #[cfg(not(target_os = "windows"))]
    fn create_semaphore(&self) -> RhiResult<Arc<dyn ISemaphore>> {
        Err(RhiError::Unsupported("DX12 is only available on Windows".to_string()))
    }
    
    #[cfg(target_os = "windows")]
    fn create_swap_chain(
        &self,
        window_handle: *mut std::ffi::c_void,
        width: u32,
        height: u32,
        format: TextureFormat,
        vsync: bool,
    ) -> RhiResult<Arc<dyn ISwapChain>> {
        use windows::Win32::Graphics::Dxgi::*;
        
        let swapchain = Dx12SwapChain::new(
            &self.device,
            &self.command_queue,
            window_handle,
            width,
            height,
            format,
            vsync,
        )?;
        
        Ok(Arc::new(swapchain))
    }
    
    #[cfg(not(target_os = "windows"))]
    fn create_swap_chain(
        &self,
        _window_handle: *mut std::ffi::c_void,
        _width: u32,
        _height: u32,
        _format: TextureFormat,
        _vsync: bool,
    ) -> RhiResult<Arc<dyn ISwapChain>> {
        Err(RhiError::Unsupported("DX12 is only available on Windows".to_string()))
    }
    
    #[cfg(target_os = "windows")]
    fn update_buffer(
        &self,
        buffer: ResourceHandle,
        offset: u64,
        data: &[u8],
    ) -> RhiResult<()> {
        // TODO: Implement buffer update via upload heap
        Ok(())
    }
    
    #[cfg(not(target_os = "windows"))]
    fn update_buffer(
        &self,
        _buffer: ResourceHandle,
        _offset: u64,
        _data: &[u8],
    ) -> RhiResult<()> {
        Err(RhiError::Unsupported("DX12 is only available on Windows".to_string()))
    }
    
    #[cfg(target_os = "windows")]
    fn map_buffer(&self, _buffer: ResourceHandle) -> RhiResult<*mut u8> {
        // TODO: Implement buffer mapping
        Err(RhiError::Unsupported("Buffer mapping requires upload heap".to_string()))
    }
    
    #[cfg(not(target_os = "windows"))]
    fn map_buffer(&self, _buffer: ResourceHandle) -> RhiResult<*mut u8> {
        Err(RhiError::Unsupported("DX12 is only available on Windows".to_string()))
    }
    
    #[cfg(target_os = "windows")]
    fn unmap_buffer(&self, _buffer: ResourceHandle) {
        // TODO: Implement buffer unmapping
    }
    
    #[cfg(not(target_os = "windows"))]
    fn unmap_buffer(&self, _buffer: ResourceHandle) {}
    
    #[cfg(target_os = "windows")]
    fn read_back_texture(&self, _texture: ResourceHandle) -> RhiResult<Vec<u8>> {
        Err(RhiError::Unsupported("Texture readback not yet implemented".to_string()))
    }
    
    #[cfg(not(target_os = "windows"))]
    fn read_back_texture(&self, _texture: ResourceHandle) -> RhiResult<Vec<u8>> {
        Err(RhiError::Unsupported("DX12 is only available on Windows".to_string()))
    }
    
    #[cfg(target_os = "windows")]
    fn destroy_resource(&self, handle: ResourceHandle) {
        // TODO: Implement proper resource destruction with reference counting
    }
    
    #[cfg(not(target_os = "windows"))]
    fn destroy_resource(&self, _handle: ResourceHandle) {}
    
    #[cfg(target_os = "windows")]
    fn wait_idle(&self) -> RhiResult<()> {
        use windows::Win32::System::Threading::*;
        
        // Create a fence and wait for it
        let fence = self.create_fence(0)?;
        unsafe {
            self.command_queue.Signal(&fence.as_any().downcast_ref::<Dx12Fence>().unwrap().fence, 1)
                .map_err(|e| RhiError::DeviceLost)?;
        }
        
        fence.set_event_on_completion(1)?;
        
        Ok(())
    }
    
    #[cfg(not(target_os = "windows"))]
    fn wait_idle(&self) -> RhiResult<()> {
        Ok(())
    }
    
    #[cfg(target_os = "windows")]
    fn get_memory_stats(&self) -> MemoryStats {
        // TODO: Query actual memory stats from DXGI
        MemoryStats::default()
    }
    
    #[cfg(not(target_os = "windows"))]
    fn get_memory_stats(&self) -> MemoryStats {
        MemoryStats::default()
    }
}

/// Factory function to create DX12 device
pub fn create_dx12_device(enable_validation: bool) -> RhiResult<Box<dyn IDevice>> {
    let device = Dx12Device::new(enable_validation)?;
    Ok(Box::new(device))
}
