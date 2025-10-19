use gilrs_core::{Event, EventType, Gilrs};
use std::fmt::format;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

use super::error::GamepadError;
use super::scratch::Direction;
use super::status::GamepadStatus;

pub struct GamepadManager {
    gilrs: Arc<Mutex<Gilrs>>,
    active_gamepad: Arc<Mutex<Option<usize>>>,
    status: Arc<Mutex<GamepadStatus>>,
    is_running: Arc<AtomicBool>,
}

impl GamepadManager {
    pub fn new() -> Result<GamepadManager, GamepadError> {
        let gilrs = Gilrs::new()
            .map_err(|e| GamepadError::GilrsError(format!("Failed to initialize Gilrs: {}", e)))?;

        Ok(GamepadManager {
            gilrs: Arc::new(Mutex::new(gilrs)),
            active_gamepad: Arc::new(Mutex::new(None)),
            status: Arc::new(Mutex::new(GamepadStatus::new())),
            is_running: Arc::new(AtomicBool::new(true)),
        })
    }

    pub fn start_event_loop(&self, app_handle: AppHandle) -> Result<(), GamepadError> {
        let gilrs = Arc::clone(&self.gilrs);
        let active_gamepad = Arc::clone(&self.active_gamepad);
        let status = Arc::clone(&self.status);
        let is_running = Arc::clone(&self.is_running);

        let main_handle = app_handle.clone();
        let sub_handle = app_handle.clone();

        thread::spawn(move || {
            if let Err(e) =
                Self::run_main_loop(gilrs, active_gamepad, status, main_handle, is_running)
            {
                eprintln!("Main loop error: {}", e);
            }
        });

        // スクラッチタイムアウト用のスレッド
        let status = Arc::clone(&self.status);
        let is_running = Arc::clone(&self.is_running);
        thread::spawn(move || {
            if let Err(e) = Self::run_scratch_timeout_loop(status, sub_handle, is_running) {
                eprintln!("Scratch timeout loop error: {}", e);
            }
        });
        Ok(())
    }

    fn run_main_loop(
        gilrs: Arc<Mutex<Gilrs>>,
        active_gamepad: Arc<Mutex<Option<usize>>>,
        status: Arc<Mutex<GamepadStatus>>,
        app_handle: AppHandle,
        is_running: Arc<AtomicBool>,
    ) -> Result<(), GamepadError> {
        while is_running.load(Ordering::Relaxed) {
            let mut gilrs = gilrs
                .lock()
                .map_err(|e| GamepadError::LockError(format!("Failed to lock gilrs: {}", e)))?;

            while let Some(Event { id, event, .. }) = gilrs.next_event() {
                if !is_running.load(Ordering::Relaxed) {
                    break;
                }

                // アクティブゲームパッドを更新
                if let Ok(mut active) = active_gamepad.lock() {
                    *active = Some(id);
                }

                if let Err(e) = Self::handle_gamepad_event(event, &status, &app_handle) {
                    eprintln!("Event handling error: {}", e);
                }
            }

            thread::sleep(Duration::from_millis(8));
        }
        Ok(())
    }

    fn handle_gamepad_event(
        event: EventType,
        status: &Arc<Mutex<GamepadStatus>>,
        app_handle: &AppHandle,
    ) -> Result<(), GamepadError> {
        let mut status = status
            .lock()
            .map_err(|e| GamepadError::LockError(format!("Failed to lock status: {}", e)))?;

        match event {
            EventType::ButtonPressed(button) => {
                let button_code = button.into_u32();
                let count = status.on_button_press(button_code)?;

                let event = serde_json::json!({
                    "type": "button",
                    "button": button_code,
                    "pressed": true,
                    "count": count,
                });

                app_handle.emit("gamepad-input", &event).map_err(|e| {
                    GamepadError::EmitError(format!("Failed to emit button press: {}", e))
                })?;
            }
            EventType::ButtonReleased(button) => {
                let button_code = button.into_u32();
                let (count, avg_release_time) = status.on_button_release(button_code)?;

                let event = serde_json::json!({
                    "type": "button",
                    "button": button_code,
                    "pressed": false,
                    "count": count,
                    "averageReleaseTime": avg_release_time.as_millis(),
                });

                app_handle.emit("gamepad-input", &event).map_err(|e| {
                    GamepadError::EmitError(format!("Failed to emit button release: {}", e))
                })?;
            }
            EventType::AxisValueChanged(axis, _) => {
                let (direction, count) = status.on_scratch_spin(axis)?;

                if let Some(direction) = direction {
                    let event = serde_json::json!({
                        "type": "scratch",
                        "axis": axis,
                        "direction": direction.as_str(),
                        "count": count,
                    });

                    app_handle.emit("gamepad-input", &event).map_err(|e| {
                        GamepadError::EmitError(format!("Failed to emit scratch: {}", e))
                    })?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn run_scratch_timeout_loop(
        status: Arc<Mutex<GamepadStatus>>,
        app_handle: AppHandle,
        is_running: Arc<AtomicBool>,
    ) -> Result<(), GamepadError> {
        while is_running.load(Ordering::Relaxed) {
            thread::sleep(Duration::from_millis(10));

            let mut status = status.lock().map_err(|e| {
                GamepadError::LockError(format!("Failed to lock status in timeout: {}", e))
            })?;

            if status.scratch_event().is_timeout() {
                if status.scratch_event_mut().reset_to_neutral() {
                    let event = serde_json::json!({
                        "type": "scratch",
                        "axis": status.scratch_event().axis(),
                        "direction": "neutral",
                        "count": status.notes_count(),
                    });

                    app_handle.emit("gamepad-input", &event).map_err(|e| {
                        GamepadError::EmitError(format!("Failed to emit timeout: {}", e))
                    })?;
                }
            }
        }
        Ok(())
    }

    pub fn notes_count(&self) -> Result<u32, GamepadError> {
        let status = self
            .status
            .lock()
            .map_err(|e| GamepadError::LockError(format!("Failed to lock status: {}", e)))?;
        Ok(status.notes_count())
    }

    pub fn difference_notes_count(&self) -> Result<u32, GamepadError> {
        let status = self
            .status
            .lock()
            .map_err(|e| GamepadError::LockError(format!("Failed to lock status: {}", e)))?;

        Ok(status.difference_notes_count())
    }

    pub fn update_last_saved_count(&mut self) -> Result<(), GamepadError> {
        let mut status = self
            .status
            .lock()
            .map_err(|e| GamepadError::LockError(format!("Failed to lock status: {}", e)))?;
        status.set_last_saved_count();
        Ok(())
    }

    pub fn shutdown(&self) {
        self.is_running.store(false, Ordering::Relaxed);
    }
}
