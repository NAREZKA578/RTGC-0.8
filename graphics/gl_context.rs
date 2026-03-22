//! OpenGL Context для RTGC-0.7
//! Реализация инициализации OpenGL контекста с использованием glutin и winit

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
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowAttributes};

/// OpenGL контекст для рендеринга
pub struct GlContext {
    pub gl: Context,
    pub window: Window,
    pub width: u32,
    pub height: u32,
    // Храним контекст и поверхность для swap_buffers
    gl_context: PossiblyCurrentContext,
    surface: glutin::surface::Surface<WindowSurface>,
}

unsafe impl Send for GlContext {}
unsafe impl Sync for GlContext {}

impl GlContext {
    /// Создаёт новый OpenGL контекст с окном
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
                configs.next().unwrap()
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
        let (width, height): (u32, u32) = window.inner_size().into();
        
        // Используем SurfaceAttributesBuilder вместо default()
        let surface_attrs = SurfaceAttributesBuilder::<WindowSurface>::new().build(
            raw_window_handle,
            NonZeroU32::new(width).unwrap(),
            NonZeroU32::new(height).unwrap(),
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
        let _ = surface.set_swap_interval(&gl_context, SwapInterval::Wait(NonZeroU32::new(1).unwrap()));

        Ok(Self {
            gl,
            window,
            width,
            height,
            gl_context,
            surface,
        })
    }

    /// Изменяет размер поверхности при ресайзе окна
    pub fn resize(&mut self, width: u32, height: u32) -> Result<(), Box<dyn std::error::Error>> {
        self.width = width;
        self.height = height;

        // Изменяем размер поверхности
        self.surface.resize(
            &self.gl_context,
            width.try_into().unwrap(),
            height.try_into().unwrap()
        );

        // Обновляем viewport в OpenGL
        unsafe {
            glow::HasContext::viewport(&self.gl, 0, 0, width as i32, height as i32);
        }

        Ok(())
    }

    /// Меняет буферы (swap buffers)
    pub fn swap_buffers(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.surface.swap_buffers(&self.gl_context)?;
        Ok(())
    }

    /// Получает размеры окна
    pub fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Устанавливает заголовок окна
    pub fn set_title(&self, title: &str) {
        self.window.set_title(title);
    }

    /// Запрашивает перерисовку
    pub fn request_redraw(&self) {
        self.window.request_redraw();
    }
}
