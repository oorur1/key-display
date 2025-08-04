use gilrs_core::{Event, EventType, Gilrs};
use std::sync::{Arc, Mutex};
use std::thread;
use tauri::{AppHandle, Emitter};
pub struct Gamepad {
    pub gilrs: Arc<Mutex<Gilrs>>,
    pub active_gamepad: Arc<Mutex<Option<usize>>>,
}

impl Gamepad {
    pub fn new() -> Gamepad {
        let gilrs = Gilrs::new().unwrap();

        let active_gamepad = None;
        Gamepad {
            gilrs: Arc::new(Mutex::new(gilrs)),
            active_gamepad: Arc::new(Mutex::new(active_gamepad)),
        }
    }

    pub fn start_event_loop(&self, app_handle: AppHandle) {
        let gilrs = Arc::clone(&self.gilrs);
        let active_gamepad = Arc::clone(&self.active_gamepad);

        thread::spawn(move || {
            loop {
                // Examine new events
                if let Ok(mut gilrs) = gilrs.lock() {
                    while let Some(Event {
                        id, event, time, ..
                    }) = gilrs.next_event()
                    {
                        if let Ok(mut active_gamepad) = active_gamepad.lock() {
                            *active_gamepad = Some(id);
                        }
                        match event {
                            EventType::ButtonPressed(button) => {
                                let button_code = button.into_u32();
                                let event = serde_json::json!({
                                    "type": "button",
                                    "button": button_code,
                                    "pressed": true,
                                });

                                app_handle.emit("gamepad-input", &event).unwrap();
                            }
                            EventType::ButtonReleased(button) => {
                                let button_code = button.into_u32();
                                let event = serde_json::json!({
                                    "type": "button",
                                    "button": button_code,
                                    "pressed": false,
                                });

                                app_handle.emit("gamepad-input", &event).unwrap();
                            }
                            EventType::AxisValueChanged(axis, _) => {
                                let event = serde_json::json!({
                                    "type": "scratch",
                                    "axis" : axis,
                                });

                                app_handle.emit("gamepad-input", &event).unwrap();
                            }
                            _ => {}
                        }
                    }
                }
                thread::sleep(std::time::Duration::from_millis(16))
            }
        });
    }
}
