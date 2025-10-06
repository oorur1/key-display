use super::error::GamepadError;
use std::time::{Duration, SystemTime};

#[derive(Debug)]
struct ButtonEvent {
    pressed_count: u32,
    pressed_time: Option<SystemTime>,
}

impl ButtonEvent {
    fn new() -> ButtonEvent {
        ButtonEvent {
            pressed_count: 0,
            pressed_time: None,
        }
    }

    fn on_press(&mut self) -> Result<(), GamepadError> {
        self.pressed_count += 1;
        self.pressed_time = Some(SystemTime::now());
        Ok(())
    }

    fn on_release(&mut self) -> Result<Option<Duration>, GamepadError> {
        if let Some(press_time) = self.pressed_time.take() {
            if let Ok(duration) = press_time.elapsed() {
                if duration < Duration::from_millis(200) {
                    return Ok(Some(duration));
                } else {
                    return Ok(None);
                }
            }
        }
        Err(GamepadError::GilrsError(String::from(
            "You released the button, but pressed time does not exit",
        )))
    }
}

#[derive(Debug)]
pub struct ButtonManager {
    buttons: std::collections::HashMap<u32, ButtonEvent>,
}

impl ButtonManager {
    pub fn new() -> ButtonManager {
        ButtonManager {
            buttons: std::collections::HashMap::new(),
        }
    }

    pub fn on_press(&mut self, button_code: u32) -> Result<(), GamepadError> {
        let button_event = self
            .buttons
            .entry(button_code)
            .or_insert_with(ButtonEvent::new);
        button_event.on_press()
    }
    pub fn on_release(&mut self, button_code: u32) -> Result<Option<Duration>, GamepadError> {
        self.buttons
            .get_mut(&button_code)
            .ok_or_else(|| GamepadError::GilrsError(format!("Button {} not found", button_code)))?
            .on_release()
    }
}
