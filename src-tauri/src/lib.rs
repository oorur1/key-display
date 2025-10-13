// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod database;
mod gamepad;
use database::DatabaseManager;
use gamepad::GamepadManager;
use std::sync::{Arc, Mutex};
use tauri::{Manager, RunEvent};

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn get_statistics(
    date: String,
    db: tauri::State<Arc<Mutex<DatabaseManager>>>,
) -> Result<Option<i32>, String> {
    let db = db
        .lock()
        .map_err(|e| format!("Failed to lock database: {}", e))?;
    db.get(&date).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_year_statistics(
    year: i32,
    db: tauri::State<Arc<Mutex<DatabaseManager>>>,
) -> Result<Vec<(String, i32)>, String> {
    let db = db
        .lock()
        .map_err(|e| format!("Failed to lock database: {}", e))?;
    db.get_year_statistics(year).map_err(|e| e.to_string())
}

#[tauri::command]
fn update_statistics(
    date: String,
    notes_count: i32,
    db: tauri::State<Arc<Mutex<DatabaseManager>>>,
) -> Result<(), String> {
    let db = db
        .lock()
        .map_err(|e| format!("Failed to lock database: {}", e))?;

    // 既存のデータがあれば更新、なければ挿入
    match db.get(&date) {
        Ok(Some(_)) => db.update(&date, notes_count).map_err(|e| e.to_string()),
        Ok(None) => db.insert(&date, notes_count).map_err(|e| e.to_string()),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
fn save_current_count(
    db: tauri::State<Arc<Mutex<DatabaseManager>>>,
    gamepad: tauri::State<Arc<Mutex<GamepadManager>>>,
) -> Result<(), String> {
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let count = {
        let gamepad = gamepad
            .lock()
            .map_err(|e| format!("Failed to lock gamepad:{}", e))?;
        gamepad
            .notes_count()
            .map_err(|e| format!("Failed to get notes_count: {}", e))?
    };
    update_statistics(today, count as i32, db)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            get_statistics,
            get_year_statistics,
            update_statistics,
            save_current_count,
        ])
        .setup(move |app| {
            let app_handle = app.handle().clone();
            let app_local_dir = app.path().local_data_dir()?;
            let db_path = app_local_dir.join("statistics.db");

            // データベースの初期化
            let mut database_manager = DatabaseManager::new(db_path)
                .map_err(|e| format!("Failed to create DatabaseManager: {}", e))?;

            database_manager
                .initialize()
                .map_err(|e| format!("Failed to initialize database: {}", e))?;

            app.manage(Arc::new(Mutex::new(database_manager)));

            let gamepad_manager = GamepadManager::new()
                .map_err(|e| format!("Failed to create GamepadManager: {}", e))?;

            gamepad_manager
                .start_event_loop(app_handle)
                .map_err(|e| format!("Failed to start event loop: {}", e))?;

            app.manage(Arc::new(Mutex::new(gamepad_manager)));
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(|app_handle, event| {
            if let RunEvent::Exit = event {
                println!("Application is exiting, saving statistics...");

                // GamepadManagerとDatabaseManagerを取得
                if let (Some(gamepad_mgr), Some(db_mgr)) = (
                    app_handle.try_state::<Arc<Mutex<GamepadManager>>>(),
                    app_handle.try_state::<Arc<Mutex<DatabaseManager>>>(),
                ) {
                    match save_current_count(db_mgr, gamepad_mgr) {
                        Ok(_) => println!("Statistics saved successfully"),
                        Err(e) => eprintln!("Failed to save statistics: {}", e),
                    }
                }
            }
        });
}
