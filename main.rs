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

use rtgc::graphics::GlContext;

struct App {
    gl_context: Option<GlContext>,
    last_frame_time: Instant,
    target_frame_time: f32,
    rotation: f32,
    frame_count: u32,
}

impl App {
    fn new() -> Self {
        Self {
            gl_context: None,
            last_frame_time: Instant::now(),
            target_frame_time: 1.0 / 60.0,
            rotation: 0.0,
            frame_count: 0,
        }
    }

    fn init_gl(&mut self, event_loop: &ActiveEventLoop) -> Result<(), Box<dyn std::error::Error>> {
        let window_attrs = WindowAttributes::default()
            .with_title("RTGC-0.7 - Russian Technological Game | Escape - выход")
            .with_inner_size(winit::dpi::LogicalSize::new(1280.0, 720.0))
            .with_resizable(true)
            .with_decorations(true)
            .with_active(true);

        let gl_context = GlContext::new(event_loop, window_attrs)?;

        // Инициализируем OpenGL состояния
        let gl = &gl_context.gl;
        unsafe {
            gl.enable(glow::DEPTH_TEST);
            gl.enable(glow::BLEND);
            gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);
            gl.clear_color(0.1, 0.1, 0.15, 1.0);
        }

        self.gl_context = Some(gl_context);
        Ok(())
    }

    fn render(&mut self) {
        let Some(gl_context) = &mut self.gl_context else { return };
        let gl = &gl_context.gl;

        unsafe {
            gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);

            // Рендерим треугольник
            let vertices: [f32; 9] = [
                0.0, 0.5, 0.0,
                -0.5, -0.5, 0.0,
                0.5, -0.5, 0.0,
            ];

            let vao = gl.create_vertex_array().ok();
            let vbo = gl.create_buffer().ok();

            if let (Some(vao), Some(vbo)) = (vao, vbo) {
                gl.bind_vertex_array(Some(vao));
                gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
                gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, bytemuck::cast_slice(&vertices), glow::STATIC_DRAW);
                gl.enable_vertex_attrib_array(0);
                gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, 0, 0);

                // Простой шейдер
                let shader_src = r#"
                    #version 330 core
                    layout(location = 0) in vec3 aPos;
                    uniform float u_rotation;
                    uniform vec3 u_color;
                    
                    void main() {
                        float c = cos(u_rotation);
                        float s = sin(u_rotation);
                        mat2 rot = mat2(c, -s, s, c);
                        vec2 pos = rot * aPos.xy;
                        gl_Position = vec4(pos, aPos.z, 1.0);
                    }
                "#;

                let shader = gl.create_shader(glow::VERTEX_SHADER).ok();
                if let Some(shader) = shader {
                    gl.shader_source(shader, shader_src);
                    gl.compile_shader(shader);
                    
                    let program = gl.create_program().ok();
                    if let Some(program) = program {
                        gl.attach_shader(program, shader);
                        gl.link_program(program);
                        gl.use_program(Some(program));

                        let rotation_loc = gl.get_uniform_location(program, "u_rotation");
                        let color_loc = gl.get_uniform_location(program, "u_color");
                        
                        let r = ((self.rotation.sin() + 1.0) * 0.5) as f32;
                        let g = ((self.rotation.cos() + 1.0) * 0.5) as f32;
                        let b = (((self.rotation * 0.5).sin() + 1.0) * 0.5) as f32;
                        
                        gl.uniform_1_f32(rotation_loc.as_ref(), self.rotation);
                        gl.uniform_3_f32(color_loc.as_ref(), r, g, b);

                        gl.draw_arrays(glow::TRIANGLES, 0, 3);
                    }
                }

                gl.delete_vertex_array(vao);
                gl.delete_buffer(vbo);
            }
        }

        let _ = gl_context.swap_buffers();
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.gl_context.is_none() {
            if let Err(e) = self.init_gl(event_loop) {
                eprintln!("Ошибка инициализации GL: {}", e);
                event_loop.exit();
            } else if let Some(w) = self.gl_context.as_ref().map(|c| &c.window) {
                w.request_redraw();
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: winit::window::WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if event.state == ElementState::Pressed {
                    if let PhysicalKey::Code(key_code) = event.physical_key {
                        match key_code {
                            KeyCode::Escape | KeyCode::F4 => {
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
                    self.last_frame_time = now;
                    self.frame_count += 1;
                    self.rotation += 0.02;

                    self.render();

                    // Обновляем FPS в заголовке каждую секунду
                    if self.frame_count % 60 == 0 {
                        if let Some(w) = self.gl_context.as_ref().map(|c| &c.window) {
                            w.set_title(&format!(
                                "RTGC-0.7 | OpenGL | FPS: {} | Escape - выход",
                                (1.0 / elapsed).round() as u32
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
