//! Менеджер рендеринга - инкапсуляция графической подсистемы
//! 
//! Этот модуль управляет рендерингом сцены, UI и пост-обработкой,
//! предоставляя контролируемый интерфейс для графических операций.

use crate::graphics::renderer::{Renderer, MenuState};
use crate::graphics::GlContext;
use crate::graphics::material::MaterialManager;
use crate::graphics::particles::ParticleSystem;
use crate::graphics::debug_renderer::DebugRenderer;
use crate::ui::HudManager;
use nalgebra::Matrix4;
use tracing::{error, info};

/// Менеджер рендеринга
pub struct RenderManager {
    /// Рендерер сцены
    renderer: Option<Renderer>,
    /// Графический контекст
    graphics_context: GlContext,
    /// Менеджер материалов
    material_manager: MaterialManager,
    /// Система частиц
    particle_system: ParticleSystem,
    /// Отладочный рендерер
    debug_renderer: DebugRenderer,
    /// HUD менеджер
    hud_manager: HudManager,
    /// Позиция мыши X
    mouse_x: f32,
    /// Позиция мыши Y
    mouse_y: f32,
    /// Режим отладки
    debug_mode: bool,
}

impl RenderManager {
    /// Создаёт новый менеджер рендеринга
    pub fn new(
        graphics_context: GlContext,
        material_manager: MaterialManager,
        particle_system: ParticleSystem,
        debug_renderer: DebugRenderer,
        hud_manager: HudManager,
    ) -> Self {
        Self {
            renderer: None,
            graphics_context,
            material_manager,
            particle_system,
            debug_renderer,
            hud_manager,
            mouse_x: 0.0,
            mouse_y: 0.0,
            debug_mode: false,
        }
    }
    
    /// Инициализирует рендерер
    pub fn initialize_renderer(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(ref gl) = self.graphics_context.gl {
            match Renderer::new(gl.clone()) {
                Ok(mut renderer) => {
                    renderer.width = self.graphics_context.width;
                    renderer.height = self.graphics_context.height;
                    renderer.menu_state = MenuState::MainMenu;
                    self.renderer = Some(renderer);
                    info!(target: "render", "Renderer initialized successfully");
                    Ok(())
                }
                Err(e) => {
                    error!(target: "render", error = ?e, "Renderer initialization failed");
                    Err(Box::new(e))
                }
            }
        } else {
            error!(target: "render", "GL context is None");
            Err("GL context not available".into())
        }
    }
    
    /// Обновляет позицию мыши
    pub fn update_mouse_position(&mut self, x: f32, y: f32) {
        self.mouse_x = x;
        self.mouse_y = y;
        
        if let Some(ref mut renderer) = self.renderer {
            renderer.mouse_x = x;
            renderer.mouse_y = y;
        }
    }
    
    /// Обновляет камеру на основе позиции вертолёта
    pub fn update_camera_from_helicopter(&mut self, position: nalgebra::Vector3<f32>) {
        if let Some(ref mut renderer) = self.renderer {
            renderer.camera.position = position;
        }
    }
    
    /// Обновляет камеру на основе позиции транспорта (общая версия)
    pub fn update_camera_from_vehicle(&mut self, position: nalgebra::Vector3<f32>, rotation: nalgebra::Quaternion<f32>) {
        if let Some(ref mut renderer) = self.renderer {
            // Камера следует за транспортом с небольшим смещением
            let offset = rotation * nalgebra::Vector3::new(0.0, 5.0, 10.0);
            renderer.camera.position = position + offset;
            renderer.camera.target = position;
            renderer.camera.update();
        }
    }
    
    /// Устанавливает трансформацию транспорта для рендеринга
    pub fn set_vehicle_transform(&mut self, position: nalgebra::Vector3<f32>, rotation: nalgebra::Quaternion<f32>) {
        if let Some(ref mut renderer) = self.renderer {
            renderer.vehicle_position = Some(position);
            renderer.vehicle_rotation = Some(rotation);
        }
    }
    
    /// Устанавливает цвета неба
    pub fn set_sky_colors(&mut self, top_color: nalgebra::Vector3<f32>, horizon_color: nalgebra::Vector3<f32>) {
        if let Some(ref mut renderer) = self.renderer {
            renderer.sky_top_color = top_color;
            renderer.sky_horizon_color = horizon_color;
        }
    }
    
    /// Устанавливает направление солнца
    pub fn set_sun_direction(&mut self, sun_dir: nalgebra::Vector3<f32>) {
        if let Some(ref mut renderer) = self.renderer {
            renderer.sun_direction = sun_dir;
        }
    }
    
    /// Устанавливает состояние меню в рендерере
    pub fn set_menu_state(&mut self, menu_state: MenuState) {
        if let Some(ref mut renderer) = self.renderer {
            renderer.menu_state = menu_state;
        }
    }
    
    /// Устанавливает режим отладки
    pub fn set_debug_mode(&mut self, enabled: bool) {
        self.debug_mode = enabled;
        if let Some(ref mut renderer) = self.renderer {
            renderer.debug_mode = enabled;
        }
    }
    
    /// Начинает кадр (очистка буфера)
    pub fn begin_frame(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.graphics_context.begin_frame()?;
        Ok(())
    }
    
    /// Рендерит кадр
    pub fn render(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(ref mut renderer) = self.renderer {
            renderer.render()?;
        } else if let Some(ref gl) = self.graphics_context.gl {
            // Fallback рендеринг
            let view_matrix = Matrix4::identity();
            let proj_matrix = self.graphics_context.get_projection_matrix(
                std::f32::consts::PI / 4.0,
                0.1,
                1000.0,
            );
            let view_proj = proj_matrix * view_matrix;
            
            self.particle_system.render(gl, view_proj);
            
            if self.debug_mode {
                self.debug_renderer.flush_to_gl(gl, view_proj);
            }
        }
        Ok(())
    }
    
    /// Завершает кадр (swap buffers)
    pub fn end_frame(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.graphics_context.end_frame()?;
        Ok(())
    }
    
    /// Обрабатывает изменение размера окна
    pub fn on_resize(&mut self, width: u32, height: u32) -> Result<(), Box<dyn std::error::Error>> {
        let width = width.max(1);
        let height = height.max(1);
        
        self.graphics_context.resize(width, height)?;
        
        if let Some(ref mut renderer) = self.renderer {
            renderer.width = width;
            renderer.height = height;
        }
        
        Ok(())
    }
    
    /// Получает ссылку на рендерер
    pub fn get_renderer(&self) -> Option<&Renderer> {
        self.renderer.as_ref()
    }
    
    /// Получает мутабельную ссылку на рендерер
    pub fn get_renderer_mut(&mut self) -> Option<&mut Renderer> {
        self.renderer.as_mut()
    }
    
    /// Получает ссылку на HUD менеджер
    pub fn get_hud_manager(&self) -> &HudManager {
        &self.hud_manager
    }
    
    /// Получает мутабельную ссылку на HUD менеджер
    pub fn get_hud_manager_mut(&mut self) -> &mut HudManager {
        &mut self.hud_manager
    }
    
    /// Проверяет, инициализирован ли рендерер
    pub fn is_initialized(&self) -> bool {
        self.renderer.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_render_manager_creation() {
        // Тест требует создания GlContext, что сложно в unit тесте
        // Поэтому просто проверяем что структура компилируется
    }
}
