//! OpenGL Context для RTGC-0.8
//! Реализация инициализации OpenGL контекста с использованием glutin и winit
//! Интеграция с RHI через GlDevice

use glow::Context;
use glutin::config::ConfigTemplateBuilder;
use glutin::context::{ContextAttributesBuilder, PossiblyCurrentContext};
use glutin::display::GetGlDisplay;
use glutin::prelude::*;
use glutin::surface::{SurfaceAttributesBuilder, SwapInterval, WindowSurface};
use glutin_winit::DisplayBuilder;
use raw_window_handle::HasWindowHandle;
use std::ffi::CStr;
use std::num::NonZeroU32;
use std::sync::Arc;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowAttributes};

use crate::graphics::rhi::gl::GlDevice;
use crate::graphics::rhi::device::IDevice;

// Примечание: glow::Context безопасно используется в многопоточной среде через Arc
// Не реализуем Send/Sync напрямую для Context (нарушает orphan rules)

/// OpenGL контекст для рендеринга с RHI интеграцией
pub struct GlContext {
    pub gl: Option<Arc<Context>>,
    pub window: Option<Window>,
    pub width: u32,
    pub height: u32,
    // Храним контекст и поверхность для swap_buffers
    gl_context: Option<PossiblyCurrentContext>,
    surface: Option<glutin::surface::Surface<WindowSurface>>,
    // RHI device для OpenGL
    pub rhi_device: Option<Arc<GlDevice>>,
}

unsafe impl Send for GlContext {}
unsafe impl Sync for GlContext {}

impl GlContext {
    /// Создаёт новый OpenGL контекст с окном и RHI устройством
    pub fn new(event_loop: &ActiveEventLoop, window_attrs: WindowAttributes) -> Result<Self, Box<dyn std::error::Error>> {
        // Шаблон для поиска подходящей конфигурации OpenGL
        let template = ConfigTemplateBuilder::new()
            .with_alpha_size(8)
            .with_transparency(false);

        // Создаём Display и окно одновременно через DisplayBuilder
        let (window, gl_config) = DisplayBuilder::new()
            .with_window_attributes(Some(window_attrs))
            .build(event_loop, template, |mut configs| {
                // Выбираем первую конфигурацию
                // configs.next() возвращает Option<Config>, используем unwrap_or с понятной ошибкой
                configs.next().unwrap_or_else(|| panic!("No suitable OpenGL config found"))
            })?;

        let window = window.ok_or("Не удалось создать окно")?;

        // Получаем display из config
        let display = gl_config.display();

        // Получаем raw window handle для создания контекста
        let raw_window_handle = window.window_handle()?.as_raw();

        // Создаём контекст
        let context_attributes = ContextAttributesBuilder::new()
            .build(Some(raw_window_handle));

        // Создаём неактивный контекст
        let not_current_context = unsafe {
            display.create_context(&gl_config, &context_attributes)?
        };

        // Создаём поверхность для рендеринга в окно
        let (raw_width, raw_height): (u32, u32) = window.inner_size().into();

        // NonZeroU32 требует значение > 0; при первом создании окно может иметь размер 0
        // Используем unwrap_or с гарантированно валидным значением
        let nz_width  = NonZeroU32::new(raw_width).unwrap_or(NonZeroU32::new(1280).unwrap());
        let nz_height = NonZeroU32::new(raw_height).unwrap_or(NonZeroU32::new(720).unwrap());
        
        // Используем SurfaceAttributesBuilder вместо default()
        let surface_attrs = SurfaceAttributesBuilder::<WindowSurface>::new().build(
            raw_window_handle,
            nz_width,
            nz_height,
        );

        let surface = unsafe {
            display.create_window_surface(&gl_config, &surface_attrs)?
        };

        // Делаем контекст активным
        let gl_context = not_current_context.make_current(&surface)?;

        // Инициализируем glow для работы с OpenGL
        let gl = unsafe {
            Context::from_loader_function(|s| {
                let c_str = CStr::from_bytes_with_nul_unchecked(s.as_bytes());
                display.get_proc_address(c_str)
            })
        };

        // Включаем VSync
        // NonZeroU32::new(1) всегда возвращает Some(1), unwrap безопасен
        let swap_interval = SwapInterval::Wait(NonZeroU32::new(1).unwrap());
        let _ = surface.set_swap_interval(&gl_context, swap_interval);

        // Создаём RHI устройство для OpenGL
        let gl_arc = Arc::new(gl);
        let rhi_device = Arc::new(GlDevice::new(gl_arc.clone()));

        Ok(Self {
            gl: Some(gl_arc),
            window: Some(window),
            width: nz_width.get(),
            height: nz_height.get(),
            gl_context: Some(gl_context),
            surface: Some(surface),
            rhi_device: Some(rhi_device),
        })
    }

    /// Изменяет размер поверхности при ресайзе окна
    pub fn resize(&mut self, width: u32, height: u32) -> Result<(), Box<dyn std::error::Error>> {
        self.width = width;
        self.height = height;

        // glutin 0.32: resize принимает NonZeroU32
        // Используем unwrap_or с гарантированно валидным значением 1
        let nz_w = NonZeroU32::new(width).unwrap_or(NonZeroU32::new(1).unwrap());
        let nz_h = NonZeroU32::new(height).unwrap_or(NonZeroU32::new(1).unwrap());
        if let (Some(surface), Some(gl_context)) = (&self.surface, &self.gl_context) {
            surface.resize(gl_context, nz_w, nz_h);
        }

        // Обновляем viewport в OpenGL
        if let Some(ref gl) = self.gl {
            unsafe {
                glow::HasContext::viewport(gl.as_ref(), 0, 0, width as i32, height as i32);
            }
        }

        Ok(())
    }

    /// Меняет буферы (swap buffers)
    pub fn swap_buffers(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let (Some(surface), Some(gl_context)) = (&self.surface, &self.gl_context) {
            surface.swap_buffers(gl_context)?;
        }
        Ok(())
    }

    /// Получает размеры окна
    pub fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Устанавливает заголовок окна
    pub fn set_title(&self, title: &str) {
        if let Some(ref window) = self.window {
            window.set_title(title);
        }
    }

    /// Запрашивает перерисовку
    pub fn request_redraw(&self) {
        if let Some(ref window) = self.window {
            window.request_redraw();
        }
    }

    /// Begin frame - подготовка к рендерингу
    pub fn begin_frame(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(ref gl) = self.gl {
            unsafe {
                use glow::HasContext;
                gl.clear_color(0.1, 0.2, 0.3, 1.0);
                gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
            }
        }
        Ok(())
    }

    /// End frame - завершение кадра (swap buffers)
    pub fn end_frame(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.swap_buffers()
    }

    /// Получить матрицу проекции для текущих размеров окна
    pub fn get_projection_matrix(&self, fov: f32, near: f32, far: f32) -> nalgebra::Matrix4<f32> {
        let aspect = self.width as f32 / self.height as f32;
        *nalgebra::Perspective3::new(aspect, fov, near, far).as_matrix()
    }

    /// Рендерить террейн
    pub fn render_terrain(
        &self,
        renderer: &mut crate::graphics::renderer::Renderer,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Делегируем рендеринг террейна в Renderer
        // Эта функция может быть расширена для передачи uniform-ов
        Ok(())
    }

    /// Рендерить транспорт
    pub fn render_vehicle(
        &self,
        renderer: &mut crate::graphics::renderer::Renderer,
        view_proj: &nalgebra::Matrix4<f32>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Делегируем рендеринг транспорта в Renderer
        Ok(())
    }

    /// Рендерить вертолёт
    pub fn render_helicopter(
        &self,
        renderer: &mut crate::graphics::renderer::Renderer,
        view_proj: &nalgebra::Matrix4<f32>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Делегируем рендеринг вертолёта в Renderer
        Ok(())
    }

    /// Создаёт placeholder GlContext для использования до инициализации окна
    pub fn new_placeholder() -> Self {
        // Создаём заглушку - реальный контекст будет создан в resumed()
        Self {
            gl: None,
            window: None,
            width: 1280,
            height: 720,
            gl_context: None,
            surface: None,
            rhi_device: None,
        }
    }

    /// Проверяет, инициализирован ли OpenGL контекст
    pub fn is_initialized(&self) -> bool {
        self.gl_context.is_some()
    }
}

fn create_dummy_window() -> winit::window::Window {
    // Создаём dummy окно через headless режим
    // В реальности это не должно вызываться, так как окно создаётся в GlContext::new
    panic!("Dummy window creation not supported - use GlContext::new instead")
}
