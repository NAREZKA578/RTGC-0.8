// RHI Module - Render Hardware Interface
// Universal abstraction layer for Vulkan, DirectX 12, and OpenGL
// Re-exports sibling modules via super (parent mod.rs declares them)

pub use super::types::*;
pub use super::device::*;
pub use super::factory::*;
pub use super::gl::*;

#[cfg(feature = "dx12")]
pub use super::dx12::*;

#[cfg(feature = "vulkan")]
pub use super::vulkan::*;

/// RHI Factory - creates device instances for different backends
pub struct RhiFactory;

impl RhiFactory {
    /// Create a device with the specified backend
    pub fn create_device(backend: GraphicsBackend, enable_validation: bool) -> RhiResult<Box<dyn IDevice>> {
        match backend {
            #[cfg(all(target_os = "windows", feature = "dx12"))]
            GraphicsBackend::DirectX12 => super::dx12::create_dx12_device(enable_validation),

            #[cfg(feature = "vulkan")]
            GraphicsBackend::Vulkan => super::vulkan::create_vulkan_device(enable_validation),

            _ => Err(RhiError::Unsupported("Requested backend is not available".to_string())),
        }
    }
    
    /// Get the best available backend for the current platform
    pub fn get_preferred_backend() -> GraphicsBackend {
        #[cfg(all(target_os = "windows", feature = "dx12"))]
        {
            GraphicsBackend::DirectX12
        }
        #[cfg(not(all(target_os = "windows", feature = "dx12")))]
        {
            #[cfg(feature = "vulkan")]
            {
                GraphicsBackend::Vulkan
            }
            #[cfg(not(feature = "vulkan"))]
            {
                // Fallback to OpenGL (not yet implemented in RHI)
                GraphicsBackend::OpenGL
            }
        }
    }
}

/// Graphics API backend
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphicsBackend {
    Vulkan,
    DirectX12,
    OpenGL,
    Metal,
}

impl GraphicsBackend {
    pub fn as_str(&self) -> &'static str {
        match self {
            GraphicsBackend::Vulkan => "Vulkan",
            GraphicsBackend::DirectX12 => "DirectX 12",
            GraphicsBackend::OpenGL => "OpenGL",
            GraphicsBackend::Metal => "Metal",
        }
    }
}

/// RHI initialization configuration
#[derive(Debug, Clone)]
pub struct RhiConfig {
    pub backend: GraphicsBackend,
    pub enable_validation: bool,
    pub enable_debug_layers: bool,
    pub max_frames_in_flight: u32,
    pub descriptor_pool_size: u32,
}

impl Default for RhiConfig {
    fn default() -> Self {
        Self {
            backend: RhiFactory::get_preferred_backend(),
            enable_validation: cfg!(debug_assertions),
            enable_debug_layers: cfg!(debug_assertions),
            max_frames_in_flight: 3,
            descriptor_pool_size: 1024,
        }
    }
}

/// Helper for building PSO descriptions
pub struct PipelineStateBuilder {
    vertex_shader: Option<ResourceHandle>,
    fragment_shader: Option<ResourceHandle>,
    compute_shader: Option<ResourceHandle>,
    input_layout: Option<InputLayout>,
    color_blend_states: Vec<ColorBlendState>,
    depth_state: DepthState,
    stencil_state: StencilState,
    rasterizer_state: RasterizerState,
    primitive_topology: PrimitiveTopology,
    sample_count: u32,
}

impl PipelineStateBuilder {
    pub fn new() -> Self {
        Self {
            vertex_shader: None,
            fragment_shader: None,
            compute_shader: None,
            input_layout: None,
            color_blend_states: Vec::new(),
            depth_state: DepthState::default(),
            stencil_state: StencilState::default(),
            rasterizer_state: RasterizerState::default(),
            primitive_topology: PrimitiveTopology::TriangleList,
            sample_count: 1,
        }
    }
    
    pub fn vertex_shader(mut self, shader: ResourceHandle) -> Self {
        self.vertex_shader = Some(shader);
        self
    }
    
    pub fn fragment_shader(mut self, shader: ResourceHandle) -> Self {
        self.fragment_shader = Some(shader);
        self
    }
    
    pub fn compute_shader(mut self, shader: ResourceHandle) -> Self {
        self.compute_shader = Some(shader);
        self
    }
    
    pub fn input_layout(mut self, layout: InputLayout) -> Self {
        self.input_layout = Some(layout);
        self
    }
    
    pub fn add_color_blend_state(mut self, state: ColorBlendState) -> Self {
        self.color_blend_states.push(state);
        self
    }
    
    pub fn depth_state(mut self, state: DepthState) -> Self {
        self.depth_state = state;
        self
    }
    
    pub fn stencil_state(mut self, state: StencilState) -> Self {
        self.stencil_state = state;
        self
    }
    
    pub fn rasterizer_state(mut self, state: RasterizerState) -> Self {
        self.rasterizer_state = state;
        self
    }
    
    pub fn primitive_topology(mut self, topology: PrimitiveTopology) -> Self {
        self.primitive_topology = topology;
        self
    }
    
    pub fn sample_count(mut self, count: u32) -> Self {
        self.sample_count = count;
        self
    }
    
    pub fn build(self) -> Result<PipelineStateObject, RhiError> {
        Ok(PipelineStateObject {
            vertex_shader: self.vertex_shader.ok_or_else(|| {
                RhiError::InvalidParameter("Vertex shader is required".to_string())
            })?,
            fragment_shader: self.fragment_shader,
            compute_shader: self.compute_shader,
            input_layout: self.input_layout.ok_or_else(|| {
                RhiError::InvalidParameter("Input layout is required".to_string())
            })?,
            color_blend_states: self.color_blend_states,
            depth_state: self.depth_state,
            stencil_state: self.stencil_state,
            rasterizer_state: self.rasterizer_state,
            primitive_topology: self.primitive_topology,
            sample_count: self.sample_count,
        })
    }
}

impl Default for PipelineStateBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper for building render pass descriptions
pub struct RenderPassBuilder {
    color_attachments: Vec<RenderAttachment>,
    depth_stencil_attachment: Option<DepthStencilAttachment>,
}

impl RenderPassBuilder {
    pub fn new() -> Self {
        Self {
            color_attachments: Vec::new(),
            depth_stencil_attachment: None,
        }
    }
    
    pub fn add_color_attachment(mut self, attachment: RenderAttachment) -> Self {
        self.color_attachments.push(attachment);
        self
    }
    
    pub fn depth_stencil_attachment(mut self, attachment: DepthStencilAttachment) -> Self {
        self.depth_stencil_attachment = Some(attachment);
        self
    }
    
    pub fn build(self) -> RenderPassDescription {
        RenderPassDescription {
            color_attachments: self.color_attachments,
            depth_stencil_attachment: self.depth_stencil_attachment,
        }
    }
}

impl Default for RenderPassBuilder {
    fn default() -> Self {
        Self::new()
    }
}
