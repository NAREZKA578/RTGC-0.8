// ЧАСТЬ 1 — HUD: ЕДИНЫЙ ЦЕНТР ИНФОРМАЦИИ
// Весь HUD хранится в одном месте, управляется единым HudManager.

use nalgebra::{Vector3, UnitQuaternion};

/// Все данные для HUD — заполняются движком, рисуются HudManager
#[derive(Debug, Clone, Default)]
pub struct VehicleHudData {
    // === Блок ДВИЖЕНИЯ ===
    pub speed_kmh: f32,              // Текущая скорость км/ч
    pub speed_max_kmh: f32,          // Максимальная скорость (для шкалы)
    pub gear: GearState,             // Передача: Park, Rev, N, 1..8
    pub engine_rpm: f32,             // Текущие обороты
    pub engine_rpm_max: f32,         // Красная зона начинается отсюда
    pub engine_running: bool,        // Двигатель запущен?

    // === Блок РЕСУРСОВ ===
    pub fuel_level: f32,             // 0.0..1.0
    pub fuel_reserve: bool,          // Резервный уровень (мигать)
    pub engine_temp: f32,            // °C, 0..120
    pub engine_overheating: bool,    // Перегрев (мигать)

    // === Блок ТРАНСМИССИИ ===
    pub diff_front_locked: bool,     // Блокировка переднего диффа
    pub diff_rear_locked: bool,      // Блокировка заднего диффа
    pub diff_center_locked: bool,    // Межосевая блокировка
    pub awd_active: bool,            // Полный привод активен
    pub low_range: bool,             // Понижающий ряд включён

    // === Блок ПОДВЕСКИ ===
    pub wheel_contact: [bool; 4],    // Какие колёса в контакте с землёй
    pub wheel_slip: [f32; 4],        // Проскальзывание 0..1 каждого колеса
    pub suspension_load: [f32; 4],   // Нагрузка подвески 0..1

    // === Блок ГРУЗА ===
    pub cargo_attached: bool,        // Груз прицеплен
    pub cargo_weight_kg: f32,        // Масса груза
    pub cargo_damage: f32,           // Повреждение груза 0..1
    pub winch_active: bool,          // Лебёдка активна
    pub winch_length_m: f32,         // Длина троса лебёдки

    // === Блок ОКРУЖЕНИЯ ===
    pub altitude_m: f32,             // Высота над уровнем моря
    pub terrain_angle_deg: f32,      // Угол наклона поверхности
    pub vehicle_roll_deg: f32,       // Крен машины (бок)
    pub vehicle_pitch_deg: f32,      // Тангаж (нос/корма)
    pub is_tipped_over: bool,        // Машина перевёрнута?

    // === Блок ПОВРЕЖДЕНИЙ ===
    pub vehicle_health: f32,         // 0.0..1.0
}

#[derive(Debug, Clone, PartialEq)]
pub enum GearState {
    Park,
    Reverse,
    Neutral,
    Drive(u8),  // 1..8
}

impl Default for GearState {
    fn default() -> Self {
        GearState::Neutral
    }
}

/// Конфигурация отображения HUD
#[derive(Debug, Clone)]
pub struct HudLayout {
    pub show_speed: bool,
    pub show_gear: bool,
    pub show_fuel: bool,
    pub show_diff_status: bool,
    pub show_wheel_status: bool,
    pub show_cargo: bool,
    pub show_terrain_angle: bool,
    pub compact_mode: bool,   // Мини-версия для слабых экранов
    pub show_minimap: bool,   // Правый блок с картой
}

impl Default for HudLayout {
    fn default() -> Self {
        Self {
            show_speed: true,
            show_gear: true,
            show_fuel: true,
            show_diff_status: true,
            show_wheel_status: true,
            show_cargo: true,
            show_terrain_angle: true,
            compact_mode: false,
            show_minimap: true,
        }
    }
}

/// Единый менеджер HUD — единственное место где рисуется интерфейс
pub struct HudManager {
    visible: bool,
    opacity: f32,
    layout: HudLayout,
    last_data: Option<VehicleHudData>,
    // Анимационные состояния
    flash_timer: f32,
    flash_element: Option<HudFlashElement>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HudFlashElement {
    FuelReserve,
    EngineOverheat,
    WheelSlip(usize),  // index 0..3
}

impl HudManager {
    pub fn new() -> Self {
        Self {
            visible: true,
            opacity: 1.0,
            layout: HudLayout::default(),
            last_data: None,
            flash_timer: 0.0,
            flash_element: None,
        }
    }

    /// Обновить данные HUD
    pub fn update(&mut self, data: VehicleHudData, dt: f32) {
        // Проверка на мигающие элементы
        if data.fuel_reserve {
            self.flash_element = Some(HudFlashElement::FuelReserve);
            self.flash_timer = 0.5;  // мигать каждые 0.5 сек
        } else if data.engine_overheating {
            self.flash_element = Some(HudFlashElement::EngineOverheat);
            self.flash_timer = 0.3;  // быстрее мигать для перегрева
        } else {
            // Проверка проскальзывания колёс
            let mut slipping_wheel = None;
            for (i, &slip) in data.wheel_slip.iter().enumerate() {
                if slip > 0.5 {
                    slipping_wheel = Some(i);
                    break;
                }
            }
            
            if let Some(idx) = slipping_wheel {
                self.flash_element = Some(HudFlashElement::WheelSlip(idx));
                self.flash_timer = 0.2;
            } else {
                self.flash_element = None;
            }
        }

        // Обновление таймера мигания
        if self.flash_timer > 0.0 {
            self.flash_timer -= dt;
            if self.flash_timer <= 0.0 {
                self.flash_timer = 0.0;
                self.flash_element = None;
            }
        }

        self.last_data = Some(data);
    }

    /// Показать/скрыть HUD
    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Установить прозрачность (0.0..1.0)
    pub fn set_opacity(&mut self, opacity: f32) {
        self.opacity = opacity.clamp(0.0, 1.0);
    }

    /// Получить текущие данные
    pub fn get_data(&self) -> Option<&VehicleHudData> {
        self.last_data.as_ref()
    }

    /// Получить конфигурацию отображения
    pub fn get_layout(&self) -> &HudLayout {
        &self.layout
    }

    /// Изменить конфигурацию отображения
    pub fn set_layout(&mut self, layout: HudLayout) {
        self.layout = layout;
    }

    /// Проверить, должен ли элемент мигать сейчас
    pub fn is_flashing(&self, element: &HudFlashElement) -> bool {
        if let Some(ref flash) = self.flash_element {
            if flash == element {
                // Мигать: включено половину времени
                return self.flash_timer > 0.25;
            }
        }
        false
    }

    /// Сгенерировать VehicleHudData из параметров автомобиля (helper)
    pub fn create_vehicle_data(
        speed_kmh: f32,
        rpm: f32,
        rpm_max: f32,
        gear: GearState,
        engine_running: bool,
        fuel: f32,
        temp: f32,
    ) -> VehicleHudData {
        VehicleHudData {
            speed_kmh,
            speed_max_kmh: 120.0,  // default for trucks
            gear,
            engine_rpm: rpm,
            engine_rpm_max: rpm_max,
            engine_running,
            fuel_level: fuel,
            fuel_reserve: fuel < 0.15,
            engine_temp: temp,
            engine_overheating: temp > 100.0,
            ..Default::default()
        }
    }

    /// Рендеринг HUD через OpenGL
    /// Вызывается из renderer.rs после отрисовки игрового мира
    pub fn render(&self, renderer: &mut crate::graphics::renderer::Renderer) {
        if !self.visible || self.last_data.is_none() {
            return;
        }

        let data = self.last_data.as_ref().unwrap();
        let layout = &self.layout;

        let screen_width = renderer.get_width() as f32;
        let screen_height = renderer.get_height() as f32;

        // === СТИЛЬ: МИНИМАЛИЗМ / ГРЯЗЬ / ХАРДКОР ===
        // Никаких спидометров и тахометров. Только критическая информация.

        // 1. ИНДИКАТОРЫ ДИФФЕРЕНЦИАЛОВ (Левый верхний угол)
        // Крупные буквы, горят ярко, когда включены
        let diff_font_size = 28.0;
        let start_x = 25.0;
        let start_y = 25.0;
        let gap = 45.0;

        // Передний дифф
        let f_color = if data.diff_front_locked { [1.0, 0.2, 0.2, 1.0] } else { [0.3, 0.3, 0.3, 0.4] };
        unsafe { renderer.draw_text("F", start_x, start_y, diff_font_size, f_color); }

        // Центральный дифф
        let c_color = if data.diff_center_locked { [1.0, 0.8, 0.0, 1.0] } else { [0.3, 0.3, 0.3, 0.4] };
        unsafe { renderer.draw_text("C", start_x + gap, start_y, diff_font_size, c_color); }

        // Задний дифф
        let r_color = if data.diff_rear_locked { [1.0, 0.2, 0.2, 1.0] } else { [0.3, 0.3, 0.3, 0.4] };
        unsafe { renderer.draw_text("R", start_x + gap * 2.0, start_y, diff_font_size, r_color); }

        // Пониженная передача
        if data.low_range {
            let low_color = [1.0, 0.6, 0.0, 1.0];
            unsafe { renderer.draw_text("LOW", start_x + gap * 0.5, start_y + 35.0, 18.0, low_color); }
        }

        // 2. СТАТУС ЛЕБЁДКИ (Правый верхний угол)
        if data.winch_active {
            let winch_x = screen_width - 180.0;
            let winch_y = 25.0;

            // Длина троса
            let rope_len = format!("{:.1}m", data.winch_length_m);
            unsafe { renderer.draw_text(&rope_len, winch_x, winch_y, 22.0, [1.0, 0.9, 0.5, 1.0]); }

            // Статус натяжения
            let tension_status = if data.winch_length_m > 0.5 { "TIGHT" } else { "LOOSE" };
            let tension_color = if data.winch_length_m > 0.5 { [1.0, 0.3, 0.3, 1.0] } else { [0.5, 0.5, 0.5, 0.8] };
            unsafe { renderer.draw_text(tension_status, winch_x, winch_y + 28.0, 16.0, tension_color); }

            // Рамка вокруг лебедки если активна
            unsafe { renderer.draw_rect(winch_x - 8.0, winch_y - 8.0, 130.0, 60.0, [0.0, 0.0, 0.0, 0.4]); }
            unsafe { renderer.draw_rect_border(winch_x - 8.0, winch_y - 8.0, 130.0, 60.0, 2.0, [0.6, 0.6, 0.6, 0.6]); }
        }

        // 3. КОЛЕСА И КОНТАКТ (Нижняя часть экрана, по центру)
        // 4 точки, показывающие загрузку колес. 
        // Зеленая = контакт с землей, Красная = в воздухе (вывешено)
        if layout.show_wheel_status {
            let wheel_y = screen_height - 70.0;
            let wheel_spacing = 50.0;
            let total_w = wheel_spacing * 3.0;
            let start_wheel_x = (screen_width - total_w) / 2.0;

            for (i, &contact) in data.wheel_contact.iter().enumerate() {
                let x = start_wheel_x + (i as f32 * wheel_spacing);
                let color = if contact { [0.0, 1.0, 0.3, 1.0] } else { [1.0, 0.0, 0.0, 0.7] };

                // Основной индикатор контакта
                let size = 10.0;
                unsafe { renderer.draw_rect(x, wheel_y, size, size, color); }

                // Если колесо в воздухе, добавляем вторую точку ниже (индикатор хода подвески)
                if !contact {
                    unsafe { renderer.draw_rect(x, wheel_y + 16.0, size, size, [0.4, 0.4, 0.4, 0.6]); }
                }

                // Мигание при сильном проскальзывании
                if data.wheel_slip.get(i).copied().unwrap_or(0.0) > 0.4 {
                    let slip_color = [1.0, 1.0, 0.0, 0.8];
                    unsafe { renderer.draw_rect(x + 2.0, wheel_y + 2.0, size - 4.0, size - 4.0, slip_color); }
                }
            }
        }

        // 4. ПОДСКАЗКИ УПРАВЛЕНИЯ (Внизу по центру, полупрозрачные)
        let hints_y = screen_height - 40.0;
        let hint_color = [0.5, 0.5, 0.5, 0.7];
        let font_size = 13.0;

        let hint_text = "[WASD] Drive  [SHIFT] Winch  [B] Diff Locks  [ESC] Menu";
        let text_w = hint_text.len() as f32 * font_size * 0.55;
        let hint_x = (screen_width - text_w) / 2.0;

        unsafe { renderer.draw_text(hint_text, hint_x, hints_y, font_size, hint_color); }

        // 5. ИНДИКАТОР ПОВРЕЖДЕНИЙ (Оверлей по краям экрана)
        // Если здоровье машины < 100%, рисуем красную виньетку по краям
        if data.vehicle_health < 1.0 {
            let damage_factor = 1.0 - data.vehicle_health;
            let alpha = (damage_factor * 0.6).min(0.75);

            let border_size = 35.0 * (1.0 + damage_factor * 1.5);

            // Top
            unsafe { renderer.draw_rect(0.0, 0.0, screen_width, border_size, [1.0, 0.0, 0.0, alpha]); }
            // Bottom
            unsafe { renderer.draw_rect(0.0, screen_height - border_size, screen_width, border_size, [1.0, 0.0, 0.0, alpha]); }
            // Left
            unsafe { renderer.draw_rect(0.0, 0.0, border_size, screen_height, [1.0, 0.0, 0.0, alpha]); }
            // Right
            unsafe { renderer.draw_rect(screen_width - border_size, 0.0, border_size, screen_height, [1.0, 0.0, 0.0, alpha]); }

            // Текст предупреждения если критично
            if data.vehicle_health < 0.25 {
                let warn_text = "CRITICAL DAMAGE";
                let warn_x = (screen_width - 180.0) / 2.0;
                let warn_y = screen_height / 2.0 - 80.0;
                unsafe { renderer.draw_text(warn_text, warn_x, warn_y, 32.0, [1.0, 0.0, 0.0, 1.0]); }
            }
        }

        // 6. СТАТУС ГРУЗА (Левая сторона, ниже диффов)
        if layout.show_cargo && data.cargo_attached {
            let cargo_x = 25.0;
            let cargo_y = 120.0;

            let weight_text = format!("{:.0} kg", data.cargo_weight_kg);
            unsafe { renderer.draw_text(&weight_text, cargo_x, cargo_y, 20.0, [0.9, 0.9, 0.9, 0.9]); }

            // Повреждение груза
            if data.cargo_damage > 0.3 {
                let damage_color = if data.cargo_damage > 0.7 { [1.0, 0.0, 0.0, 1.0] } else { [1.0, 0.5, 0.0, 1.0] };
                unsafe { renderer.draw_text("DAMAGED", cargo_x, cargo_y + 25.0, 16.0, damage_color); }
            }
        }
    }
}

impl Default for HudManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hud_manager_creation() {
        let hud = HudManager::new();
        assert!(hud.is_visible());
        assert_eq!(hud.get_data(), None);
    }

    #[test]
    fn test_hud_update() {
        let mut hud = HudManager::new();
        let data = VehicleHudData {
            speed_kmh: 60.0,
            engine_rpm: 2000.0,
            gear: GearState::Drive(3),
            engine_running: true,
            fuel_level: 0.5,
            ..Default::default()
        };
        
        hud.update(data.clone(), 0.016);
        
        assert_eq!(hud.get_data().unwrap().speed_kmh, 60.0);
        assert_eq!(hud.get_data().unwrap().gear, GearState::Drive(3));
    }

    #[test]
    fn test_fuel_reserve_flash() {
        let mut hud = HudManager::new();
        let data = VehicleHudData {
            fuel_level: 0.1,  // ниже 15%
            ..Default::default()
        };
        
        hud.update(data, 0.016);
        assert!(hud.flash_element.is_some());
        assert_eq!(hud.flash_element.unwrap(), HudFlashElement::FuelReserve);
    }
}
