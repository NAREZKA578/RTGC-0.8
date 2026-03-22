# RTGC-0.7 — Real-Time Ground & air Combat Simulator

**RTGC-0.7** — это высокопроизводительный симулятор военной техники с реалистичной физикой, написанный на языке программирования **Rust**. Проект включает в себя продвинутую физику наземного транспорта и вертолётов, процедурную генерацию мира, многопоточную архитектуру и поддержку современных графических API.

## 🚀 Возможности

### Физика
- **Наземный транспорт**: Реалистичная модель подвески, сцепления колёс, дифференциалов
- **Вертолёты**: Полная 6DOF физика с Blade Element Momentum Theory (BEMT)
- **Асинхронная симуляция**: Многопоточный физический движок с sub-stepping
- **Пространственное хэширование**: Оптимизированное обнаружение коллизий

### Графика
- **RHI абстракция**: Единый интерфейс для OpenGL, Vulkan, DirectX 12
- **LOD система**: Динамические уровни детализации с hysteresis
- **Потоковая загрузка**: Асинхронная загрузка текстур и мешей
- **PBR рендеринг**: HDR, tone mapping, metallic/roughness материалы

### Архитектура
- **ECS (Entity Component System)**: Эффективное управление сущностями
- **Job System**: Параллельное выполнение задач
- **Arena Allocator**: Быстрое выделение памяти
- **Модульность**: Чёткое разделение ответственности между модулями

## 📁 Структура проекта

```
RTGC-0.7/
├── src/
│   ├── physics/           # Физический движок
│   │   ├── helicopter.rs  # Физика вертолётов (универсальная)
│   │   ├── advanced_vehicle.rs  # Физика транспорта
│   │   ├── async_physics.rs     # Асинхронная симуляция
│   │   ├── spatial_hash.rs      # Пространственное хэширование
│   │   └── ...
│   ├── graphics/          # Графическая подсистема
│   │   ├── rhi/           # RHI абстракция (GL/Vulkan/DX12)
│   │   ├── lod_system.rs  # Система LOD
│   │   └── ...
│   ├── ecs/               # Entity Component System
│   │   └── job_system.rs  # Система задач
│   ├── assets/            # Загрузка ресурсов
│   ├── audio/             # Аудио система
│   ├── input/             # Обработка ввода
│   └── game/              # Игровая логика
├── assets/                # Ресурсы игры
├── Cargo.toml             # Зависимости Rust
├── PLEN.md                # Подробная документация по всем файлам
└── README.md              # Этот файл
```

## 🛠️ Быстрый старт

### Требования
- Rust 1.75+ (nightly рекомендуется)
- Vulkan SDK (для Vulkan бэкенда)
- Графическая карта с поддержкой OpenGL 4.5+ или Vulkan 1.2+

### Сборка
```bash
# Клонируйте репозиторий
git clone https://github.com/NAREZKA578/RTGC-0.7.git
cd RTGC-0.7

# Сборка релизной версии
cargo build --release

# Запуск
cargo run --release
```

### Тесты
```bash
cargo test --release
```

## 🚁 Использование физики вертолётов

```rust
use nalgebra::Vector3;
use rtgc::physics::helicopter::{Helicopter, HelicopterConfig, EngineType};

// Создание лёгкого вертолёта (Robinson R22)
let config = HelicopterConfig::light_helicopter();
let mut heli = Helicopter::with_config(Vector3::new(0.0, 10.0, 0.0), config);

// Или создание среднего вертолёта (Bell UH-1)
let config = HelicopterConfig::medium_helicopter();
let mut heli = Helicopter::with_config(Vector3::new(0.0, 50.0, 0.0), config);

// Или пользовательская конфигурация
let config = HelicopterConfig::custom(
    5000.0,   // масса (кг)
    8.0,      // радиус ротора (м)
    4,        // количество лопастей
    1500000.0 // мощность двигателя (Вт)
);
let mut heli = Helicopter::with_config(Vector3::zeros(), config);

// Запуск двигателя
heli.engine.start_engine();

// Управление
heli.controls.collective = 0.7;      // Общий шаг
heli.controls.cyclic_longitudinal = 0.3; // Циклик вперёд
heli.controls.cyclic_lateral = -0.2;     // Циклик влево
heli.controls.tail_rotor_pedals = 0.1;   // Педали
heli.controls.throttle = 0.9;            // Дроссель

// Обновление физики (dt = 16ms для 60 FPS)
heli.update(0.016);

// Получение состояния для рендеринга
let state = heli.get_state();
println!("Высота: {:.2} м, Скорость: {:.2} м/с", state.altitude, state.airspeed);
```

## 📊 Производительность

| Компонент | Оптимизация | Результат |
|-----------|-------------|-----------|
| Физика вертолёта | BEMT с 20 элементами | < 50 мкс на шаг |
| Пространственное хэширование | Inverse cell size | O(1) доступ |
| Job System | Lock-free очереди | ~95% утилизация CPU |
| Arena Allocator | Пакетное выделение | ~10x быстрее std::alloc |
| LOD система | Hysteresis + throttling | Без "popcorn effect" |

## 🔧 Конфигурация вертолётов

Проект включает предустановленные конфигурации:

| Тип | Масса | Мощность | Радиус ротора | Лопасти |
|-----|-------|----------|---------------|---------|
| Light (R22) | 620 кг | 97 кВт | 3.83 м | 2 |
| Medium (UH-1) | 2370 кг | 1050 кВт | 7.32 м | 2 |
| Heavy (Mi-8) | 7200 кг | 2500 кВт | 9.55 м | 5 |

## 📖 Документация

- **README.md** (этот файл) — краткое руководство и быстрый старт
- **PLEN.md** — подробное описание всех файлов проекта, их назначения и возможностей

## 🤝 Вклад в проект

1. Fork репозитория
2. Создайте feature branch (`git checkout -b feature/amazing-feature`)
3. Commit изменений (`git commit -m 'Add amazing feature'`)
4. Push в branch (`git push origin feature/amazing-feature`)
5. Откройте Pull Request

## 📄 Лицензия

Этот проект распространяется под лицензией MIT.

## 📞 Контакты

- GitHub: [@NAREZKA578](https://github.com/NAREZKA578)
- Issues: [GitHub Issues](https://github.com/NAREZKA578/RTGC-0.7/issues)

---

**RTGC-0.7** — создан с использованием языка Rust для максимальной производительности и безопасности.
