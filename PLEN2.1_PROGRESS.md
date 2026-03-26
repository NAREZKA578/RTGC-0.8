# PLEN2.1 — Прогресс выполнения

## ФАЗА 0 — ФУНДАМЕНТ ИГРЫ ✅ ЗАВЕРШЕНА

### Ф0.1 — Пешеходный персонаж ✅
**Файл:** `src/game/player.rs` (332 строки)
- [x] Player struct (имя, пол, рост, навыки, деньги, инвентарь 60кг)
- [x] LAYER_PLAYER = 0b10000
- [x] PlayerState: OnFoot / InVehicle
- [x] CameraMode: FirstPerson / ThirdPerson
- [x] Управление: WASD + Пробел (прыжок) + Shift (бег)
- [x] Взаимодействие: F (вход/выход из техники, общее взаимодействие)
- [x] Камера: 3rd person по умолчанию, зум колесом, вращение ПКМ, V для переключения

### Ф0.2 — Экран создания персонажа ✅
**Файл:** `src/game/character_creation.rs` (651 строка)
- [x] Пол (мужской/женский)
- [x] Рост (1.50–2.10 м)
- [x] Цвет кожи (8 вариантов)
- [x] Лицо (6 вариантов)
- [x] Волосы (11 цветов × 8 причёсок)
- [x] Образование (ВУЗы/колледжи из toml)
- [x] Выбор цвета UAZ Patriot (12 цветов)
- [x] Точка старта (Новосибирск и область)
- [x] Итоговый экран (навыки, деньги, контакты)

### Ф0.3 — Система навыков ✅
**Файл:** `src/game/skills.rs` (397 строк)
- [x] Skill: rank 1–12, mastery 0.0–1.0, total_hours
- [x] 20 навыков (mechanics, electrics, welding, construction, road_building, driving, tracked, piloting, flying, crane, geology, drilling, logging, mining, business, logistics, trading, navigation, medicine, fitness)
- [x] gain_xp(hours, difficulty)
- [x] Ограничения: самообучение ≤ ранг 4, ВУЗ даёт ранг 4-6
- [x] Проверки влияния:
  - [x] mechanics < 2 → не умеет чинить двигатель
  - [x] piloting < 4 → нет лицензии на вертолёт
  - [x] business < 3 → нельзя открыть ИП
  - [x] geology ≥ 4 → видишь тип ресурса
  - [x] logistics ≥ 6 → +52% к оплате контрактов
  - [x] fitness влияет на бег/выносливость
  - [x] medicine влияет на лечение

### Ф0.4 — UAZ Patriot 2017 ✅
**Файл:** `assets/vehicles/uaz_patriot/uaz_patriot_2017.vehicle.toml` (174 строки)
- [x] Реальный ЗМЗ-409 (149.6 л.с., 235.4 Нм)
- [x] Масса 2100 кг, 4×4
- [x] 12 цветов
- [x] Лебёдка 4500 кг
- [x] Детали с integrity 65-95%

### Ф0.5 — База ВУЗов ✅
**Файл:** `assets/education/universities.toml` (589 строк)
- [x] 20+ российских ВУЗов (НГТУ, НГУ, МГУ, МФТИ, ВШЭ, МАДИ...)
- [x] 3 китайских (Цинхуа, Пекинский, Харбинский)
- [x] 35+ специальностей с навыками
- [x] Стартовый капитал и контакты

---

## ДОПОЛНИТЕЛЬНЫЕ СИСТЕМЫ ✅

### Система сохранений ✅
**Файл:** `src/game/save.rs` (496 строк)
- [x] Сохранения только в safe locations (кровать, палатка, машина с койкой, собственность)
- [x] SaveData: игрок (визуал + навыки), позиция, техника, WorldState
- [x] MAX_SAVE_SLOTS = 10
- [x] SaveLocationType: Bed, Tent, VehicleCabin, OwnedProperty

### Debug Menu ✅
**Файл:** `src/game/debug_menu.rs` (237 строк)
- [x] F3 overlay
- [x] FPS + frametime
- [x] CPU usage per core
- [x] RAM usage
- [x] GPU VRAM (если glow позволит)
- [x] Активные RigidBody, коллизии, чанки
- [x] Навыки игрока, деньги, инвентарь
- [x] Координаты, скорость, состояние

### Event System ✅
**Файл:** `src/game/events.rs` (268 строк)
- [x] crossbeam-channel
- [x] События: PlayerEnteredVehicle, PlayerExitedVehicle, SkillLeveledUp, VehicleDamaged, InteractionTriggered
- [x] EventManager для подписчиков
- [x] publish_event(), poll_events()

### Main Menu System ✅
**Файл:** `src/game/main_menu.rs` (262 строки)
- [x] MenuState: MainMenu, CharacterCreation, Loading, Paused
- [x] New Game → Character Creation
- [x] Continue → загрузка последнего сохранения
- [x] Options, Exit
- [x] MenuAction enum для обработки

### Interaction System ✅
**Файл:** `src/game/interaction.rs` (418 строк)
- [x] LAYER_INTERACTABLE_* битмаски
- [x] MAX_INTERACTION_DISTANCE = 3.0 м
- [x] InteractableType: VehicleDoor, Door, PickableObject, NPC, Bed, Workbench, FuelPump, Shop
- [x] Raycast от камеры
- [x] try_interact() с cooldown 300мс
- [x] Обработка входа/выхода из техники
- [x] Открытие/закрытие дверей
- [x] Подбор предметов (проверка веса)
- [x] Сон/сохранение на кровати

### UI System ✅
**Файл:** `src/game/ui.rs` (427 строк)
- [x] UIVisibility flags
- [x] HUDData (health, stamina, speed, fuel, money, time, weather, location, gear, rpm, heading, position)
- [x] Notification system (Info, Success, Warning, Error, SkillUp, Achievement)
- [x] InteractionPrompt
- [x] MinimapData (waypoints, vehicles, NPCs)
- [x] UIManager: update, add_notification, notify_skill_up, toggle_*
- [x] Конфиг: `assets/ui/ui_config.toml`

### Inventory System ✅
**Файл:** `src/game/inventory.rs` (489 строк)
- [x] MAX_INVENTORY_WEIGHT = 60.0 кг
- [x] MAX_INVENTORY_SLOTS = 40
- [x] ItemType: 50+ типов (Tools, Vehicle parts, Construction, Resources, Consumables, Miscellaneous, Documents)
- [x] InventoryItem: type, quantity, condition, custom_name
- [x] Stacking (max_stack_size per type)
- [x] add_item(), remove_item(), has_item(), get_quantity()
- [x] Weight calculation
- [x] Value calculation (rubles)

---

## ИТОГО ФАЗА 0

| Категория | Файлов | Строк кода | Статус |
|-----------|--------|------------|--------|
| **Основные системы** | 10 | ~4500 | ✅ |
| **Конфиги/assets** | 3 | ~850 | ✅ |
| **Всего** | **13** | **~5350** | **✅ ГОТОВО** |

### Созданные файлы:
1. `src/game/player.rs` — персонаж
2. `src/game/character_creation.rs` — создание персонажа
3. `src/game/skills.rs` — навыки
4. `src/game/events.rs` — события
5. `src/game/debug_menu.rs` — debug overlay
6. `src/game/save.rs` — сохранения
7. `src/game/main_menu.rs` — главное меню
8. `src/game/interaction.rs` — взаимодействия
9. `src/game/ui.rs` — интерфейс
10. `src/game/inventory.rs` — инвентарь
11. `src/game/mod.rs` — обновлён
12. `assets/vehicles/uaz_patriot/uaz_patriot_2017.vehicle.toml` — UAZ
13. `assets/education/universities.toml` — ВУЗы
14. `assets/ui/ui_config.toml` — UI конфиг

---

## СЛЕДУЮЩИЕ ШАГИ — ФАЗА 1

### Ф1 — Первая играбельная сессия
- [ ] Ф1.1 — Запуск игры → Main Menu → Character Creation → Spawn
- [ ] Ф1.2 — Прогулка по городу (WASD + камера)
- [ ] Ф1.3 — Вход в UAZ Patriot (F) → поездка
- [ ] Ф1.4 — Выход из машины (F) → взаимодействие
- [ ] Ф1.5 — Сохранение на кровати
- [ ] Ф1.6 — Загрузка сохранения

### Ф2 — Экономика и миссии
- [ ] Ф2.1 — Биржа грузов (mission_generator)
- [ ] Ф2.2 — Выполнение контрактов
- [ ] Ф2.3 — Получение денег и опыта
- [ ] Ф2.4 — Покупка топлива, запчастей
- [ ] Ф2.5 — Ремонт техники

### Ф3 — Мир и окружение
- [ ] Ф3.1 — Генерация Новосибирска и области
- [ ] Ф3.2 — Дороги, АЗС, магазины
- [ ] Ф3.3 — NPC, диалоги
- [ ] Ф3.4 — Погода, день/ночь

---

**Дата обновления:** 2025-03-26  
**Статус:** ФАЗА 0 полностью завершена, готова к интеграции и тестированию.
