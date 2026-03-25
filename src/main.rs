// RTGC-0.7 Main Entry Point - Полноценный OpenGL рендеринг
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use winit::{
    application::ApplicationHandler,
    event::{ElementState, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::WindowAttributes,
};
use glow::HasContext;
use std::time::Instant;
use tracing::{error, warn, info};

use rtgc::graphics::GlContext;
use rtgc::config::{Config, GraphicsConfig};

struct GlResources {
    vao: glow::VertexArray,
    vbo: glow::Buffer,
    program: glow::Program,
    rotation_loc: Option<glow::UniformLocation>,
    color_loc: Option<glow::UniformLocation>,
}

impl Drop for GlResources {
    fn drop(&mut self) {
        // Ресурсы будут удалены в cleanup, если контекст доступен
    }
}

struct App {
    gl_context: Option<GlContext>,
    gl_resources: Option<GlResources>,
    last_frame_time: Instant,
    target_frame_time: f32,
    rotation: f32,
    frame_count: u32,
    fps_timer: Instant,
    current_fps: u32,
    config: Config,
}

impl App {
    fn new() -> Self {
        // Загружаем конфиг, если существует, иначе используем дефолтный
        let config = Config::load("config.json").unwrap_or_else(|_| {
            warn!("Config file not found or invalid, using default config");
            Config::default()
        });
        
        let target_fps = if config.graphics.vsync {
            // При vsync целевой FPS зависит от частоты обновления монитора
            60.0
        } else {
            // Без vsync используем максимальный FPS или из конфига
            config.graphics.max_fps.unwrap_or(120.0) as f32
        };
        
        Self {
            gl_context: None,
            gl_resources: None,
            last_frame_time: Instant::now(),
            target_frame_time: 1.0 / target_fps,
            rotation: 0.0,
            frame_count: 0,
            fps_timer: Instant::now(),
            current_fps: 0,
            config,
        }
    }

    fn init_gl(&mut self, event_loop: &ActiveEventLoop) -> Result<(), Box<dyn std::error::Error>> {
        let graphics = &self.config.graphics;
        
        let window_attrs = WindowAttributes::default()
            .with_title("RTGC-0.7 - Russian Technological Game | Escape - выход")
            .with_inner_size(winit::dpi::LogicalSize::new(
                graphics.window_width as f64,
                graphics.window_height as f64,
            ))
            .with_resizable(true)
            .with_decorations(!graphics.fullscreen)
            .with_active(true);

        let gl_context = GlContext::new(event_loop, window_attrs)?;

        // Инициализируем OpenGL состояния
        let gl = &gl_context.gl;
        unsafe {
            gl.enable(glow::DEPTH_TEST);
            gl.enable(glow::BLEND);
            gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);
            gl.clear_color(0.1, 0.1, 0.15, 1.0);
            gl.viewport(0, 0, graphics.window_width as i32, graphics.window_height as i32);
        }

        // Создаем ресурсы один раз при инициализации
        let resources = self.create_gl_resources(&gl_context.gl)?;
        
        self.gl_context = Some(gl_context);
        self.gl_resources = Some(resources);
        Ok(())
    }

    /// Создает OpenGL ресурсы (VAO, VBO, шейдеры) - вызывается один раз при инициализации
    fn create_gl_resources(&self, gl: &glow::Context) -> Result<GlResources, Box<dyn std::error::Error>> {
        let vertices: [f32; 9] = [
            0.0, 0.5, 0.0,
            -0.5, -0.5, 0.0,
            0.5, -0.5, 0.0,
        ];

        // Создаем VAO и VBO
        let vao = gl.create_vertex_array()
            .ok_or("Failed to create vertex array")?;
        let vbo = gl.create_buffer()
            .ok_or("Failed to create buffer")?;

        unsafe {
            gl.bind_vertex_array(Some(vao));
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
            gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, bytemuck::cast_slice(&vertices), glow::STATIC_DRAW);
            gl.enable_vertex_attrib_array(0);
            gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, 0, 0);
            gl.bind_vertex_array(None);
            gl.bind_buffer(glow::ARRAY_BUFFER, None);
        }

        // Компилируем шейдеры
        let vert_src = r#"
            #version 330 core
            layout(location = 0) in vec3 aPos;
            uniform float u_rotation;

            void main() {
                float c = cos(u_rotation);
                float s = sin(u_rotation);
                mat2 rot = mat2(c, -s, s, c);
                vec2 pos = rot * aPos.xy;
                gl_Position = vec4(pos, aPos.z, 1.0);
            }
        "#;

        let frag_src = r#"
            #version 330 core
            out vec4 FragColor;
            uniform vec3 u_color;

            void main() {
                FragColor = vec4(u_color, 1.0);
            }
        "#;

        let vert_shader = gl.create_shader(glow::VERTEX_SHADER)
            .ok_or("Failed to create vertex shader")?;
        let frag_shader = gl.create_shader(glow::FRAGMENT_SHADER)
            .ok_or("Failed to create fragment shader")?;

        unsafe {
            gl.shader_source(vert_shader, vert_src);
            gl.compile_shader(vert_shader);

            if !gl.get_shader_compile_status(vert_shader) {
                let log = gl.get_shader_info_log(vert_shader);
                gl.delete_shader(vert_shader);
                return Err(format!("Vertex shader compilation error: {}", log).into());
            }

            gl.shader_source(frag_shader, frag_src);
            gl.compile_shader(frag_shader);

            if !gl.get_shader_compile_status(frag_shader) {
                let log = gl.get_shader_info_log(frag_shader);
                gl.delete_shader(vert_shader);
                gl.delete_shader(frag_shader);
                return Err(format!("Fragment shader compilation error: {}", log).into());
            }

            let program = gl.create_program()
                .ok_or("Failed to create program")?;
            gl.attach_shader(program, vert_shader);
            gl.attach_shader(program, frag_shader);
            gl.link_program(program);

            if !gl.get_program_link_status(program) {
                let log = gl.get_program_info_log(program);
                gl.delete_program(program);
                gl.delete_shader(vert_shader);
                gl.delete_shader(frag_shader);
                return Err(format!("Program link error: {}", log).into());
            }

            // Шейдеры можно удалить после линковки - программа их копирует
            gl.detach_shader(program, vert_shader);
            gl.detach_shader(program, frag_shader);
            gl.delete_shader(vert_shader);
            gl.delete_shader(frag_shader);

            gl.use_program(Some(program));
        }

        let rotation_loc = unsafe { gl.get_uniform_location(program, "u_rotation") };
        let color_loc = unsafe { gl.get_uniform_location(program, "u_color") };

        Ok(GlResources {
            vao,
            vbo,
            program,
            rotation_loc,
            color_loc,
        })
    }

    /// Освобождает OpenGL ресурсы
    fn cleanup_gl_resources(&mut self) {
        if let (Some(resources), Some(gl_context)) = (self.gl_resources.take(), &self.gl_context) {
            let gl = &gl_context.gl;
            unsafe {
                gl.delete_vertex_array(resources.vao);
                gl.delete_buffer(resources.vbo);
                gl.delete_program(resources.program);
            }
        }
    }

    fn render(&mut self) {
        let (gl_context, resources) = match (&mut self.gl_context, &self.gl_resources) {
            (Some(ctx), Some(res)) => (ctx, res),
            _ => {
                error!("Render called but GL context or resources not initialized");
                return;
            }
        };
        
        let gl = &gl_context.gl;

        // Проверка ошибок OpenGL перед рендерингом (для отладки)
        #[cfg(debug_assertions)]
        unsafe {
            let err = gl.get_error();
            if err != glow::NO_ERROR {
                warn!("OpenGL error before render: {:?}", err);
            }
        }

        unsafe {
            gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
            gl.bind_vertex_array(Some(resources.vao));
            gl.use_program(Some(resources.program));

            // Вычисляем цвет на основе вращения
            let r = ((self.rotation.sin() + 1.0) * 0.5) as f32;
            let g = ((self.rotation.cos() + 1.0) * 0.5) as f32;
            let b = (((self.rotation * 0.5).sin() + 1.0) * 0.5) as f32;

            // Обновляем uniforms
            gl.uniform_1_f32(resources.rotation_loc.as_ref(), self.rotation);
            gl.uniform_3_f32(resources.color_loc.as_ref(), r, g, b);

            // Рисуем треугольник
            gl.draw_arrays(glow::TRIANGLES, 0, 3);
            
            gl.bind_vertex_array(None);
            gl.use_program(None);
        }

        // Проверка ошибок после рендеринга
        #[cfg(debug_assertions)]
        unsafe {
            let err = gl.get_error();
            if err != glow::NO_ERROR {
                warn!("OpenGL error after render: {:?}", err);
            }
        }

        let _ = gl_context.swap_buffers();
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.gl_context.is_none() {
            match self.init_gl(event_loop) {
                Ok(()) => {
                    info!("GL context initialized successfully");
                    if let Some(w) = self.gl_context.as_ref().map(|c| &c.window) {
                        w.request_redraw();
                    }
                }
                Err(e) => {
                    error!("Критическая ошибка инициализации GL: {}", e);
                    error!("Причина: OpenGL контекст не создан. Проверьте драйверы видеокарты.");
                    event_loop.exit();
                }
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: winit::window::WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                self.cleanup_gl_resources();
                event_loop.exit();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if event.state == ElementState::Pressed {
                    if let PhysicalKey::Code(key_code) = event.physical_key {
                        match key_code {
                            KeyCode::Escape | KeyCode::F4 => {
                                self.cleanup_gl_resources();
                                event_loop.exit();
                            }
                            KeyCode::KeyR => {
                                self.rotation = 0.0;
                                if let Some(w) = self.gl_context.as_ref().map(|c| &c.window) {
                                    w.set_title("RTGC-0.7 - Сброшено!");
                                    w.request_redraw();
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
            WindowEvent::Resized(physical_size) => {
                if let Some(ref mut ctx) = self.gl_context {
                    let _ = ctx.resize(physical_size.width, physical_size.height);
                    unsafe {
                        ctx.gl.viewport(0, 0, physical_size.width as i32, physical_size.height as i32);
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                let now = Instant::now();
                let elapsed = now.duration_since(self.last_frame_time).as_secs_f32();
                
                if elapsed >= self.target_frame_time {
                    // Используем delta time для плавного вращения независимо от FPS
                    let delta_time = elapsed.min(0.1); // Ограничиваем delta time чтобы избежать скачков
                    self.last_frame_time = now;
                    self.frame_count += 1;
                    
                    // Вращение теперь зависит от времени (радиан в секунду)
                    const ROTATION_SPEED: f32 = 1.0; // 1 радиан в секунду
                    self.rotation += ROTATION_SPEED * delta_time;

                    self.render();

                    // Обновляем FPS счетчик
                    let fps_elapsed = now.duration_since(self.fps_timer).as_secs_f32();
                    if fps_elapsed >= 0.5 {
                        self.current_fps = (self.frame_count as f32 / fps_elapsed) as u32;
                        self.frame_count = 0;
                        self.fps_timer = now;
                        
                        if let Some(w) = self.gl_context.as_ref().map(|c| &c.window) {
                            w.set_title(&format!(
                                "RTGC-0.7 | OpenGL | FPS: {} | Config: {}x{} | Escape - выход",
                                self.current_fps,
                                self.config.graphics.window_width,
                                self.config.graphics.window_height
                            ));
                        }
                    }

                    if let Some(w) = self.gl_context.as_ref().map(|c| &c.window) {
                        if w.has_focus() {
                            w.request_redraw();
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = App::new();
    event_loop.run_app(&mut app)?;

    Ok(())
}
