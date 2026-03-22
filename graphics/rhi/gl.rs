//! OpenGL RHI Backend для RTGC-0.7 - Минимальная заглушка

use super::device::{
    IDevice, ICommandList, ICommandQueue, IFence, ISemaphore, ISwapChain,
    TextureViewDescription, DescriptorHeapDescription,
    RenderPassDescription, DeviceFeatures, DeviceLimits, MemoryStats, IndexFormat,
};
use super::types::{
    ResourceHandle, BufferDescription, TextureDescription,
    TextureFormat, SamplerDescription, ShaderDescription,
    PipelineStateObject, CommandListType, Viewport, ScissorRect, PrimitiveTopology,
    RhiResult, RhiError, ShaderStage,
};
use glow::{Context, HasContext};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

// Заглушки для внутренних структур
pub struct GlCommandQueueInternal;
pub struct GlFenceInternal { value: AtomicU64 }
pub struct GlSemaphoreInternal;
pub struct GlSwapChainInternal { width: u32, height: u32 }

unsafe impl Send for GlCommandQueueInternal {}
unsafe impl Sync for GlCommandQueueInternal {}
unsafe impl Send for GlFenceInternal {}
unsafe impl Sync for GlFenceInternal {}
unsafe impl Send for GlSemaphoreInternal {}
unsafe impl Sync for GlSemaphoreInternal {}
unsafe impl Send for GlSwapChainInternal {}
unsafe impl Sync for GlSwapChainInternal {}

pub struct GlDevice {
    pub context: Arc<Context>,
    resource_counter: AtomicU64,
    device_name: String,
    features: DeviceFeatures,
    limits: DeviceLimits,
}

unsafe impl Send for GlDevice {}
unsafe impl Sync for GlDevice {}

impl GlDevice {
    pub fn new(context: Arc<Context>) -> Self {
        let device_name = unsafe { context.get_parameter_string(glow::RENDERER) };
        Self {
            context,
            resource_counter: AtomicU64::new(1),
            device_name,
            features: Self::query_features(),
            limits: Self::query_limits(),
        }
    }

    fn generate_handle(&self) -> ResourceHandle {
        ResourceHandle(self.resource_counter.fetch_add(1, Ordering::Relaxed))
    }

    fn query_features() -> DeviceFeatures {
        DeviceFeatures {
            anisotropic_filtering: true, bc_compression: false, compute_shaders: true,
            geometry_shaders: true, tessellation: true, conservative_rasterization: false,
            multi_draw_indirect: false, draw_indirect_first_instance: false,
            dual_source_blending: true, depth_bounds_test: false, sample_rate_shading: true,
            texture_cube_map_array: true, texture_3d_as_2d_array: true, independent_blend: true,
            logic_op: true, occlusion_query: true, timestamp_query: true,
            pipeline_statistics_query: false, stream_output: true, variable_rate_shading: false,
            mesh_shaders: false, ray_tracing: false, sampler_lod_bias: true, border_color_clamp: true,
        }
    }

    fn query_limits() -> DeviceLimits {
        DeviceLimits {
            max_texture_dimension_1d: 16384, max_texture_dimension_2d: 16384,
            max_texture_dimension_3d: 2048, max_texture_array_layers: 2048,
            max_buffer_size: 256 * 1024 * 1024, max_vertex_input_attributes: 16,
            max_vertex_input_bindings: 16, max_vertex_input_attribute_offset: 2047,
            max_vertex_input_binding_stride: 2048, max_vertex_output_components: 128,
            max_fragment_input_components: 128, max_fragment_output_attachments: 8,
            max_compute_work_group_count: [65535, 65535, 65535],
            max_compute_work_group_invocations: 1024, max_compute_shared_memory_size: 32768,
            max_uniform_buffer_range: 65536, max_storage_buffer_range: 128 * 1024 * 1024,
            max_sampler_anisotropy: 16.0, min_texel_buffer_offset_alignment: 1,
            min_uniform_buffer_offset_alignment: 256, min_storage_buffer_offset_alignment: 1,
            max_descriptor_set_samplers: 128, max_descriptor_set_uniform_buffers: 84,
            max_descriptor_set_storage_buffers: 96, max_descriptor_set_textures: 128,
            max_descriptor_set_storage_images: 64, max_per_stage_descriptor_samplers: 32,
            max_per_stage_descriptor_uniform_buffers: 14, max_per_stage_descriptor_storage_buffers: 16,
            max_per_stage_descriptor_textures: 48, max_per_stage_descriptor_storage_images: 16,
        }
    }
}

impl IDevice for GlDevice {
    fn get_device_name(&self) -> &str { &self.device_name }
    fn get_features(&self) -> DeviceFeatures { self.features.clone() }
    fn get_limits(&self) -> DeviceLimits { self.limits.clone() }
    fn create_buffer(&self, _desc: &BufferDescription) -> RhiResult<ResourceHandle> { Ok(self.generate_handle()) }
    fn create_texture(&self, _desc: &TextureDescription) -> RhiResult<ResourceHandle> { Ok(self.generate_handle()) }
    fn create_texture_view(&self, texture: ResourceHandle, _desc: &TextureViewDescription) -> RhiResult<ResourceHandle> { Ok(texture) }
    fn create_sampler(&self, _desc: &SamplerDescription) -> RhiResult<ResourceHandle> { Ok(self.generate_handle()) }
    fn create_shader(&self, _desc: &ShaderDescription) -> RhiResult<ResourceHandle> { Ok(self.generate_handle()) }
    fn create_pipeline_state(&self, _desc: &PipelineStateObject) -> RhiResult<ResourceHandle> { Ok(self.generate_handle()) }
    fn create_descriptor_heap(&self, _desc: &DescriptorHeapDescription) -> RhiResult<ResourceHandle> { Ok(ResourceHandle(0)) }
    fn create_command_list(&self, _cmd_type: CommandListType) -> RhiResult<Arc<dyn ICommandList>> { Ok(Arc::new(GlCommandList)) }
    fn create_command_queue(&self, _cmd_type: CommandListType) -> RhiResult<Arc<dyn ICommandQueue>> { Ok(Arc::new(GlCommandQueueInternal)) }
    fn create_fence(&self, initial_value: u64) -> RhiResult<Arc<dyn IFence>> { Ok(Arc::new(GlFenceInternal { value: AtomicU64::new(initial_value) })) }
    fn create_semaphore(&self) -> RhiResult<Arc<dyn ISemaphore>> { Ok(Arc::new(GlSemaphoreInternal)) }
    fn create_swap_chain(&self, _window_handle: *mut std::ffi::c_void, width: u32, height: u32, _format: TextureFormat, _vsync: bool) -> RhiResult<Arc<dyn ISwapChain>> { Ok(Arc::new(GlSwapChainInternal { width, height })) }
    fn update_buffer(&self, _buffer: ResourceHandle, _offset: u64, _data: &[u8]) -> RhiResult<()> { Ok(()) }
    fn map_buffer(&self, _buffer: ResourceHandle) -> RhiResult<*mut u8> { Ok(std::ptr::null_mut()) }
    fn unmap_buffer(&self, _buffer: ResourceHandle) {}
    fn read_back_texture(&self, _texture: ResourceHandle) -> RhiResult<Vec<u8>> { Ok(Vec::new()) }
    fn destroy_resource(&self, _handle: ResourceHandle) {}
    fn wait_idle(&self) -> RhiResult<()> { Ok(()) }
    fn get_memory_stats(&self) -> MemoryStats {
        MemoryStats {
            total_gpu_memory: u64::MAX,
            used_gpu_memory: 0,
            total_upload_memory: u64::MAX,
            used_upload_memory: 0,
            total_download_memory: u64::MAX,
            used_download_memory: 0,
        }
    }
}

pub struct GlCommandList;

impl ICommandList for GlCommandList {
    fn reset(&mut self) -> RhiResult<()> { Ok(()) }
    fn close(&mut self) -> RhiResult<()> { Ok(()) }
    fn begin_render_pass(&mut self, _desc: &RenderPassDescription) {}
    fn end_render_pass(&mut self) {}
    fn set_pipeline_state(&mut self, _pso: ResourceHandle) {}
    fn set_primitive_topology(&mut self, _topology: PrimitiveTopology) {}
    fn set_viewport(&mut self, _viewport: &Viewport) {}
    fn set_scissor_rect(&mut self, _scissor: &ScissorRect) {}
    fn set_blend_constants(&mut self, _constants: [f32; 4]) {}
    fn set_stencil_reference(&mut self, _reference: u8) {}
    fn bind_vertex_buffers(&mut self, _start_slot: u32, _buffers: &[(ResourceHandle, u64)]) {}
    fn bind_index_buffer(&mut self, _buffer: ResourceHandle, _offset: u64, _format: IndexFormat) {}
    fn bind_constant_buffer(&mut self, _stage: ShaderStage, _slot: u32, _buffer: ResourceHandle) {}
    fn bind_shader_resource(&mut self, _stage: ShaderStage, _slot: u32, _srv: ResourceHandle) {}
    fn bind_sampler(&mut self, _stage: ShaderStage, _slot: u32, _sampler: ResourceHandle) {}
    fn draw(&mut self, _vertex_count: u32, _instance_count: u32, _start_vertex: u32, _start_instance: u32) {}
    fn draw_indexed(&mut self, _index_count: u32, _instance_count: u32, _start_index: u32, _base_vertex: i32, _start_instance: u32) {}
    fn draw_indirect(&mut self, _buffer: ResourceHandle, _offset: u64, _draw_count: u32) {}
    fn draw_indexed_indirect(&mut self, _buffer: ResourceHandle, _offset: u64, _draw_count: u32) {}
    fn dispatch(&mut self, _group_count_x: u32, _group_count_y: u32, _group_count_z: u32) {}
    fn dispatch_indirect(&mut self, _buffer: ResourceHandle, _offset: u64) {}
    fn clear_render_target(&mut self, _view: ResourceHandle, _color: [f32; 4]) {}
    fn clear_depth_stencil(&mut self, _view: ResourceHandle, _clear_depth: Option<f32>, _clear_stencil: Option<u8>) {}
    fn insert_debug_marker(&mut self, _name: &str) {}
    fn begin_debug_group(&mut self, _name: &str) {}
    fn end_debug_group(&mut self) {}
    fn resource_barrier(&mut self, _barriers: &[super::device::ResourceBarrier]) {}
}

impl ICommandQueue for GlCommandQueueInternal {
    fn submit(&self, _command_lists: &[&dyn ICommandList], _wait_semaphores: &[Arc<dyn ISemaphore>], _signal_semaphores: &[Arc<dyn ISemaphore>]) -> RhiResult<()> { Ok(()) }
    fn present(&self, _swap_chain: &dyn ISwapChain) -> RhiResult<()> { Ok(()) }
    fn signal(&self, _fence: &dyn IFence, _value: u64) -> RhiResult<()> { Ok(()) }
    fn wait(&self, _fence: &dyn IFence, _value: u64, _timeout_ms: u32) -> RhiResult<bool> { Ok(true) }
}

impl IFence for GlFenceInternal {
    fn get_value(&self) -> u64 { self.value.load(Ordering::SeqCst) }
    fn set_event_on_completion(&self, _value: u64) -> RhiResult<Arc<dyn std::any::Any + Send + Sync>> { Ok(Arc::new(())) }
}

impl ISemaphore for GlSemaphoreInternal {}

impl ISwapChain for GlSwapChainInternal {
    fn get_current_back_buffer_index(&self) -> u32 { 0 }
    fn get_back_buffer(&self) -> ResourceHandle { ResourceHandle(0) }
    fn resize(&mut self, width: u32, height: u32) -> RhiResult<()> { self.width = width; self.height = height; Ok(()) }
    fn present(&self) -> RhiResult<()> { Ok(()) }
}
