// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod gamepad;
mod database;
use gamepad::GamepadManager;
use database::DatabaseManager;
use tauri::Manager;
use std::sync::{Arc, Mutex};

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn get_statistics(
    date: String,
    db: tauri::State<Arc<Mutex<DatabaseManager>>>,
) -> Result<Option<i32>, String>{
    let db = db.lock().map_err(|e| format!("Failed to lock database: {}", e))?;
    db.get(&date).map_err(|e| e.to_string())
}

#[tauri::command]
fn update_statistics(
    date: String,
    notes_count: i32,
    db: tauri::State<Arc<Mutex<DatabaseManager>>>,
) -> Result<(), String> {
    let db = db.lock().map_err(|e| format!("Failed to lock database: {}", e))?;
    
    // 既存のデータがあれば更新、なければ挿入
    match db.get(&date) {
        Ok(Some(_)) => db.update(&date, notes_count).map_err(|e| e.to_string()),
        Ok(None) => db.insert(&date, notes_count).map_err(|e| e.to_string()),
        Err(e) => Err(e.to_string()),
    }
}


#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            get_statistics,
            update_statistics
        ])
        .setup(move |app| {
            let app_handle = app.handle().clone();
            let app_local_dir = app.path().local_data_dir()?;
            let db_path = app_local_dir.join("statistics.db");

            // データベースの初期化
            let mut database_manager = DatabaseManager::new(db_path)
            .map_err(|e| format!("Failed to create DatabaseManager: {}", e))?;

            database_manager.initialize()
            .map_err(|e| format!("Failed to initialize database: {}", e))?;
            
            app.manage(Arc::new(Mutex::new(database_manager)));

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
