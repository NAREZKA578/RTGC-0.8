//! Graphics Module for RTGC-0.8
//! Provides rendering, camera, shaders, meshes, textures, and RHI abstraction

pub mod renderer;
// pub mod renderer_rhi; // Дублирует функциональность renderer.rs
pub mod camera;
pub mod shader;
pub mod mesh;
pub mod texture;
pub mod lod_system;
pub mod texture_streaming;
pub mod lighting;
pub mod rhi;
pub use rhi::rhi_module::{RhiFactory, RhiConfig, GraphicsBackend};
pub mod material;
pub mod particles;
pub mod debug_renderer;
pub mod gl_context;
// pub mod dx11_context; // DX11 has API issues - use through rhi instead
pub mod render_command;
pub mod render_queue;

pub use renderer::Renderer;
// pub use renderer_rhi::RendererRhi; // Дублирует функциональность
pub use camera::Camera;
pub use shader::Shader;
pub use mesh::{Mesh, MeshHandle};
pub use texture::Texture;
pub use gl_context::GlContext;
pub use render_command::{RenderCommand, Handle};
pub use render_queue::{RenderQueue, RenderQueueStats};
// pub use lod_system::LodSystem; // нет такого типа, используется LodManager
// pub use texture_streaming::TextureStreamer; // нет такого типа
// pub use lighting::{Light, LightManager, LightingConfig}; // нет LightManager и LightingConfig
// pub use rhi::{RhiFactory, RhiConfig, IDevice, GraphicsBackend, RhiManager}; // большинство типов не существует
pub use material::{Material, MaterialManager, MaterialLayers, MaterialParams, TextureQuality, MaterialStats};
