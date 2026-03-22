use glow::{Context, HasContext};
use std::sync::Arc;
use std::collections::HashMap;
use nalgebra::{Vector3, Matrix4, UnitQuaternion};
use crate::graphics::{camera::Camera, mesh::Mesh, shader::Shader, texture::Texture};
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

pub struct Renderer {
    gl: Arc<Context>,
    pub shader: Shader,
    pub camera: Camera,
    models: HashMap<String, Model>,
    current_city_index: usize,
    pub menu_state: MenuState,
    pub lod_manager: LodManager,
    pub texture_streaming: TextureStreamingSystem,
    // SPRINT 1: Terrain & Vehicle rendering
    terrain_mesh: Option<Mesh>,
    vehicle_box_mesh: Option<Mesh>,
    vehicle_transform: Option<(Vector3<f32>, UnitQuaternion<f32>)>,
    // Window dimensions for HUD rendering
    width: u32,
    height: u32,
    // HUD Manager reference for rendering
    hud_data: Option<crate::ui::hud::VehicleHudData>,
    // SPRINT 5: Weather and Day/Night cycle support
    sky_color_top: Vector3<f32>,
    sky_color_horizon: Vector3<f32>,
    sun_direction: Vector3<f32>,
    ambient_intensity: f32,
    vehicle_lights_enabled: bool,
    // Задача 2: Vehicle shader
    vehicle_shader: Option<Shader>,
    // Исп-2: Sky shader (separate from terrain shader)
    sky_shader: Option<Shader>,
    // Задача 3: Sky VAO
    sky_vao: Option<glow::VertexArray>,
    sky_vbo: Option<glow::Buffer>,  // Сохраняем VBO для обновления цветов
    // Граф-1: Bitmap font texture
    font_texture: Option<Texture>,
    font_chars: HashMap<char, [f32; 4]>, // char -> [u, v, w, h] UV coords
    // Граф-2: Batched HUD VAO/VBO for optimization
    hud_vao: Option<glow::VertexArray>,
    hud_vbo: Option<glow::Buffer>,
    hud_vertices: Vec<f32>,
    // Граф-3: Minimap texture
    minimap_texture: Option<Texture>,
    minimap_size: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MenuState {
    Loading,
    MainMenu,
    CitySelection,
    InGame,
    WorldCreation,
    Settings,
    Paused,  // Ввод-2: Пауза внутри игры
}

impl Renderer {
    pub fn new(gl: Context) -> Result<Self, Box<dyn std::error::Error>> {
        let gl = Arc::new(gl);

        unsafe {
            gl.enable(glow::DEPTH_TEST);
            gl.depth_func(glow::LESS);
            gl.enable(glow::CULL_FACE);
            gl.cull_face(glow::BACK);
        }

        // Исп-4: Загружать шейдер из файла
        let vertex_src = std::fs::read_to_string("assets/shaders/terrain.vert")
            .unwrap_or_else(|_| include_str!("../../assets/shaders/terrain.vert").to_string());
        let fragment_src = std::fs::read_to_string("assets/shaders/terrain.frag")
            .unwrap_or_else(|_| include_str!("../../assets/shaders/terrain.frag").to_string());
        let shader = Shader::new(&gl, &vertex_src, &fragment_src)?;

        // Задача 2: Загрузить vehicle shader
        let vehicle_vertex_src = std::fs::read_to_string("assets/shaders/vehicle.vert")
            .unwrap_or_else(|_| include_str!("../../assets/shaders/vehicle.vert").to_string());
        let vehicle_fragment_src = std::fs::read_to_string("assets/shaders/vehicle.frag")
            .unwrap_or_else(|_| include_str!("../../assets/shaders/vehicle.frag").to_string());
        let vehicle_shader = Shader::new(&gl, &vehicle_vertex_src, &vehicle_fragment_src).ok();

        // Исп-2: Создать простой шейдер для неба
        let sky_shader = Shader::new(&gl,
            "#version 330 core\nlayout(location=0) in vec2 pos;\nlayout(location=1) in vec3 col;\nout vec3 v_col;\nvoid main() { gl_Position = vec4(pos, 0.0, 1.0); v_col = col; }",
            "#version 330 core\nin vec3 v_col; out vec4 FragColor;\nvoid main() { FragColor = vec4(v_col, 1.0); }"
        ).ok();

        let camera = Camera::new(
            Vector3::new(0.0, 0.0, 3.0),
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(0.0, 1.0, 0.0),
            45.0,
            800.0 / 600.0,
            0.1,
            100.0,
        );
        
        // Задача 3: Создать VAO для неба
        let (sky_vao, sky_vbo) = unsafe {
            let vao = gl.create_vertex_array().ok();
            let vbo = gl.create_buffer().ok();
            if let Some(v) = vao {
                gl.bind_vertex_array(Some(v));
                // Вершины для 2 треугольников на весь экран [x, y, r, g, b]
                let verts: [f32; 30] = [
                   -1.0, -1.0,  0.7, 0.8, 0.9,  // bottom-left horizon
                    1.0, -1.0,  0.7, 0.8, 0.9,  // bottom-right horizon
                    1.0,  1.0,  0.4, 0.6, 0.9,  // top-right top
                   -1.0, -1.0,  0.7, 0.8, 0.9,
                    1.0,  1.0,  0.4, 0.6, 0.9,
                   -1.0,  1.0,  0.4, 0.6, 0.9,  // top-left top
                ];
                if let Some(b) = vbo {
                    gl.bind_buffer(glow::ARRAY_BUFFER, Some(b));
                    gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, bytemuck::cast_slice(&verts), glow::STATIC_DRAW);
                    gl.enable_vertex_attrib_array(0);
                    gl.vertex_attrib_pointer_f32(0, 2, glow::FLOAT, false, 20, 0);
                    gl.enable_vertex_attrib_array(1);
                    gl.vertex_attrib_pointer_f32(1, 3, glow::FLOAT, false, 20, 8);
                }
            }
            (vao, vbo)
        };
        
        // Граф-1: Создать bitmap font texture (процедурно, 128x128, 16x16 сетка символов)
        let (font_texture, font_chars) = Self::create_bitmap_font(&gl);
        
        // Граф-2: Создать VAO/VBO для батчинга HUD
        let (hud_vao, hud_vbo) = unsafe {
            let vao = gl.create_vertex_array().ok();
            let vbo = gl.create_buffer().ok();
            if let Some(vao) = vao {
                gl.bind_vertex_array(Some(vao));
            }
            if let Some(vbo) = vbo {
                gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
                // Пустой буфер, будем обновлять каждый кадр
                gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, &[], glow::DYNAMIC_DRAW);
                gl.enable_vertex_attrib_array(0); // position: vec2
                gl.vertex_attrib_pointer_f32(0, 2, glow::FLOAT, false, 24, 0);
                gl.enable_vertex_attrib_array(1); // color: vec4
                gl.vertex_attrib_pointer_f32(1, 4, glow::FLOAT, false, 24, 8);
                gl.enable_vertex_attrib_array(2); // uv: vec2
                gl.vertex_attrib_pointer_f32(2, 2, glow::FLOAT, false, 24, 16);
            }
            gl.bind_vertex_array(None);
            gl.bind_buffer(glow::ARRAY_BUFFER, None);
            (vao, vbo)
        };
        
        Ok(Self {
            gl,
            shader,
            camera,
            models: HashMap::new(),
            current_city_index: 0,
            menu_state: MenuState::Loading,
            lod_manager: LodManager::new(),
            texture_streaming: TextureStreamingSystem::new(128, 10.0, 5),
            // SPRINT 1: Initialize terrain & vehicle mesh placeholders
            terrain_mesh: None,
            vehicle_box_mesh: None,
            vehicle_transform: None,
            hud_data: None,
            // SPRINT 5: Weather and Day/Night defaults
            sky_color_top: Vector3::new(0.4, 0.6, 0.9),
            sky_color_horizon: Vector3::new(0.7, 0.8, 0.9),
            sun_direction: Vector3::y(),
            ambient_intensity: 0.5,
            vehicle_lights_enabled: false,
            // Задача 2: Vehicle shader
            vehicle_shader,
            // Исп-2: Sky shader
            sky_shader,
            // Задача 3: Sky VAO
            sky_vao,
            sky_vbo,
            // Граф-1: Bitmap font
            font_texture: Some(font_texture),
            font_chars,
            // Граф-2: Batched HUD
            hud_vao,
            hud_vbo,
            hud_vertices: Vec::with_capacity(1024),
            // Граф-3: Minimap
            minimap_texture: None,
            minimap_size: 128,
            width: 800,
            height: 600,
        })
    }
    
    /// Граф-1: Создать процедурную bitmap font текстуру 128x128
    fn create_bitmap_font(gl: &Arc<Context>) -> (Texture, HashMap<char, [f32; 4]>) {
        use std::collections::HashMap;
        // Создаём текстуру 128x128 с символами 8x8 в сетке 16x16
        let mut pixels = vec![255u8; 128 * 128 * 4]; // RGBA
        let mut font_chars = HashMap::new();
        
        // Простые глифы для ASCII 32-127 (96 символов)
        // Каждый символ 8x8 пикселей, сетка 16 колонок × 6 рядов = 96 мест
        for (idx, c) in (32..=127).enumerate() {
            let col = idx % 16;
            let row = idx / 16;
            let base_x = col * 8;
            let base_y = row * 8;
            
            // UV координаты для этого символа
            let u = col as f32 / 16.0;
            let v = row as f32 / 16.0;
            let w = 1.0 / 16.0;
            let h = 1.0 / 16.0;
            font_chars.insert(c as char, [u, v, w, h]);
            
            // Рисуем простой глиф (паттерн на основе кода символа)
            for dy in 0..8 {
                for dx in 0..8 {
                    let px = base_x + dx;
                    let py = base_y + dy;
                    let pidx = (py * 128 + px) * 4;
                    
                    // Простой паттерн: некоторые пиксели чёрные, некоторые белые
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
                    } else {
                        pixels[pidx] = 255;
                        pixels[pidx + 1] = 255;
                        pixels[pidx + 2] = 255;
                        pixels[pidx + 3] = 0;
                    }
                }
            }
        }
        
        let texture = Texture::from_rgba8(gl, 128, 128, &pixels).unwrap_or_else(|_| {
            // Fallback: создать пустую текстуру
            Texture::from_rgba8(gl, 1, 1, &[255, 255, 255, 255]).unwrap()
        });
        
        (texture, font_chars)
    }
    
    /// Set the terrain mesh for rendering
    pub fn set_terrain_mesh(&mut self, mesh: Mesh) {
        self.terrain_mesh = Some(mesh);
    }
    
    /// Set vehicle transform and HUD data
    pub fn set_vehicle_transform(&mut self, pos: Vector3<f32>, rot: UnitQuaternion<f32>) {
        self.vehicle_transform = Some((pos, rot));
    }
    
    /// Set HUD data for rendering
    pub fn set_hud_data(&mut self, data: crate::ui::hud::VehicleHudData) {
        self.hud_data = Some(data);
    }

    // SPRINT 5: Weather and Day/Night cycle methods
    pub fn set_sky_color(&mut self, top: Vector3<f32>, horizon: Vector3<f32>) {
        self.sky_color_top = top;
        self.sky_color_horizon = horizon;
        // Граф-4: Обновить VAO неба с новыми цветами
        self.update_sky_colors(top, horizon);
    }

    /// Граф-4: Обновить цвета неба в VAO
    fn update_sky_colors(&self, top: Vector3<f32>, horizon: Vector3<f32>) {
        unsafe {
            if let Some(vao) = self.sky_vao {
                self.gl.bind_vertex_array(Some(vao));
                // Обновить вершины с новыми цветами через buffer_sub_data
                let verts: [f32; 30] = [
                   -1.0, -1.0,  horizon.x, horizon.y, horizon.z,  // bottom-left horizon
                    1.0, -1.0,  horizon.x, horizon.y, horizon.z,  // bottom-right horizon
                    1.0,  1.0,  top.x, top.y, top.z,              // top-right top
                   -1.0, -1.0,  horizon.x, horizon.y, horizon.z,
                    1.0,  1.0,  top.x, top.y, top.z,
                   -1.0,  1.0,  top.x, top.y, top.z,              // top-left top
                ];
                if let Some(_vbo) = self.sky_vbo {
                    // Используем buffer_sub_data для обновления без пересоздания
                    self.gl.buffer_sub_data_u8_slice(glow::ARRAY_BUFFER, 0, bytemuck::cast_slice(&verts));
                }
            }
        }
    }

    pub fn set_sun_direction(&mut self, dir: Vector3<f32>) {
        self.sun_direction = dir;
    }

    pub fn set_ambient_intensity(&mut self, intensity: f32) {
        self.ambient_intensity = intensity.clamp(0.0, 1.0);
    }

    pub fn enable_vehicle_lights(&mut self, enable: bool) {
        self.vehicle_lights_enabled = enable;
    }

    /// Create a simple box mesh for the vehicle (temporary until GLTF loading works)
    pub fn create_vehicle_box_mesh(&mut self, half_extents: Vector3<f32>) -> Result<(), Box<dyn std::error::Error>> {
        // Create a unit cube centered at origin, scaled by half_extents
        let hx = half_extents.x;
        let hy = half_extents.y;
        let hz = half_extents.z;
        
        // Cube vertices: 8 corners with normals
        let vertices: Vec<f32> = vec![
            // Front face (z = +hz)
            -hx, -hy,  hz,  0.0, 0.0, 1.0,  0.0, 0.0,
             hx, -hy,  hz,  0.0, 0.0, 1.0,  1.0, 0.0,
             hx,  hy,  hz,  0.0, 0.0, 1.0,  1.0, 1.0,
            -hx,  hy,  hz,  0.0, 0.0, 1.0,  0.0, 1.0,
            // Back face (z = -hz)
             hx, -hy, -hz,  0.0, 0.0,-1.0,  0.0, 0.0,
            -hx, -hy, -hz,  0.0, 0.0,-1.0,  1.0, 0.0,
            -hx,  hy, -hz,  0.0, 0.0,-1.0,  1.0, 1.0,
             hx,  hy, -hz,  0.0, 0.0,-1.0,  0.0, 1.0,
            // Top face (y = +hy)
            -hx,  hy, -hz,  0.0, 1.0, 0.0,  0.0, 0.0,
             hx,  hy, -hz,  0.0, 1.0, 0.0,  1.0, 0.0,
             hx,  hy,  hz,  0.0, 1.0, 0.0,  1.0, 1.0,
            -hx,  hy,  hz,  0.0, 1.0, 0.0,  0.0, 1.0,
            // Bottom face (y = -hy)
            -hx, -hy,  hz,  0.0,-1.0, 0.0,  0.0, 0.0,
             hx, -hy,  hz,  0.0,-1.0, 0.0,  1.0, 0.0,
             hx, -hy, -hz,  0.0,-1.0, 0.0,  1.0, 1.0,
            -hx, -hy, -hz,  0.0,-1.0, 0.0,  0.0, 1.0,
            // Right face (x = +hx)
             hx, -hy, -hz,  1.0, 0.0, 0.0,  0.0, 0.0,
             hx,  hy, -hz,  1.0, 0.0, 0.0,  1.0, 0.0,
             hx,  hy,  hz,  1.0, 0.0, 0.0,  1.0, 1.0,
             hx, -hy,  hz,  1.0, 0.0, 0.0,  0.0, 1.0,
            // Left face (x = -hx)
            -hx, -hy,  hz, -1.0, 0.0, 0.0,  0.0, 0.0,
            -hx,  hy,  hz, -1.0, 0.0, 0.0,  1.0, 0.0,
            -hx,  hy, -hz, -1.0, 0.0, 0.0,  1.0, 1.0,
            -hx, -hy, -hz, -1.0, 0.0, 0.0,  0.0, 1.0,
        ];
        
        let indices: Vec<u32> = vec![
            0, 1, 2, 0, 2, 3,       // Front
            4, 5, 6, 4, 6, 7,       // Back
            8, 9, 10, 8, 10, 11,    // Top
            12, 13, 14, 12, 14, 15, // Bottom
            16, 17, 18, 16, 18, 19, // Right
            20, 21, 22, 20, 22, 23, // Left
        ];
        
        self.vehicle_box_mesh = Some(Mesh::new_raw(&self.gl, &vertices, &indices)?);
        Ok(())
    }
    
    pub fn render(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        unsafe {
            // SPRINT 5: Clear with sky gradient color (using top color for now)
            self.gl.clear_color(
                self.sky_color_top.x,
                self.sky_color_top.y,
                self.sky_color_top.z,
                1.0
            );
            self.gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
        }

        // Задача 3: Рендерить небо перед сценой
        if self.menu_state == MenuState::InGame {
            self.render_sky()?;
        }

        // Update LOD system based on camera position
        self.lod_manager.update_all_lods(&self.camera.position);

        // Update texture streaming based on camera position
        self.texture_streaming.update_camera_position(nalgebra::Vector2::new(
            self.camera.position.x,
            self.camera.position.z,
        ));

        match self.menu_state {
            MenuState::Loading => self.render_loading_screen()?,
            MenuState::MainMenu => self.render_main_menu()?,
            MenuState::CitySelection => self.render_city_selection()?,
            MenuState::InGame | MenuState::Paused => self.render_game()?,
            MenuState::WorldCreation => self.render_world_creation()?,
            MenuState::Settings => self.render_settings()?,
        }

        Ok(())
    }
    
    pub fn update_camera_for_frame(&mut self, truck_position: Vector3<f32>, truck_rotation: UnitQuaternion<f32>) {
        self.camera.update_for_truck(truck_position, truck_rotation);
    }
    
    fn render_loading_screen(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        unsafe {
            self.gl.disable(glow::DEPTH_TEST);
            self.gl.clear_color(0.05, 0.05, 0.1, 1.0);
            self.gl.clear(glow::COLOR_BUFFER_BIT);
            // Центральная надпись (пока просто прямоугольник)
            let w = self.width as f32;
            let h = self.height as f32;
            self.draw_rect(w/2.0 - 100.0, h/2.0 - 30.0, 200.0, 60.0, [0.2, 0.4, 0.6, 0.9]);
            self.gl.enable(glow::DEPTH_TEST);
        }
        Ok(())
    }
    
    fn render_main_menu(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Задача 7: Меню рисуется, а не println!
        unsafe {
            self.gl.disable(glow::DEPTH_TEST);
            self.gl.clear_color(0.05, 0.05, 0.1, 1.0);
            self.gl.clear(glow::COLOR_BUFFER_BIT);

            let w = self.width as f32;
            let h = self.height as f32;

            // Центральная панель
            self.draw_rect(w/2.0 - 150.0, h/2.0 - 120.0, 300.0, 240.0, [0.1, 0.1, 0.15, 0.9]);

            // Пункты меню как цветные полосы
            // "Новая игра" — зелёная
            self.draw_rect(w/2.0 - 120.0, h/2.0 - 80.0, 240.0, 40.0, [0.2, 0.6, 0.2, 0.8]);
            // "Продолжить" — синяя
            self.draw_rect(w/2.0 - 120.0, h/2.0 - 30.0, 240.0, 40.0, [0.2, 0.3, 0.6, 0.8]);
            // "Настройки" — серая
            self.draw_rect(w/2.0 - 120.0, h/2.0 + 20.0, 240.0, 40.0, [0.3, 0.3, 0.3, 0.8]);
            // "Выход" — красная
            self.draw_rect(w/2.0 - 120.0, h/2.0 + 70.0, 240.0, 40.0, [0.6, 0.2, 0.2, 0.8]);

            self.gl.enable(glow::DEPTH_TEST);
        }
        Ok(())
    }
    
    fn render_city_selection(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        unsafe {
            self.gl.disable(glow::DEPTH_TEST);
            self.gl.clear_color(0.05, 0.05, 0.1, 1.0);
            self.gl.clear(glow::COLOR_BUFFER_BIT);
            let w = self.width as f32;
            let h = self.height as f32;
            // Панель выбора города
            self.draw_rect(w/2.0 - 200.0, h/2.0 - 150.0, 400.0, 300.0, [0.1, 0.1, 0.15, 0.9]);
            self.gl.enable(glow::DEPTH_TEST);
        }
        Ok(())
    }
    
    fn render_game(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Render the actual game scene with proper OpenGL rendering
        
        // Get visible objects from LOD system
        let visible_objects = self.lod_manager.get_objects_in_view(&self.camera.position, 100.0);

        // Use the shader
        self.shader.bind(&self.gl);
        
        // Set up view and projection matrices
        let projection = self.camera.projection_matrix();
        let view = self.camera.view_matrix();
        
        unsafe {
            // Set uniforms with safe handling - skip if uniform not found
            if let Some(u_projection) = self.gl.get_uniform_location(self.shader.program(), "u_projection") {
                self.gl.uniform_matrix_4_f32_slice(Some(&u_projection), false, projection.as_slice());
            }
            if let Some(u_view) = self.gl.get_uniform_location(self.shader.program(), "u_view") {
                self.gl.uniform_matrix_4_f32_slice(Some(&u_view), false, view.as_slice());
            }
            // SPRINT 5: Light position from sun direction (scaled for shader)
            if let Some(u_light_pos) = self.gl.get_uniform_location(self.shader.program(), "u_light_pos") {
                let light_pos = self.sun_direction * 100.0;
                self.gl.uniform_3_f32(Some(&u_light_pos), light_pos.x, light_pos.y, light_pos.z);
            }
            if let Some(u_view_pos) = self.gl.get_uniform_location(self.shader.program(), "u_view_pos") {
                self.gl.uniform_3_f32(Some(&u_view_pos), self.camera.position.x, self.camera.position.y, self.camera.position.z);
            }
            // SPRINT 5: Light color affected by ambient intensity and weather
            if let Some(u_light_color) = self.gl.get_uniform_location(self.shader.program(), "u_light_color") {
                let light_intensity = self.ambient_intensity;
                self.gl.uniform_3_f32(Some(&u_light_color), 
                    light_intensity, light_intensity, light_intensity * 1.1);
            }
            // Исп-7: Ambient intensity uniform
            if let Some(u) = self.gl.get_uniform_location(self.shader.program(), "u_ambient_intensity") {
                self.gl.uniform_1_f32(Some(&u), self.ambient_intensity);
            }
            // Задача 10: Fog uniforms
            if let Some(u) = self.gl.get_uniform_location(self.shader.program(), "u_fog_start") {
                self.gl.uniform_1_f32(Some(&u), 200.0);
            }
            if let Some(u) = self.gl.get_uniform_location(self.shader.program(), "u_fog_end") {
                self.gl.uniform_1_f32(Some(&u), 500.0);
            }
            if let Some(u) = self.gl.get_uniform_location(self.shader.program(), "u_fog_color") {
                self.gl.uniform_3_f32(Some(&u), self.sky_color_horizon.x,
                                      self.sky_color_horizon.y, self.sky_color_horizon.z);
            }
            // Исп-7: Solid color disabled for terrain rendering
            if let Some(u) = self.gl.get_uniform_location(self.shader.program(), "u_use_solid_color") {
                self.gl.uniform_1_i32(Some(&u), 0);
            }
        }
        
        // === SPRINT 1: Render terrain mesh ===
        if let Some(ref terrain_mesh) = self.terrain_mesh {
            unsafe {
                // Set model matrix to identity for terrain
                if let Some(u_model) = self.gl.get_uniform_location(self.shader.program(), "u_model") {
                    let identity = Matrix4::identity();
                    self.gl.uniform_matrix_4_f32_slice(Some(&u_model), false, identity.as_slice());
                }
            }
            terrain_mesh.draw(&self.gl);
        }

        // === SPRINT 1: Render vehicle as box ===
        if let Some((pos, rot)) = self.vehicle_transform {
            let model_matrix = rot.to_homogeneous().prepend_translation(&pos);

            // Задача 2: Использовать vehicle_shader если доступен
            if let Some(ref vs) = self.vehicle_shader {
                vs.bind(&self.gl);
                unsafe {
                    if let Some(u_model) = self.gl.get_uniform_location(vs.program(), "u_model") {
                        self.gl.uniform_matrix_4_f32_slice(Some(&u_model), false, model_matrix.as_slice());
                    }
                    if let Some(u_color) = self.gl.get_uniform_location(vs.program(), "u_color") {
                        // Ржавый металл цвет
                        self.gl.uniform_4_f32(Some(&u_color), 0.8, 0.3, 0.1, 1.0);
                    }
                }
            } else {
                self.shader.bind(&self.gl);
                unsafe {
                    if let Some(u_model) = self.gl.get_uniform_location(self.shader.program(), "u_model") {
                        self.gl.uniform_matrix_4_f32_slice(Some(&u_model), false, model_matrix.as_slice());
                    }
                }
            }

            if let Some(ref box_mesh) = self.vehicle_box_mesh {
                box_mesh.draw(&self.gl);
            }
        }
        
        // Render each visible object using appropriate LOD model
        // TODO: Fix LOD rendering - vertices type mismatch
        // for (_index, lod_model) in visible_objects {
        //     match lod_model {
        //         crate::graphics::lod_system::LodModel::HighPoly { vertices, indices } => {
        //             let mesh = Mesh::new(&self.gl, &vertices, &indices)?;
        //             mesh.draw();
        //         },
        //         crate::graphics::lod_system::LodModel::MediumPoly { vertices, indices } => {
        //             let mesh = Mesh::new(&self.gl, &vertices, &indices)?;
        //             mesh.draw();
        //         },
        //         crate::graphics::lod_system::LodModel::LowPoly { vertices, indices } => {
        //             let mesh = Mesh::new(&self.gl, &vertices, &indices)?;
        //             mesh.draw();
        //         },
        //         crate::graphics::lod_system::LodModel::Billboard { texture_id, size } => {
        //             // Skip billboards for now
        //         },
        //     }
        // }
        
        // Also render models from the traditional model system
        for (_, model) in &self.models {
            for mesh in &model.meshes {
                mesh.draw(&self.gl);
            }
        }
        
        // === SPRINT 2: Render HUD ===
        // HUD рисуется после основной сцены, без depth test
        self.render_hud()?;
        
        // Ввод-2: Рендерить оверлей паузы если в режиме Paused
        if self.menu_state == MenuState::Paused {
            self.render_pause_overlay()?;
        }
        
        Ok(())
    }
    
    /// Ввод-2: Оверлей паузы
    fn render_pause_overlay(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        unsafe {
            self.gl.disable(glow::DEPTH_TEST);
            
            let w = self.width as f32;
            let h = self.height as f32;
            
            // Полупрозрачный фон
            self.draw_rect(0.0, 0.0, w, h, [0.0, 0.0, 0.0, 0.5]);
            
            // Центральная панель
            self.draw_rect(w/2.0 - 150.0, h/2.0 - 100.0, 300.0, 200.0, [0.1, 0.1, 0.15, 0.95]);
            
            // Кнопка "Продолжить" (зелёная)
            self.draw_rect(w/2.0 - 120.0, h/2.0 - 40.0, 240.0, 40.0, [0.2, 0.6, 0.2, 0.8]);
            
            // Кнопка "Настройки" (серая)
            self.draw_rect(w/2.0 - 120.0, h/2.0 + 10.0, 240.0, 40.0, [0.3, 0.3, 0.3, 0.8]);
            
            // Кнопка "Выход в меню" (красная)
            self.draw_rect(w/2.0 - 120.0, h/2.0 + 60.0, 240.0, 40.0, [0.6, 0.2, 0.2, 0.8]);

            self.gl.enable(glow::DEPTH_TEST);
        }
        Ok(())
    }

    /// Задача 3: Рендерить небо (gradient quad)
    pub fn render_sky(&self) -> Result<(), Box<dyn std::error::Error>> {
        unsafe {
            self.gl.disable(glow::DEPTH_TEST);

            // Исп-2: Использовать sky_shader для рендеринга неба
            if let Some(ref ss) = self.sky_shader {
                ss.bind(&self.gl);
            } else {
                self.gl.enable(glow::DEPTH_TEST);
                return Ok(());
            }

            if let Some(vao) = self.sky_vao {
                self.gl.bind_vertex_array(Some(vao));
                self.gl.draw_arrays(glow::TRIANGLES, 0, 6);
            }

            // Вернуть основной шейдер
            self.shader.bind(&self.gl);

            self.gl.enable(glow::DEPTH_TEST);
        }
        Ok(())
    }
    
    /// Render HUD overlay (2D UI without depth test)
    pub fn render_hud(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        use crate::ui::hud::HudFlashElement;
        
        // Get HUD data from renderer's stored data (set by engine via set_hud_data)
        let hud_data = self.hud_data.clone().unwrap_or_else(|| crate::ui::hud::VehicleHudData {
            speed_kmh: self.vehicle_transform
                .map(|(_, _)| 65.0)  // placeholder if no data
                .unwrap_or(0.0),
            engine_rpm: 2200.0,
            engine_rpm_max: 3200.0,
            gear: crate::ui::hud::GearState::Drive(4),
            engine_running: true,
            fuel_level: 0.75,
            ..Default::default()
        });
        
        unsafe {
            // Disable depth test for 2D UI
            self.gl.disable(glow::DEPTH_TEST);
            
            // Use simple color for now (will use shader later)
            self.gl.use_program(Some(self.shader.program()));
            
            // Draw speed panel (bottom left rectangle)
            self.draw_rect(10.0, self.height as f32 - 60.0, 200.0, 50.0, [0.1, 0.1, 0.1, 0.8]);
            
            // Draw speed value (simple representation)
            let speed_text = format!("{:.0} km/h", hud_data.speed_kmh);
            // Text rendering will be added later with bitmap font
            
            // Draw RPM bar
            let rpm_ratio = (hud_data.engine_rpm / hud_data.engine_rpm_max).min(1.0);
            let bar_width = 150.0 * rpm_ratio;
            self.draw_rect(20.0, self.height as f32 - 40.0, bar_width, 10.0, [0.2, 0.8, 0.2, 1.0]);
            
            // Draw fuel bar
            let fuel_width = 100.0 * hud_data.fuel_level;
            self.draw_rect(20.0, self.height as f32 - 25.0, fuel_width, 8.0, [0.8, 0.8, 0.2, 1.0]);
            
            // Draw wheel contact indicators (4 dots)
            for (i, &contact) in hud_data.wheel_contact.iter().enumerate() {
                let x = 250.0 + (i as f32 * 20.0);
                let y = self.height as f32 - 40.0;
                let color = if contact { [0.0, 1.0, 0.0, 1.0] } else { [1.0, 0.0, 0.0, 1.0] };
                // Using small rect instead of circle for simplicity
                self.draw_rect(x - 6.0, y - 6.0, 12.0, 12.0, color);
            }
            
            // Flash warning for low fuel
            if hud_data.fuel_reserve {
                self.draw_rect(150.0, self.height as f32 - 25.0, 100.0, 8.0, [1.0, 0.0, 0.0, 1.0]);
            }
            
            // Граф-3: Мини-карта в правом верхнем углу
            self.render_minimap(&hud_data);
            
            // Re-enable depth test
            self.gl.enable(glow::DEPTH_TEST);
        }
        
        Ok(())
    }

    /// Draw a 2D rectangle (simple quad) with proper VAO/VBO implementation
    pub unsafe fn draw_rect(&self, x: f32, y: f32, width: f32, height: f32, color: [f32; 4]) {
        // Create orthographic projection for UI
        let ortho = Matrix4::new_orthographic(
            0.0, self.width as f32,
            0.0, self.height as f32,
            -1.0, 1.0
        );
        
        // Set up vertices for a quad (2 triangles)
        let vertices: [f32; 8] = [
            x, y,                    // bottom-left
            x + width, y,            // bottom-right
            x + width, y + height,   // top-right
            x, y + height,           // top-left
        ];
        
        let indices: [u32; 6] = [
            0, 1, 2,
            0, 2, 3,
        ];
        
        // Create temporary VAO/VBO for the rect
        let vao = match self.gl.create_vertex_array() {
            Ok(v) => v,
            Err(_) => return,
        };
        let vbo = match self.gl.create_buffer() {
            Ok(v) => v,
            Err(_) => { self.gl.delete_vertex_array(vao); return; },
        };
        let ebo = match self.gl.create_buffer() {
            Ok(v) => v,
            Err(_) => { self.gl.delete_vertex_array(vao); self.gl.delete_buffer(vbo); return; },
        };
        
        self.gl.bind_vertex_array(Some(vao));
        
        self.gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
        self.gl.buffer_data_u8_slice(
            glow::ARRAY_BUFFER,
            bytemuck::cast_slice(&vertices),
            glow::STREAM_DRAW,
        );
        
        self.gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(ebo));
        self.gl.buffer_data_u8_slice(
            glow::ELEMENT_ARRAY_BUFFER,
            bytemuck::cast_slice(&indices),
            glow::STREAM_DRAW,
        );
        
        // Position attribute (location 0) - 2 floats per vertex
        self.gl.enable_vertex_attrib_array(0);
        self.gl.vertex_attrib_pointer_f32(0, 2, glow::FLOAT, false, 8, 0);
        
        // Исп-1: Pass color and uniforms to shader before drawing
        if let Some(u) = self.gl.get_uniform_location(self.shader.program(), "u_use_solid_color") {
            self.gl.uniform_1_i32(Some(&u), 1); // enable solid color mode
        }
        if let Some(u) = self.gl.get_uniform_location(self.shader.program(), "u_color") {
            self.gl.uniform_4_f32(Some(&u), color[0], color[1], color[2], color[3]);
        }
        if let Some(u) = self.gl.get_uniform_location(self.shader.program(), "u_projection") {
            self.gl.uniform_matrix_4_f32_slice(Some(&u), false, ortho.as_slice());
        }
        
        // Draw the quad
        self.gl.draw_elements(glow::TRIANGLES, 6, glow::UNSIGNED_INT, 0);
        
        // Исп-1: Reset to terrain mode after drawing
        if let Some(u) = self.gl.get_uniform_location(self.shader.program(), "u_use_solid_color") {
            self.gl.uniform_1_i32(Some(&u), 0); // back to terrain mode
        }
        
        // Cleanup
        self.gl.delete_vertex_array(vao);
        self.gl.delete_buffer(vbo);
        self.gl.delete_buffer(ebo);
    }

    /// Draw a 2D rectangle border
    pub unsafe fn draw_rect_border(&self, x: f32, y: f32, width: f32, height: f32, thickness: f32, color: [f32; 4]) {
        // Top
        self.draw_rect(x, y, width, thickness, color);
        // Bottom
        self.draw_rect(x, y + height - thickness, width, thickness, color);
        // Left
        self.draw_rect(x, y, thickness, height, color);
        // Right
        self.draw_rect(x + width - thickness, y, thickness, height, color);
    }

    /// Draw a 2D triangle (for minimap player icon)
    unsafe fn draw_triangle(&self, x1: f32, y1: f32, x2: f32, y2: f32, x3: f32, y3: f32, color: [f32; 4]) {
        let ortho = Matrix4::new_orthographic(
            0.0, self.width as f32,
            0.0, self.height as f32,
            -1.0, 1.0
        );
        
        let vertices: [f32; 6] = [x1, y1, x2, y2, x3, y3];
        
        let vao = match self.gl.create_vertex_array() {
            Ok(v) => v,
            Err(_) => return,
        };
        let vbo = match self.gl.create_buffer() {
            Ok(v) => v,
            Err(_) => { self.gl.delete_vertex_array(vao); return; },
        };
        
        self.gl.bind_vertex_array(Some(vao));
        self.gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
        self.gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, bytemuck::cast_slice(&vertices), glow::STREAM_DRAW);
        self.gl.enable_vertex_attrib_array(0);
        self.gl.vertex_attrib_pointer_f32(0, 2, glow::FLOAT, false, 8, 0);
        
        if let Some(u) = self.gl.get_uniform_location(self.shader.program(), "u_use_solid_color") {
            self.gl.uniform_1_i32(Some(&u), 1);
        }
        if let Some(u) = self.gl.get_uniform_location(self.shader.program(), "u_color") {
            self.gl.uniform_4_f32(Some(&u), color[0], color[1], color[2], color[3]);
        }
        if let Some(u) = self.gl.get_uniform_location(self.shader.program(), "u_projection") {
            self.gl.uniform_matrix_4_f32_slice(Some(&u), false, ortho.as_slice());
        }
        
        self.gl.draw_arrays(glow::TRIANGLES, 0, 3);
        
        if let Some(u) = self.gl.get_uniform_location(self.shader.program(), "u_use_solid_color") {
            self.gl.uniform_1_i32(Some(&u), 0);
        }
        
        self.gl.delete_vertex_array(vao);
        self.gl.delete_buffer(vbo);
    }
    
    /// Граф-3: Рендеринг мини-карты
    fn render_minimap(&mut self, hud_data: &crate::ui::hud::VehicleHudData) {
        let map_size = 128.0;
        let margin = 10.0;
        let x = self.width as f32 - map_size - margin;
        let y = self.height as f32 - map_size - margin;
        
        unsafe {
            // Рамка мини-карты
            self.draw_rect(x - 2.0, y - 2.0, map_size + 4.0, map_size + 4.0, [0.0, 0.0, 0.0, 0.8]);
            
            // Фон (условная земля)
            self.draw_rect(x, y, map_size, map_size, [0.2, 0.3, 0.2, 1.0]);
            
            // Иконка игрока (треугольник по центру)
            let cx = x + map_size / 2.0;
            let cy = y + map_size / 2.0;
            let icon_size = 8.0;
            self.draw_triangle(
                cx, cy - icon_size,
                cx - icon_size / 2.0, cy + icon_size / 2.0,
                cx + icon_size / 2.0, cy + icon_size / 2.0,
                [1.0, 1.0, 0.0, 1.0],
            );
            
            // Если есть данные о грузе - показать маркер
            if hud_data.cargo_attached {
                self.draw_rect(x + map_size - 20.0, y + 5.0, 10.0, 10.0, [0.0, 1.0, 1.0, 1.0]);
            }
        }
    }

    /// Граф-1: Draw text using bitmap font
    pub unsafe fn draw_text(&mut self, text: &str, x: f32, y: f32, size: f32, color: [f32; 4]) {
        let char_size = size; // 8x8 scaled
        let mut cursor_x = x;

        // Bind font texture
        if let Some(ref tex) = self.font_texture {
            tex.bind(&self.gl);
        }
        
        // Use shader with texturing mode
        if let Some(u) = self.gl.get_uniform_location(self.shader.program(), "u_use_solid_color") {
            self.gl.uniform_1_i32(Some(&u), 0); // use texture mode
        }
        if let Some(u) = self.gl.get_uniform_location(self.shader.program(), "u_use_texture") {
            self.gl.uniform_1_i32(Some(&u), 1);
        }
        
        for ch in text.chars() {
            if let Some(uv) = self.font_chars.get(&ch) {
                let [u, v, w, h] = *uv;
                
                // Draw textured quad for this character
                let vertices: [f32; 32] = [
                    // pos (2) + color (4) + uv (2) = 8 floats per vertex, 4 vertices
                    cursor_x, y + char_size,       color[0], color[1], color[2], color[3],   u, v + h,
                    cursor_x + char_size, y + char_size,  color[0], color[1], color[2], color[3],   u + w, v + h,
                    cursor_x + char_size, y,              color[0], color[1], color[2], color[3],   u + w, v,
                    cursor_x, y,                          color[0], color[1], color[2], color[3],   u, v,
                ];
                
                let indices: [u32; 6] = [0, 1, 2, 0, 2, 3];
                
                let vao = self.gl.create_vertex_array().ok();
                let vbo = self.gl.create_buffer().ok();
                let ebo = self.gl.create_buffer().ok();
                
                if let (Some(vao), Some(vbo), Some(ebo)) = (vao, vbo, ebo) {
                    self.gl.bind_vertex_array(Some(vao));
                    self.gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
                    self.gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, bytemuck::cast_slice(&vertices), glow::STREAM_DRAW);
                    self.gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(ebo));
                    self.gl.buffer_data_u8_slice(glow::ELEMENT_ARRAY_BUFFER, bytemuck::cast_slice(&indices), glow::STREAM_DRAW);
                    
                    // pos: loc 0, 2 floats
                    self.gl.enable_vertex_attrib_array(0);
                    self.gl.vertex_attrib_pointer_f32(0, 2, glow::FLOAT, false, 32, 0);
                    // color: loc 1, 4 floats
                    self.gl.enable_vertex_attrib_array(1);
                    self.gl.vertex_attrib_pointer_f32(1, 4, glow::FLOAT, false, 32, 8);
                    // uv: loc 2, 2 floats
                    self.gl.enable_vertex_attrib_array(2);
                    self.gl.vertex_attrib_pointer_f32(2, 2, glow::FLOAT, false, 32, 24);
                    
                    self.gl.draw_elements(glow::TRIANGLES, 6, glow::UNSIGNED_INT, 0);
                    
                    self.gl.delete_vertex_array(vao);
                    self.gl.delete_buffer(vbo);
                    self.gl.delete_buffer(ebo);
                }
                
                cursor_x += char_size;
            } else if ch == ' ' {
                cursor_x += char_size;
            }
        }
        
        // Reset shader state
        if let Some(u) = self.gl.get_uniform_location(self.shader.program(), "u_use_texture") {
            self.gl.uniform_1_i32(Some(&u), 0);
        }
    }
    
    /// Get renderer width
    pub fn get_width(&self) -> u32 {
        self.width
    }
    
    /// Get renderer height
    pub fn get_height(&self) -> u32 {
        self.height
    }
    
    fn render_world_creation(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        unsafe {
            self.gl.disable(glow::DEPTH_TEST);
            self.gl.clear_color(0.05, 0.05, 0.1, 1.0);
            self.gl.clear(glow::COLOR_BUFFER_BIT);
            let w = self.width as f32;
            let h = self.height as f32;
            // Панель создания мира
            self.draw_rect(w/2.0 - 200.0, h/2.0 - 150.0, 400.0, 300.0, [0.1, 0.1, 0.15, 0.9]);
            self.gl.enable(glow::DEPTH_TEST);
        }
        Ok(())
    }
    
    fn render_settings(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        unsafe {
            self.gl.disable(glow::DEPTH_TEST);
            self.gl.clear_color(0.05, 0.05, 0.1, 1.0);
            self.gl.clear(glow::COLOR_BUFFER_BIT);
            let w = self.width as f32;
            let h = self.height as f32;
            // Панель настроек
            self.draw_rect(w/2.0 - 200.0, h/2.0 - 150.0, 400.0, 300.0, [0.1, 0.1, 0.15, 0.9]);
            self.gl.enable(glow::DEPTH_TEST);
        }
        Ok(())
    }
    
    pub fn load_model(&mut self, name: String, model: Model) {
        self.models.insert(name, model);
    }

    pub fn render_model(&self, model_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(model) = self.models.get(model_name) {
            self.shader.bind(&self.gl);
            
            let projection = self.camera.projection_matrix();
            let view = self.camera.view_matrix();
            
            unsafe {
                // Set uniforms with safe handling - skip if uniform not found
                if let Some(u_projection) = self.gl.get_uniform_location(self.shader.program(), "u_projection") {
                    self.gl.uniform_matrix_4_f32_slice(Some(&u_projection), false, projection.as_slice());
                }
                if let Some(u_view) = self.gl.get_uniform_location(self.shader.program(), "u_view") {
                    self.gl.uniform_matrix_4_f32_slice(Some(&u_view), false, view.as_slice());
                }
            }

            for mesh in &model.meshes {
                mesh.draw(&self.gl);
            }
        }

        Ok(())
    }

    pub fn set_camera(&mut self, camera: Camera) {
        self.camera = camera;
    }

    pub fn next_city(&mut self) {
        self.current_city_index = (self.current_city_index + 1) % 14; // 14 Siberian cities
    }

    pub fn prev_city(&mut self) {
        if self.current_city_index == 0 {
            self.current_city_index = 13;
        } else {
            self.current_city_index -= 1;
        }
    }
}