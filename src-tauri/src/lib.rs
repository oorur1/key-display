// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod gamepad;
use gamepad::Gamepad;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};

#[derive(Serialize, Deserialize)]
pub struct GamepadEvent {
    pub button: u32,
    pub pressed: bool,
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

pub fn send_gamepad_event(app_handle: &AppHandle, button: u32, pressed: bool) {
    let event = GamepadEvent { button, pressed };
    app_handle.emit("gamepad-input", &event).unwrap();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let gamepad = Gamepad::new();
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet])
        .setup(move |app| {
            let app_handle = app.handle().clone();
            gamepad.start_event_loop(app_handle);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
