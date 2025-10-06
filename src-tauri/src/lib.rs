// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod gamepad;
use gamepad::GamepadManager;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet])
        .setup(move |app| {
            let app_handle = app.handle().clone();

            let gamepad_manager = GamepadManager::new()
                .map_err(|e| format!("Failed to create GamepadManager: {}", e))?;

            gamepad_manager
                .start_event_loop(app_handle)
                .map_err(|e| format!("Failed to start event loop: {}", e))?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
