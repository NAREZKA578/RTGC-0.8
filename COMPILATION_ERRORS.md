# Список ошибок компиляции RTGC-0.8

Дата: 27 марта 2026 г.
Всего ошибок: 334
Предупреждений: 217

## Критические ошибки (E0xxx)

### Ошибки импорта (E0432, E0433, E0603)

1. **E0432** - unresolved import `crate::input::gamepad::Gamepad`
   - Файл: src/input/input_module.rs
   - Решение: Gamepad не экспортируется из gamepad модуля

2. **E0432** - unresolved import `crate::physics_module` (2 случая)
   - Файлы: src/physics/fracture_component.rs, src/physics/deformable_terrain.rs
   - Решение: Использовать `crate::physics::physics_module`

3. **E0433** - failed to resolve: unresolved import (3 случая)
   - Профиль ProfileScope не найден

4. **E0603** - struct import `Material` is private
   - Файл: src/graphics/render_command.rs
   - Решение: Экспортировать Material из graphics/mod.rs

### Ошибки типов (E0412, E0308, E0277)

5. **E0412** - cannot find type `SurfaceType` in module `crate::physics`
   - Файл: src/engine.rs
   - Решение: SurfaceType находится в world::terrain_generator

6. **E0412** - cannot find type `InventoryItem` in module `crate::game::cargo`
   - Файл: src/game/save.rs
   - Решение: InventoryItem находится в game::inventory

7. **E0412/E0433** - cannot find type `Vec2`/`Vec3` in this scope (множество случаев)
   - Файлы: game/ui.rs, game/interaction.rs
   - Решение: Использовать nalgebra::Vector2/Vector3

8. **E0308** - mismatched types (множество случаев)
   - Различные файлы, несовместимые типы

9. **E0277** - `*mut c_void` cannot be shared/sent between threads safely
   - Файл: src/graphics/gl_context.rs
   - Решение: Добавить unsafe impl Send/Sync

### Ошибки заимствования (E0499, E0502, E0382)

10. **E0499** - cannot borrow `*self` as mutable more than once at a time
    - Файл: src/physics/crane_arm.rs:356
    - Решение: Рефакторинг update_load_physics

11. **E0382** - use of moved value: `stats`
    - Файл: src/game/debug_menu.rs:68
    - Решение: Clone stats перед использованием

### Ошибки реализации трейтов (E0119, E0223, E0061)

12. **E0119** - conflicting implementations of trait `std::marker::Copy` for type `Handle<_>`
    - Файл: src/graphics/render_command.rs
    - Решение: Удалить дублирующую реализацию Copy

13. **E0599** - no method named `clone` found for struct `parking_lot::lock_api::RwLock`
    - Файл: src/game/events.rs
    - Решение: Использовать lock() вместо clone()

### Ошибки отсутствующих методов/полей (E0599, E0609)

14. **E0599** - no method named `update` found for mutable reference
    - Файл: src/audio/engine.rs

15. **E0609** - no field `position` on type `nalgebra::Matrix<f32, ...>`
    - Файл: src/graphics/mesh.rs (множество случаев)
    - Решение: Использовать правильный тип Vertex

16. **E0599** - no variant or associated item named `VertexBuffer` found for enum `BufferType`
    - Файл: src/graphics/renderer_rhi.rs
    - Решение: Проверить названия вариантов enum

## Предупреждения (warning)

### Неиспользуемые переменные (217 предупреждений)

Основные категории:
- `unused variable: dt` - параметр времени во многих функциях
- `unused variable: renderer` - параметр renderer в gl_context.rs
- `unused variable: terrain_getter` - параметр в physics и world модулях
- `unused variable: main_road_dir` - параметр в buildings.rs
- `unused variable: body`, `distance` - параметры в day_night_cycle.rs

### Переменные, которые не нуждаются в mutable

- `variable does not need to be mutable` в async_physics.rs, russian_names.rs, road_network.rs, random.rs

### Значения, присвоенные, но не прочитанные

- `value assigned to 's' is never read` в physics_module.rs
- `value assigned to 't' is never read` в physics_module.rs
- `value assigned to 'y' is never read` в debug_menu.rs

## Файлы с наибольшим количеством ошибок

1. **src/graphics/rhi/gl.rs** - множество ошибок с форматами текстур и методами
2. **src/graphics/mesh.rs** - ошибки с типами Vertex и методами
3. **src/audio/engine.rs** - ошибки с методами update_3d_position, is_finished
4. **src/graphics/renderer_rhi.rs** - ошибки с BufferType и полями TextureDescription
5. **src/world/*.rs** - множество предупреждений о неиспользуемых переменных

## Рекомендуемый порядок исправления

1. Исправить ошибки импорта (E0432, E0433, E0603)
2. Исправить ошибки типов (E0412, E0308)
3. Исправить ошибки заимствования (E0499, E0502, E0382)
4. Исправить ошибки реализации трейтов (E0119)
5. Добавить отсутствующие методы/поля (E0599, E0609)
6. Убрать предупреждения о неиспользуемых переменных
