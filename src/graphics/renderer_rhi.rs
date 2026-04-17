// Renderer with RHI abstraction - uses IDevice instead of direct glow calls
// This allows switching between OpenGL, Vulkan, DX12 backends without changes

use nalgebra::{Matrix4, UnitQuaternion, Vector3};
use std::collections::HashMap;
use std::sync::Arc;

use crate::graphics::rhi::{
    AddressMode, BufferDesc, BufferDescription, BufferType, BufferUsage, ClearValue,
    ColorBlendState, CullMode, DepthState, FilterMode, FrontFace, ICommandList, IDevice,
    InputLayout, LoadOp, PipelineStateObject, PrimitiveTopology, RasterizerState, RenderAttachment,
    RenderPassDescription, ResourceBarrier, ResourceHandle, ResourceState, RhiResult,
    SamplerDescription, ScissorRect, ShaderDescription, ShaderStage, StoreOp, TextureDescription,
    TextureDimension, TextureFormat, TextureType, TextureUsage, VertexAttribute, VertexFormat,
    Viewport,
};

use crate::graphics::{camera::Camera, mesh::Mesh, texture::Texture};
// use crate::graphics::models::{Model as ModelGen, Vertex as ModelVertex}; // нет такого модуля
use crate::graphics::lod_system::{LodManager, LodObject};
use crate::graphics::texture_streaming::TextureStreamingSystem;

#[derive(Debug, Clone)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub tex_coords: [f32; 2],
}

pub struct Model {
    pub meshes: Vec<Mesh>,
    pub textures: Vec<Texture>,
}

/// RHI-based Renderer
pub struct RendererRhi {
    device: Arc<dyn IDevice>,
    command_list: Option<Arc<dyn ICommandList>>,
    pub camera: Camera,

    // Resources
    terrain_mesh: Option<Mesh>,
    terrain_vertex_buffer: Option<ResourceHandle>,
    terrain_index_buffer: Option<ResourceHandle>,
    vehicle_vertex_buffer: Option<ResourceHandle>,
    vehicle_index_buffer: Option<ResourceHandle>,

    // Shaders and pipelines
    terrain_pipeline: Option<ResourceHandle>,
    vehicle_pipeline: Option<ResourceHandle>,
    sky_pipeline: Option<ResourceHandle>,
    hud_pipeline: Option<ResourceHandle>,

    // State
    models: HashMap<String, Model>,
    current_city_index: usize,
    pub menu_state: MenuState,
    pub lod_manager: LodManager,
    pub texture_streaming: TextureStreamingSystem,

    // Vehicle state
    vehicle_transform: Option<(Vector3<f32>, UnitQuaternion<f32>)>,
    vehicle_lights_enabled: bool,

    // Window dimensions
    width: u32,
    height: u32,

    // HUD
    hud_data: Option<crate::ui::hud::VehicleHudData>,

    // Sky and lighting
    sky_color_top: Vector3<f32>,
    sky_color_horizon: Vector3<f32>,
    sun_direction: Vector3<f32>,
    ambient_intensity: f32,

    // Font for HUD text
    font_texture: Option<ResourceHandle>,
    font_chars: HashMap<char, [f32; 4]>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MenuState {
    Loading,
    MainMenu,
    CitySelection,
    InGame,
    WorldCreation,
    Settings,
    Paused,
    CharacterCreation,
}

impl RendererRhi {
    pub fn new(
        device: Arc<dyn IDevice>,
        width: u32,
        height: u32,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let camera = Camera::new(
            Vector3::new(0.0, 0.0, 3.0),
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(0.0, 1.0, 0.0),
            45.0,
            width as f32 / height as f32,
            0.1,
            1000.0,
        );

        // Create bitmap font texture
        let (font_texture, font_chars) = Self::create_bitmap_font(&device)?;

        Ok(Self {
            device,
            command_list: None,
            camera,
            terrain_mesh: None,
            terrain_vertex_buffer: None,
            terrain_index_buffer: None,
            vehicle_vertex_buffer: None,
            vehicle_index_buffer: None,
            terrain_pipeline: None,
            vehicle_pipeline: None,
            sky_pipeline: None,
            hud_pipeline: None,
            models: HashMap::new(),
            current_city_index: 0,
            menu_state: MenuState::Loading,
            lod_manager: LodManager::new(),
            texture_streaming: TextureStreamingSystem::new(128, 10.0, 5),
            vehicle_transform: None,
            vehicle_lights_enabled: false,
            width,
            height,
            hud_data: None,
            sky_color_top: Vector3::new(0.4, 0.6, 0.9),
            sky_color_horizon: Vector3::new(0.7, 0.8, 0.9),
            sun_direction: Vector3::y(),
            ambient_intensity: 0.5,
            font_texture: Some(font_texture),
            font_chars,
        })
    }

    /// Create procedural bitmap font texture
    fn create_bitmap_font(
        device: &Arc<dyn IDevice>,
    ) -> Result<(ResourceHandle, HashMap<char, [f32; 4]>), Box<dyn std::error::Error>> {
        use std::collections::HashMap;

        // Create 128x128 RGBA texture
        let mut pixels = vec![255u8; 128 * 128 * 4];
        let mut font_chars = HashMap::new();

        // Generate glyphs for ASCII 32-127
        for (idx, c) in (32..=127).enumerate() {
            let col = idx % 16;
            let row = idx / 16;
            let base_x = col * 8;
            let base_y = row * 8;

            let u = col as f32 / 16.0;
            let v = row as f32 / 16.0;
            let w = 1.0 / 16.0;
            let h = 1.0 / 16.0;
            font_chars.insert(c as char, [u, v, w, h]);

            // Simple glyph pattern
            for dy in 0..8 {
                for dx in 0..8 {
                    let px = base_x + dx;
                    let py = base_y + dy;
                    let pidx = (py * 128 + px) * 4;

                    let pattern = match c {
                        b'0'..=b'9' => (dx + dy) % 3 == 0,
                        b'A'..=b'Z' | b'a'..=b'z' => (dx * dy) % 2 == 0,
                        b' ' => false,
                        _ => (dx + dy) % 2 == 0,
                    };

                    if pattern {
                        pixels[pidx] = 0;
                        pixels[pidx + 1] = 0;
                        pixels[pidx + 2] = 0;
                        pixels[pidx + 3] = 255;
                    }
                }
            }
        }

        let desc = TextureDescription {
            dimension: TextureDimension::D2,
            texture_type: TextureType::Texture2D,
            width: 128,
            height: 128,
            depth: 1,
            depth_or_array_layers: 1,
            mip_levels: 1,
            format: TextureFormat::R8G8B8A8Unorm,
            usage: TextureUsage::SHADER_READ,
            initial_state: ResourceState::ShaderResource,
        };

        let texture = device.create_texture(&desc)?;
        Ok((texture, font_chars))
    }

    pub fn set_terrain_mesh(&mut self, mesh: Mesh) {
        // Upload mesh data to GPU via RHI
        // Create vertex and index buffers from mesh data
        self.terrain_mesh = Some(mesh);
    }

    pub fn get_terrain_mesh(&self) -> Option<&Mesh> {
        self.terrain_mesh.as_ref()
    }

    pub fn set_vehicle_transform(&mut self, pos: Vector3<f32>, rot: UnitQuaternion<f32>) {
        self.vehicle_transform = Some((pos, rot));
    }

    pub fn set_hud_data(&mut self, data: crate::ui::hud::VehicleHudData) {
        self.hud_data = Some(data);
    }

    pub fn set_sky_color(&mut self, top: Vector3<f32>, horizon: Vector3<f32>) {
        self.sky_color_top = top;
        self.sky_color_horizon = horizon;
    }

    pub fn set_sun_direction(&mut self, dir: Vector3<f32>) {
        self.sun_direction = dir;
    }

    pub fn set_ambient_intensity(&mut self, intensity: f32) {
        self.ambient_intensity = intensity;
    }

    pub fn enable_vehicle_lights(&mut self, enable: bool) {
        self.vehicle_lights_enabled = enable;
    }

    pub fn render(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Begin frame - create command list for RHI rendering
        let cmd_list = self
            .device
            .create_command_list(crate::graphics::rhi::CommandListType::Direct);

        // Use cmd_list to record and submit rendering commands
        // Full RHI integration requires pipeline state, descriptor heaps, etc.
        if let Ok(_list) = cmd_list {
            // In a full implementation, we would:
            // 1. Begin render pass
            // 2. Bind pipelines and resources
            // 3. Record draw commands
            // 4. End render pass and submit
            tracing::trace!("Command list ready for recording");
        }

        // Clear screen via OpenGL (fallback for now)
        // Render pass would be implemented here in full RHI backend

        match self.menu_state {
            MenuState::Loading => self.render_loading_screen()?,
            MenuState::MainMenu => self.render_main_menu()?,
            MenuState::CitySelection => self.render_city_selection()?,
            MenuState::InGame | MenuState::WorldCreation => self.render_game()?,
            MenuState::Paused => {
                self.render_game()?;
                self.render_pause_overlay()?;
            }
            MenuState::Settings => self.render_settings()?,
            MenuState::CharacterCreation => self.render_character_creation()?,
        }

        Ok(())
    }

    fn render_loading_screen(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Render loading screen with background color
        Ok(())
    }

    fn render_main_menu(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Render main menu UI
        Ok(())
    }

    fn render_city_selection(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Render city selection UI
        Ok(())
    }

    fn render_game(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Render 3D scene
        self.render_sky()?;
        self.render_terrain()?;
        self.render_vehicle()?;

        // Render HUD
        if self.hud_data.is_some() {
            self.render_hud()?;
        }

        Ok(())
    }

    fn render_pause_overlay(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Render pause menu overlay
        Ok(())
    }

    fn render_sky(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Render sky gradient
        Ok(())
    }

    fn render_terrain(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Render terrain mesh
        Ok(())
    }

    fn render_vehicle(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Render vehicle box placeholder
        Ok(())
    }

    fn render_hud(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Render HUD elements using batched quads
        Ok(())
    }

    fn render_settings(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Render settings UI
        Ok(())
    }

    fn render_character_creation(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Render character creation UI
        Ok(())
    }

    pub fn get_width(&self) -> u32 {
        self.width
    }

    pub fn get_height(&self) -> u32 {
        self.height
    }

    pub fn set_camera(&mut self, camera: Camera) {
        self.camera = camera;
    }

    pub fn next_city(&mut self) {
        self.current_city_index = (self.current_city_index + 1) % 10;
    }

    pub fn prev_city(&mut self) {
        self.current_city_index = if self.current_city_index == 0 {
            9
        } else {
            self.current_city_index - 1
        };
    }
}
