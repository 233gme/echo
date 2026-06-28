# Echo

Приложение для захвата аудио на macOS.

## Установка

```bash
cargo build --release
```

## Использование

- **Hotkey**: Shift+Super+E — начать захват
- **System Tray**: иконка в трее для управления приложением

## Структура

- `src/lib.rs` — публичный API библиотеки
- `src/main.rs` — точка входа приложения
- `src/config.rs` — управление конфигурацией
- `src/audio.rs` — аудио запись
- `src/capture.rs` — захват экрана

## Зависимости

- `screen_captureKit` — захват экрана
- `tray-icon` — системный трея
- `hotkey` — горячие клавиши
- `serde` + `serde_yaml` — конфигурация
- `dirs` — пути к директориям
