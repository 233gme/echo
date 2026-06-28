use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tao::event_loop::{ControlFlow, EventLoop};
use tao::window::WindowBuilder;
use tray_icon::{
    TrayIconBuilder,
    menu::{Menu, MenuItem, PredefinedMenuItem},
};
use global_hotkey::{
    GlobalHotKeyEvent, GlobalHotKeyManager,
    hotkey::{HotKey, Modifiers, Code},
};
use notify_rust::Notification;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

use echo::config::AppConfig;

#[derive(Debug, Clone)]
enum AppState {
    Idle,
    Recording { start_time: Instant, file_path: String },
    Processing { stage: String, progress: f32 },
    Error(String),
}

#[derive(Debug, Clone)]
enum MenuAction {
    StartRecording,
    StopRecording,
    ProcessingDone,
    ProcessingError(String),
    OpenSettings,
    OpenFolder,
    OpenObsidian,
    Quit,
}

// Храним MenuItem напрямую — у него есть set_enabled
struct MenuItems {
    start_recording: MenuItem,
    stop_recording: MenuItem,
    open_settings: MenuItem,
    open_folder: MenuItem,
    open_obsidian: MenuItem,
    quit: MenuItem,
    menu: Menu,
}

fn main() {
    env_logger::init();

    let config = AppConfig::load();
    config.ensure_dirs();

    let event_loop = EventLoop::new();

    let _window = WindowBuilder::new()
        .with_visible(false)
        .build(&event_loop)
        .unwrap();

    let menu_items = create_menu_items();
    let tray_icon = create_tray_icon(&menu_items);

    let hotkey_manager = GlobalHotKeyManager::new().unwrap();
    let hotkey = HotKey::new(
        Some(Modifiers::META | Modifiers::SHIFT),
        Code::KeyR,
    );
    hotkey_manager.register(hotkey).unwrap();

    let state = Arc::new(std::sync::Mutex::new(AppState::Idle));
    let recording = Arc::new(AtomicBool::new(false));

    let (tx, mut rx) = mpsc::unbounded_channel::<MenuAction>();

    let rt = tokio::runtime::Runtime::new().unwrap();

    // Клонируем ID для сравнения в event loop
    let start_id = menu_items.start_recording.id().clone();
    let stop_id = menu_items.stop_recording.id().clone();
    let settings_id = menu_items.open_settings.id().clone();
    let folder_id = menu_items.open_folder.id().clone();
    let obsidian_id = menu_items.open_obsidian.id().clone();
    let quit_id = menu_items.quit.id().clone();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::WaitUntil(Instant::now() + Duration::from_millis(100));

        // Горячие клавиши
        if let Ok(event) = GlobalHotKeyEvent::receiver().try_recv() {
            if event.state == global_hotkey::HotKeyState::Pressed {
                let rec = recording.load(Ordering::SeqCst);
                if rec {
                    tx.send(MenuAction::StopRecording).unwrap();
                } else {
                    tx.send(MenuAction::StartRecording).unwrap();
                }
            }
        }

        // Клики по меню
        if let Ok(event) = tray_icon::menu::MenuEvent::receiver().try_recv() {
            let menu_id = event.id;

            if menu_id == start_id {
                if !recording.load(Ordering::SeqCst) {
                    tx.send(MenuAction::StartRecording).unwrap();
                }
            } else if menu_id == stop_id {
                if recording.load(Ordering::SeqCst) {
                    tx.send(MenuAction::StopRecording).unwrap();
                }
            } else if menu_id == settings_id {
                tx.send(MenuAction::OpenSettings).unwrap();
            } else if menu_id == folder_id {
                tx.send(MenuAction::OpenFolder).unwrap();
            } else if menu_id == obsidian_id {
                tx.send(MenuAction::OpenObsidian).unwrap();
            } else if menu_id == quit_id {
                tx.send(MenuAction::Quit).unwrap();
            }
        }

        // Обработка действий
        while let Ok(action) = rx.try_recv() {
            match action {
                MenuAction::StartRecording => {
                    if !recording.load(Ordering::SeqCst) {
                        recording.store(true, Ordering::SeqCst);
                        let file_path = config.get_recording_path();
                        *state.lock().unwrap() = AppState::Recording {
                            start_time: Instant::now(),
                            file_path: file_path.clone(),
                        };
                        update_tray_icon(&tray_icon, &state);
                        update_menu_state(&menu_items, true);
                        show_notification("Echo", "Запись начата — ⌘+⇧+R для остановки");

                        rt.spawn(async move {
                            echo::audio::start_recording(file_path).await;
                        });
                    }
                }
                MenuAction::StopRecording => {
                    if recording.load(Ordering::SeqCst) {
                        recording.store(false, Ordering::SeqCst);
                        echo::audio::stop_recording();

                        let file_path = match &*state.lock().unwrap() {
                            AppState::Recording { file_path, .. } => file_path.clone(),
                            _ => config.get_recording_path(),
                        };

                        *state.lock().unwrap() = AppState::Processing {
                            stage: "Транскрибация".to_string(),
                            progress: 0.0,
                        };
                        update_tray_icon(&tray_icon, &state);
                        update_menu_state(&menu_items, false);
                        show_notification("Echo", "Запись завершена — обработка...");

                        let tx_clone = tx.clone();
                        rt.spawn(async move {
                            match send_to_backend(file_path).await {
                                Ok(_) => tx_clone.send(MenuAction::ProcessingDone).unwrap(),
                                Err(e) => tx_clone.send(MenuAction::ProcessingError(e.to_string())).unwrap(),
                            }
                        });
                    }
                }
                MenuAction::ProcessingDone => {
                    *state.lock().unwrap() = AppState::Idle;
                    update_tray_icon(&tray_icon, &state);
                    show_notification("Echo", "✅ Готово — проверь Obsidian");
                }
                MenuAction::ProcessingError(err) => {
                    *state.lock().unwrap() = AppState::Error(err.clone());
                    update_tray_icon(&tray_icon, &state);
                    show_notification("Echo", &format!("❌ Ошибка: {}", err));
                }
                MenuAction::OpenSettings => open_settings(&config),
                MenuAction::OpenFolder => open_folder(&config.recordings_dir),
                MenuAction::OpenObsidian => open_obsidian(&config),
                MenuAction::Quit => *control_flow = ControlFlow::Exit,
            }
        }

        match event {
            tao::event::Event::WindowEvent { event, .. } => {
                if let tao::event::WindowEvent::CloseRequested = event {
                    *control_flow = ControlFlow::Exit;
                }
            }
            _ => {}
        }
    });
}

fn create_menu_items() -> MenuItems {
    let menu = Menu::new();

    let start_recording = MenuItem::new("🔴 Начать запись ⌘⇧R", true, None);
    let stop_recording = MenuItem::new("⏹️ Остановить запись", false, None);
    let open_settings = MenuItem::new("⚙️ Настройки...", true, None);
    let open_folder = MenuItem::new("📂 Открыть папку", true, None);
    let open_obsidian = MenuItem::new("📖 Открыть Obsidian", true, None);
    let quit = MenuItem::new("❌ Выход", true, None);

    menu.append(&start_recording).unwrap();
    menu.append(&stop_recording).unwrap();
    menu.append(&PredefinedMenuItem::separator()).unwrap();
    menu.append(&open_settings).unwrap();
    menu.append(&open_folder).unwrap();
    menu.append(&open_obsidian).unwrap();
    menu.append(&PredefinedMenuItem::separator()).unwrap();
    menu.append(&quit).unwrap();

    MenuItems {
        start_recording,
        stop_recording,
        open_settings,
        open_folder,
        open_obsidian,
        quit,
        menu,
    }
}

fn create_tray_icon(menu_items: &MenuItems) -> tray_icon::TrayIcon {
    let icon = load_icon("idle");

    TrayIconBuilder::new()
        .with_menu(Box::new(menu_items.menu.clone()))
        .with_tooltip("Echo — Meeting Assistant")
        .with_icon(icon)
        .build()
        .unwrap()
}

fn update_tray_icon(tray_icon: &tray_icon::TrayIcon, state: &Arc<std::sync::Mutex<AppState>>) {
    let (icon, tooltip) = match &*state.lock().unwrap() {
        AppState::Idle => (load_icon("idle"), "Echo — Ожидание".to_string()),
        AppState::Recording { .. } => (load_icon("recording"), "Echo — Запись...".to_string()),
        AppState::Processing { stage, .. } => (load_icon("processing"), format!("Echo — {}", stage)),
        AppState::Error(_) => (load_icon("error"), "Echo — Ошибка".to_string()),
    };
    tray_icon.set_icon(Some(icon)).unwrap();
    tray_icon.set_tooltip(Some(&tooltip)).unwrap();
}

fn update_menu_state(menu_items: &MenuItems, is_recording: bool) {
    // MenuItem имеет set_enabled — вызываем напрямую
    menu_items.start_recording.set_enabled(!is_recording);
    menu_items.stop_recording.set_enabled(is_recording);
}

fn load_icon(state: &str) -> tray_icon::Icon {
    let (r, g, b) = match state {
        "idle" => (76, 175, 80),
        "recording" => (244, 67, 54),
        "processing" => (33, 150, 243),
        "error" => (244, 67, 54),
        _ => (128, 128, 128),
    };

    let size = 32;
    let mut rgba = Vec::with_capacity(size * size * 4);
    for y in 0..size {
        for x in 0..size {
            let dx = x as f32 - size as f32 / 2.0;
            let dy = y as f32 - size as f32 / 2.0;
            let dist = (dx * dx + dy * dy).sqrt();
            let radius = size as f32 / 2.0 - 2.0;

            if dist < radius {
                rgba.push(r);
                rgba.push(g);
                rgba.push(b);
                rgba.push(255);
            } else {
                rgba.push(0);
                rgba.push(0);
                rgba.push(0);
                rgba.push(0);
            }
        }
    }

    tray_icon::Icon::from_rgba(rgba, size as u32, size as u32).unwrap()
}

fn show_notification(title: &str, body: &str) {
    Notification::new()
        .summary(title)
        .body(body)
        .timeout(Duration::from_secs(5))
        .show()
        .unwrap();
}

fn open_settings(config: &AppConfig) {
    std::process::Command::new("open")
        .arg(&config.config_path)
        .spawn()
        .unwrap();
}

fn open_folder(path: &std::path::Path) {
    std::process::Command::new("open")
        .arg(path)
        .spawn()
        .unwrap();
}

fn open_obsidian(config: &AppConfig) {
    let vault_path = &config.obsidian_vault;
    std::process::Command::new("open")
        .arg(format!("obsidian://open?vault={}", vault_path.display()))
        .spawn()
        .unwrap();
}

async fn send_to_backend(file_path: String) -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let response = client
        .post("http://127.0.0.1:8000/api/process")
        .json(&serde_json::json!({"file_path": file_path}))
        .send()
        .await?;

    if response.status().is_success() {
        Ok(())
    } else {
        Err(format!("Backend error: {}", response.status()).into())
    }
}
