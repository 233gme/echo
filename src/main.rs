use anyhow::Result;
use std::sync::Arc;
use tray_icon::{Menu, MenuItem, TrayIcon, TrayIconBuilder, TrayIconEvent};
use hotkey::{GlobalHotKeyManager, hotkey::{HotKey, Modifiers, KeyCode}};

mod config;
mod audio;
mod capture;

fn main() -> Result<()> {
    // Инициализация горячих клавиш
    let hotkey_manager = GlobalHotKeyManager::new()?;
    let hotkey = HotKey::new(Modifiers::SHIFT | Modifiers::SUPER, KeyCode::KeyE)?;
    hotkey_manager.register(hotkey)?;

    // Создание системного трея
    let menu = Menu::new();
    let quit_item = MenuItem::new("Quit", true, None);
    menu.append(&quit_item)?;

    let tray_icon = TrayIconBuilder::new()
        .with_menu(menu)
        .with_tooltip("Echo")
        .build()?;

    tray_icon.add_event_listener(|event| {
        if let TrayIconEvent::Click { .. } = event {
            println!("Tray icon clicked");
        }
    });

    println!("Echo started! Press Shift+Super+E to trigger capture");

    // Основной цикл приложения
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
