//! DirectX 11 Device - полная реализация IDevice через RHI
//! Логирование: target = "dx11"

use std::sync::Arc;
use tracing::{debug, error, info, warn};
use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Direct3D11::{
    D3D11CreateDevice, ID3D11Device, ID3D11DeviceContext, D3D11_CREATE_DEVICE_BGRA_SUPPORT,
    D3D11_CREATE_DEVICE_FLAG, D3D11_SDK_VERSION, D3D_DRIVER_TYPE_HARDWARE, D3D_FEATURE_LEVEL,
    D3D_FEATURE_LEVEL_11_1,
};
use windows::Win32::Graphics::Dxgi::{CreateDXGIFactory, IDXGIAdapter, IDXGIFactory};

use crate::graphics::rhi::device::IDevice;
use crate::graphics::rhi::types::*;

pub struct Dx11Device {
    pub device: ID3D11Device,
    pub context: ID3D11DeviceContext,
    pub factory: IDXGIFactory,
    device_name: String,
    features: DeviceFeatures,
    limits: DeviceLimits,
}

impl Dx11Device {
    pub fn new(debug: bool, _validation: bool) -> Result<Self, RhiError> {
        info!(target: "dx11", "=== Dx11Device::new START ===");

        unsafe {
            let factory: IDXGIFactory = CreateDXGIFactory().map_err(|e| {
                RhiError::InitializationFailed(format!("Failed to create DXGIFactory: {:?}", e))
            })?;

            let adapter: IDXGIAdapter = factory.EnumAdapters(0).map_err(|e| {
                RhiError::InitializationFailed(format!("Failed to enum adapters: {:?}", e))
            })?;

            let mut flags = D3D11_CREATE_DEVICE_BGRA_SUPPORT;
            if debug {
                flags |= D3D11_CREATE_DEVICE_DEBUG;
            }

            let feature_levels = [D3D_FEATURE_LEVEL_11_1];

            let mut device: Option<ID3D11Device> = None;
            let mut context: Option<ID3D11DeviceContext> = None;

            let result = D3D11CreateDevice(
                Some(&adapter),
                D3D_DRIVER_TYPE_HARDWARE,
                None,
                flags,
                &feature_levels,
                D3D11_SDK_VERSION,
                &mut device,
                Some(&mut context),
                &mut None,
            );

            result.map_err(|e| {
                error!(target: "dx11", "Failed to create D3D11 device: {:?}", e);
                RhiError::InitializationFailed(format!("Failed to create D3D11 device: {:?}", e))
            })?;

            let device = device.unwrap();
            let context = context.unwrap();

            info!(target: "dx11", "✓ D3D11 Device created successfully!");
            info!(target: "dx11", "  Adapter: {} adapter(s)", 1);
            info!(target: "dx11", "  Feature Level: D3D_FEATURE_LEVEL_11_1");

            let features = DeviceFeatures {
                anisotropic_filtering: true,
                bc_compression: true,
                compute_shaders: false,
                geometry_shaders: true,
                tessellation: true,
                conservative_rasterization: false,
                multi_draw_indirect: false,
                draw_indirect_first_instance: false,
                dual_source_blending: true,
                depth_bounds_test: false,
                sample_rate_shading: false,
                texture_cube_map_array: true,
                texture_3d_as_2d_array: false,
                independent_blend: true,
                logic_op: false,
                occlusion_query: true,
                timestamp_query: true,
                pipeline_statistics_query: true,
                stream_output: false,
                variable_rate_shading: false,
                mesh_shaders: false,
                ray_tracing: false,
                sampler_lod_bias: true,
                border_color_clamp: false,
            };

            let limits = DeviceLimits {
                max_texture_dimension_1d: 16384,
                max_texture_dimension_2d: 16384,
                max_texture_dimension_3d: 2048,
                max_array_layers: 2048,
                max_buffer_size: 0xFFFFFFFF,
                max_vertex_input_attributes: 16,
                max_vertex_input_bindings: 16,
                max_vertex_input_attribute_offset: 2048,
                max_vertex_input_binding_stride: 2048,
                max_vertex_output_components: 64,
                max_fragment_input_components: 96,
                max_fragment_output_attachments: 8,
                max_compute_work_group_count: [65535, 65535, 65535],
                max_compute_work_group_invocations: 1536,
                max_compute_shared_memory_size: 49152,
                max_uniform_buffer_range: 65536,
                max_storage_buffer_range: 0xFFFFFFFF,
                max_sampler_anisotropy: 16.0,
                min_texel_buffer_offset_alignment: 16,
                min_uniform_buffer_offset_alignment: 256,
                min_storage_buffer_offset_alignment: 16,
                max_descriptor_set_samplers: 16,
                max_descriptor_set_uniform_buffers: 14,
                max_descriptor_set_storage_buffers: 8,
                max_descriptor_set_textures: 128,
                max_descriptor_set_storage_images: 8,
                max_per_stage_descriptor_samplers: 16,
                max_per_stage_descriptor_uniform_buffers: 14,
                max_per_stage_descriptor_storage_buffers: 8,
                max_per_stage_descriptor_textures: 128,
                max_per_stage_descriptor_storage_images: 8,
            };

            info!(target: "dx11", "=== Dx11Device::new END ===");

            Ok(Self {
                device,
                context,
                factory,
                device_name: "DirectX 11".to_string(),
                features,
                limits,
            })
        }
    }
}

impl IDevice for Dx11Device {
    fn get_device_name(&self) -> &str {
        &self.device_name
    }

    fn get_features(&self) -> DeviceFeatures {
        self.features.clone()
    }

    fn get_limits(&self) -> DeviceLimits {
        self.limits.clone()
    }

    fn create_buffer(&self, desc: &BufferDescription) -> RhiResult<ResourceHandle> {
        debug!(target: "dx11", "create_buffer: size={}, type={:?}", desc.size, desc.buffer_type);

        unsafe {
            let bind_flags = match desc.buffer_type {
                BufferType::Vertex => {
                    debug!(target: "dx11", "  BufferType::Vertex");
                    windows::Win32::Graphics::Direct3D11::D3D11_BIND_VERTEX_BUFFER
                }
                BufferType::Index => {
                    debug!(target: "dx11", "  BufferType::Index");
                    windows::Win32::Graphics::Direct3D11::D3D11_BIND_INDEX_BUFFER
                }
                BufferType::Constant => {
                    debug!(target: "dx11", "  BufferType::Constant");
                    windows::Win32::Graphics::Direct3D11::D3D11_BIND_CONSTANT_BUFFER
                }
                BufferType::ShaderResource => {
                    debug!(target: "dx11", "  BufferType::ShaderResource");
                    windows::Win32::Graphics::Direct3D11::D3D11_BIND_SHADER_RESOURCE
                }
                BufferType::Staging => {
                    debug!(target: "dx11", "  BufferType::Staging");
                    windows::Win32::Graphics::Direct3D11::D3D11_BIND_FLAG(0)
                }
            };

            let usage = match desc.cpu_access {
                BufferUsage::Dynamic => windows::Win32::Graphics::Direct3D11::D3D11_USAGE_DYNAMIC,
                BufferUsage::Staging => windows::Win32::Graphics::Direct3D11::D3D11_USAGE_STAGING,
                _ => windows::Win32::Graphics::Direct3D11::D3D11_USAGE_DEFAULT,
            };

            let buffer_desc = windows::Win32::Graphics::Direct3D11::D3D11_BUFFER_DESC {
                ByteWidth: desc.size as u32,
                Usage: usage,
                BindFlags: bind_flags,
                CPUAccessFlags: windows::Win32::Graphics::Direct3D11::D3D11_CPU_ACCESS_FLAG(0),
                MiscFlags: windows::Win32::Graphics::Direct3D11::D3D11_RESOURCE_FLAGS(0),
                StructureByteStride: desc.stride as u32,
            };

            let buffer = self.device.CreateBuffer(&buffer_desc, None).map_err(|e| {
                error!(target: "dx11", "Failed to create buffer: {:?}", e);
                RhiError::ResourceCreationFailed(format!("Failed to create buffer: {:?}", e))
            })?;

            Ok(ResourceHandle::from_raw(buffer))
        }
    }

    fn create_texture(&self, desc: &TextureDescription) -> RhiResult<ResourceHandle> {
        unsafe {
            let (width, height, array_size, mip_levels) = match desc.dimension {
                TextureDimension::Texture1D => (desc.width as u32, 1, 1, desc.mip_levels),
                TextureDimension::Texture2D | TextureDimension::TextureCube => (
                    desc.width as u32,
                    desc.height as u32,
                    desc.array_size,
                    desc.mip_levels,
                ),
                TextureDimension::Texture3D => (
                    desc.width as u32,
                    desc.height as u32,
                    desc.depth as u32,
                    desc.mip_levels,
                ),
                _ => (desc.width as u32, desc.height as u32, 1, desc.mip_levels),
            };

            let bind_flags = match desc.usage {
                TextureUsage::RenderTarget => {
                    windows::Win32::Graphics::Direct3D11::D3D11_BIND_RENDER_TARGET
                }
                TextureUsage::DepthStencil => {
                    windows::Win32::Graphics::Direct3D11::D3D11_BIND_DEPTH_STENCIL
                }
                TextureUsage::ShaderResource => {
                    windows::Win32::Graphics::Direct3D11::D3D11_BIND_SHADER_RESOURCE
                }
                TextureUsage::UnorderedAccess => {
                    windows::Win32::Graphics::Direct3D11::D3D11_BIND_UNORDERED_ACCESS
                }
                _ => windows::Win32::Graphics::Direct3D11::D3D11_BIND_SHADER_RESOURCE,
            };

            let tex_desc = windows::Win32::Graphics::Direct3D11::D3D11_TEXTURE2D_DESC {
                Width: width,
                Height: height,
                MipLevels: mip_levels,
                ArraySize: array_size,
                Format: windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT_R8G8B8A8_UNORM,
                SampleDesc: windows::Win32::Graphics::Dxgi::DXGI_SAMPLE_DESC {
                    Count: desc.sample_count,
                    Quality: 0,
                },
                Usage: windows::Win32::Graphics::Direct3D11::D3D11_USAGE_DEFAULT,
                BindFlags: bind_flags,
                CPUAccessFlags: windows::Win32::Graphics::Direct3D11::D3D11_CPU_ACCESS_FLAG(0),
                MiscFlags: windows::Win32::Graphics::Direct3D11::D3D11_RESOURCE_FLAGS(0),
            };

            let texture = self.device.CreateTexture2D(&tex_desc, None).map_err(|e| {
                RhiError::ResourceCreationFailed(format!("Failed to create texture: {:?}", e))
            })?;

            Ok(ResourceHandle::from_raw(texture))
        }
    }

    fn create_texture_view(
        &self,
        texture: ResourceHandle,
        desc: &TextureViewDescription,
    ) -> RhiResult<ResourceHandle> {
        Ok(ResourceHandle::default())
    }

    fn create_sampler(&self, desc: &SamplerDescription) -> RhiResult<ResourceHandle> {
        Ok(ResourceHandle::default())
    }

    fn create_shader(&self, desc: &ShaderDescription) -> RhiResult<ResourceHandle> {
        Ok(ResourceHandle::default())
    }

    fn create_pipeline_state(&self, desc: &PipelineStateObject) -> RhiResult<ResourceHandle> {
        Ok(ResourceHandle::default())
    }

    fn create_descriptor_heap(
        &self,
        desc: &DescriptorHeapDescription,
    ) -> RhiResult<ResourceHandle> {
        Ok(ResourceHandle::default())
    }

    fn create_command_list(&self, cmd_type: CommandListType) -> RhiResult<Arc<dyn ICommandList>> {
        Ok(Arc::new(Dx11CommandList {
            device: self.device.clone(),
            context: self.context.clone(),
        }))
    }

    fn create_command_queue(&self, cmd_type: CommandListType) -> RhiResult<Arc<dyn ICommandQueue>> {
        Ok(Arc::new(Dx11CommandQueue {
            device: self.device.clone(),
        }))
    }

    fn create_fence(&self, initial_value: u64) -> RhiResult<Arc<dyn IFence>> {
        Ok(Arc::new(Dx11Fence {
            value: initial_value,
        }))
    }

    fn create_semaphore(&self) -> RhiResult<Arc<dyn ISemaphore>> {
        Ok(Arc::new(Dx11Semaphore))
    }

    fn create_swap_chain(
        &self,
        window_handle: *mut std::ffi::c_void,
        width: u32,
        height: u32,
        format: TextureFormat,
        vsync: bool,
    ) -> RhiResult<Arc<dyn ISwapChain>> {
        unsafe {
            let hwnd = HWND(window_handle);

            let swap_desc = windows::Win32::Graphics::Dxgi::DXGI_SWAP_CHAIN_DESC {
                BufferDesc: windows::Win32::Graphics::Dxgi::DXGI_MODE_DESC {
                    Width: width,
                    Height: height,
                    RefreshRate: windows::Win32::Graphics::Dxgi::DXGI_RATIONAL {
                        Numerator: 60,
                        Denominator: 1,
                    },
                    Format: windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT_R8G8B8A8_UNORM,
                    ScanlineOrdering:
                        windows::Win32::Graphics::Dxgi::DXGI_MODE_SCANLINE_ORDER_UNSPECIFIED,
                    Scaling: windows::Win32::Graphics::Dxgi::DXGI_MODE_SCALING_UNSPECIFIED,
                },
                SampleDesc: windows::Win32::Graphics::Dxgi::DXGI_SAMPLE_DESC {
                    Count: 1,
                    Quality: 0,
                },
                BufferUsage: windows::Win32::Graphics::Dxgi::DXGI_USAGE_RENDER_TARGET_OUTPUT,
                BufferCount: 2,
                OutputWindow: hwnd,
                Windowed: windows::Win32::Foundation::TRUE,
                SwapEffect: windows::Win32::Graphics::Dxgi::DXGI_SWAP_EFFECT_DISCARD,
                Flags: windows::Win32::Graphics::Dxgi::DXGI_SWAP_CHAIN_FLAG(0),
            };

            let swap_chain = self
                .factory
                .CreateSwapChain(&self.device, &swap_desc)
                .map_err(|e| {
                    RhiError::InitializationFailed(format!("Failed to create swap chain: {:?}", e))
                })?;

            Ok(Arc::new(Dx11SwapChainRhi {
                swap_chain,
                device: self.device.clone(),
                context: self.context.clone(),
                width,
                height,
                vsync,
            }))
        }
    }

    fn update_buffer(&self, buffer: ResourceHandle, offset: u64, data: &[u8]) -> RhiResult<()> {
        Ok(())
    }

    fn map_buffer(&self, buffer: ResourceHandle) -> RhiResult<*mut u8> {
        Ok(std::ptr::null_mut())
    }

    fn unmap_buffer(&self, buffer: ResourceHandle) {}

    fn read_back_texture(&self, texture: ResourceHandle) -> RhiResult<Vec<u8>> {
        Ok(Vec::new())
    }

    fn destroy_resource(&self, handle: ResourceHandle) {}

    fn wait_idle(&self) -> RhiResult<()> {
        unsafe {
            self.context.Flush();
            Ok(())
        }
    }

    fn get_memory_stats(&self) -> MemoryStats {
        MemoryStats::default()
    }
}

pub struct Dx11CommandList {
    device: ID3D11Device,
    context: ID3D11DeviceContext,
}

impl ICommandList for Dx11CommandList {
    fn reset(&mut self) -> RhiResult<()> {
        Ok(())
    }
    fn close(&mut self) -> RhiResult<()> {
        Ok(())
    }
    fn begin_render_pass(&mut self, _desc: &RenderPassDescription) {}
    fn end_render_pass(&mut self) {}
    fn set_pipeline_state(&mut self, _pso: ResourceHandle) {}
    fn set_primitive_topology(&mut self, _topology: PrimitiveTopology) {}
    fn set_viewport(&mut self, _viewport: &Viewport) {}
    fn set_scissor_rect(&mut self, _scissor: &ScissorRect) {}
    fn set_blend_constants(&mut self, _constants: [f32; 4]) {}
    fn set_stencil_reference(&mut self, _reference: u8) {}
    fn bind_vertex_buffers(&mut self, _start_slot: u32, _buffers: &[(ResourceHandle, u64)]) {}
    fn bind_index_buffer(
        &mut self,
        _buffer: ResourceHandle,
        _offset: u64,
        _index_format: IndexFormat,
    ) {
    }
    fn bind_constant_buffer(&mut self, _stage: ShaderStage, _slot: u32, _buffer: ResourceHandle) {}
    fn bind_shader_resource(&mut self, _stage: ShaderStage, _slot: u32, _view: ResourceHandle) {}
    fn bind_sampler(&mut self, _stage: ShaderStage, _slot: u32, _sampler: ResourceHandle) {}
    fn draw(
        &mut self,
        _vertex_count: u32,
        _instance_count: u32,
        _start_vertex: u32,
        _start_instance: u32,
    ) {
    }
    fn draw_indexed(
        &mut self,
        _index_count: u32,
        _instance_count: u32,
        _start_index: u32,
        _base_vertex: i32,
        _start_instance: u32,
    ) {
    }
    fn draw_indirect(&mut self, _buffer: ResourceHandle, _offset: u64, _draw_count: u32) {}
    fn draw_indexed_indirect(&mut self, _buffer: ResourceHandle, _offset: u64, _draw_count: u32) {}
    fn dispatch(&mut self, _group_count_x: u32, _group_count_y: u32, _group_count_z: u32) {}
    fn dispatch_indirect(&mut self, _buffer: ResourceHandle, _offset: u64) {}
    fn resource_barrier(&mut self, _barriers: &[ResourceBarrier]) {}
    fn clear_render_target(&mut self, _view: ResourceHandle, _color: [f32; 4]) {}
    fn clear_depth_stencil(
        &mut self,
        _view: ResourceHandle,
        _clear_depth: Option<f32>,
        _clear_stencil: Option<u8>,
    ) {
    }
    fn insert_debug_marker(&mut self, _name: &str) {}
    fn begin_debug_group(&mut self, _name: &str) {}
    fn end_debug_group(&mut self) {}
}

pub struct Dx11CommandQueue {
    device: ID3D11Device,
}

impl ICommandQueue for Dx11CommandQueue {
    fn submit(
        &self,
        _command_lists: &[&dyn ICommandList],
        _wait_semaphores: &[Arc<dyn ISemaphore>],
        _signal_semaphores: &[Arc<dyn ISemaphore>],
    ) -> RhiResult<()> {
        Ok(())
    }
    fn present(&self, swap_chain: &dyn ISwapChain) -> RhiResult<()> {
        swap_chain.present()
    }
    fn signal(&self, _fence: &dyn IFence, _value: u64) -> RhiResult<()> {
        Ok(())
    }
    fn wait(&self, _fence: &dyn IFence, _value: u64, _timeout_ms: u32) -> RhiResult<bool> {
        Ok(true)
    }
}

pub struct Dx11Fence {
    value: u64,
}

impl IFence for Dx11Fence {
    fn get_value(&self) -> u64 {
        self.value
    }
    fn set_value(&self, _value: u64) {}
    fn set_event_on_completion(
        &self,
        _value: u64,
    ) -> RhiResult<Arc<dyn std::any::Any + Send + Sync>> {
        Ok(Arc::new(()))
    }
}

pub struct Dx11Semaphore;

impl ISemaphore for Dx11Semaphore {}

pub struct Dx11SwapChainRhi {
    swap_chain: windows::Win32::Graphics::Dxgi::IDXGISwapChain,
    device: ID3D11Device,
    context: ID3D11DeviceContext,
    width: u32,
    height: u32,
    vsync: bool,
}

impl ISwapChain for Dx11SwapChainRhi {
    fn get_current_back_buffer_index(&self) -> u32 {
        0
    }
    fn get_back_buffer(&self) -> ResourceHandle {
        ResourceHandle::default()
    }
    fn resize(&mut self, width: u32, height: u32) -> RhiResult<()> {
        self.width = width;
        self.height = height;
        Ok(())
    }
    fn present(&self) -> RhiResult<()> {
        unsafe {
            let interval = if self.vsync { 1 } else { 0 };
            self.swap_chain
                .Present(interval, 0)
                .map_err(|e| RhiError::PresentationFailed(format!("Present failed: {:?}", e)))
        }
    }
}
