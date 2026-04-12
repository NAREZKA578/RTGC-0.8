use nalgebra::Vector3;

/// Типы осадков
#[derive(Clone, Copy, PartialEq)]
pub enum PrecipitationType {
    None,
    Rain,
    Snow,
}

/// Состояние погоды
#[derive(Clone)]
pub struct WeatherState {
    pub precipitation_type: PrecipitationType,
    pub intensity: f32,      // 0.0 .. 1.0
    pub cloud_coverage: f32, // 0.0 .. 1.0
    pub wind_speed: f32,     // м/с
    pub wind_direction: Vector3<f32>,
    pub temperature: f32, // Цельсий
}

impl Default for WeatherState {
    fn default() -> Self {
        Self {
            precipitation_type: PrecipitationType::None,
            intensity: 0.0,
            cloud_coverage: 0.3,
            wind_speed: 2.0,
            wind_direction: Vector3::new(1.0, 0.0, 0.0).normalize(),
            temperature: 20.0,
        }
    }
}

/// Цикл дня и ночи
pub struct DayNightCycle {
    current_time: f32, // Время суток в часах (0.0 - 24.0)
    day_duration: f32, // Длительность дня в реальных секундах (например, 600 сек = 10 мин)
    sun_direction: Vector3<f32>,
    sun_intensity: f32,
    moon_intensity: f32,
    sky_color_top: Vector3<f32>,
    sky_color_horizon: Vector3<f32>,
    is_night: bool,
}

impl DayNightCycle {
    pub fn new(start_hour: f32, day_duration_sec: f32) -> Self {
        let mut cycle = Self {
            current_time: start_hour,
            day_duration: day_duration_sec,
            sun_direction: Vector3::y(),
            sun_intensity: 1.0,
            moon_intensity: 0.0,
            sky_color_top: Vector3::new(0.4, 0.6, 0.9),
            sky_color_horizon: Vector3::new(0.7, 0.8, 0.9),
            is_night: false,
        };
        cycle.update_sun(0.0);
        cycle
    }

    pub fn update(&mut self, dt: f32) {
        // Продвигаем время
        let hours_per_second = 24.0 / self.day_duration;
        self.current_time += dt * hours_per_second;
        if self.current_time >= 24.0 {
            self.current_time -= 24.0;
        }

        self.update_sun(dt);
    }

    fn update_sun(&mut self, _dt: f32) {
        // Угол солнца: 0° = восход (6:00), 90° = зенит (12:00), 180° = закат (18:00)
        // Смещаем фазу так, чтобы 12:00 было максимумом
        let angle_deg = (self.current_time - 6.0) * 15.0;
        let angle_rad = angle_deg.to_radians();

        // Солнце движется по дуге X-Y плоскости (для простоты, без учета широты)
        // Y вверх, X вперед/назад
        self.sun_direction = Vector3::new(angle_rad.sin(), angle_rad.cos(), 0.0).normalize();

        // Интенсивность зависит от высоты солнца (Y компонента)
        let height = self.sun_direction.y;

        if height > 0.0 {
            // День
            self.sun_intensity = height.min(1.0);
            self.moon_intensity = 0.0;
            self.is_night = false;

            // Цвет неба днем
            self.sky_color_top = Vector3::new(0.4, 0.6, 0.9);
            self.sky_color_horizon = Vector3::new(0.7, 0.8, 0.9);
        } else {
            // Ночь/Сумерки
            self.sun_intensity = 0.0;
            self.moon_intensity = (-height).min(1.0) * 0.3; // Луна слабее
            self.is_night = height < -0.2; // Полная ночь когда солнце глубоко

            // Цвет неба ночью
            let t = ((-height) * 5.0).min(1.0); // Интерполяция заката
            self.sky_color_top =
                Vector3::new(0.05, 0.05, 0.15).lerp(&Vector3::new(0.4, 0.6, 0.9), t);
            self.sky_color_horizon =
                Vector3::new(0.1, 0.1, 0.2).lerp(&Vector3::new(0.7, 0.8, 0.9), t);
        }
    }

    pub fn get_sun_direction(&self) -> Vector3<f32> {
        self.sun_direction
    }

    pub fn get_sun_intensity(&self) -> f32 {
        self.sun_intensity
    }

    pub fn get_ambient_intensity(&self) -> f32 {
        // Базовое освещение даже ночью (от луны/звезд)
        (if self.is_night { 0.1 } else { 0.4 }) + self.sun_intensity * 0.6
    }

    pub fn get_sky_color_top(&self) -> Vector3<f32> {
        self.sky_color_top
    }

    pub fn get_sky_color_horizon(&self) -> Vector3<f32> {
        self.sky_color_horizon
    }

    pub fn get_hour(&self) -> f32 {
        self.current_time
    }

    pub fn is_headlights_needed(&self) -> bool {
        self.sun_intensity < 0.3 || self.is_night
    }
}

/// Система погоды
pub struct WeatherSystem {
    state: WeatherState,
    transition_timer: f32,
    next_weather_duration: f32,
    seed: u64,
}

impl WeatherSystem {
    pub fn new(seed: u64) -> Self {
        Self {
            state: WeatherState::default(),
            transition_timer: 0.0,
            next_weather_duration: 300.0, // Меняется каждые 5 мин
            seed,
        }
    }

    pub fn update(&mut self, dt: f32, current_hour: f32) {
        self.transition_timer += dt;

        // Простая смена погоды по таймеру (можно усложнить через шум Перлина от seed)
        if self.transition_timer > self.next_weather_duration {
            self.transition_timer = 0.0;
            self.randomize_weather(current_hour);
            self.next_weather_duration = 300.0 + (self.seed % 600) as f32; // 5-15 мин
        }
    }

    fn randomize_weather(&mut self, hour: f32) {
        // Шанс дождя выше днем или вечером
        let rain_chance = if hour > 6.0 && hour < 20.0 { 0.4 } else { 0.2 };

        if (self.seed % 100) as f32 / 100.0 < rain_chance {
            self.state.precipitation_type = PrecipitationType::Rain;
            self.state.intensity = 0.5 + ((self.seed % 50) as f32 / 50.0) * 0.5;
            self.state.cloud_coverage = 0.8 + ((self.seed % 20) as f32 / 20.0) * 0.2;
        } else {
            self.state.precipitation_type = PrecipitationType::None;
            self.state.intensity = 0.0;
            self.state.cloud_coverage = 0.2 + ((self.seed % 40) as f32 / 40.0) * 0.5;
        }

        self.seed = self.seed.wrapping_mul(6364136223846793005).wrapping_add(1);
    }

    pub fn get_state(&self) -> &WeatherState {
        &self.state
    }

    /// Возвращает коэффициент трения (1.0 = сухо, 0.4 = сильный дождь/грязь)
    pub fn get_friction_modifier(&self) -> f32 {
        match self.state.precipitation_type {
            PrecipitationType::None => 1.0,
            PrecipitationType::Rain => 1.0 - (self.state.intensity * 0.4), // До 0.6
            PrecipitationType::Snow => 1.0 - (self.state.intensity * 0.6), // До 0.4
        }
    }

    pub fn get_precipitation_intensity(&self) -> f32 {
        self.state.intensity
    }

    pub fn get_cloud_coverage(&self) -> f32 {
        self.state.cloud_coverage
    }
}
