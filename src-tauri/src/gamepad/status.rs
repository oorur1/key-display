use super::button::ButtonManager;
use super::error::GamepadError;
use super::scratch::{Direction, ScratchEvent};
use super::statistics::GameStatistics;
use std::time::Duration;

#[derive(Debug)]
pub struct GamepadStatus {
    button_manager: ButtonManager,
    scratch_event: ScratchEvent,
    statistics: GameStatistics,
}

impl GamepadStatus {
    pub fn new() -> Self {
        Self {
            button_manager: ButtonManager::new(),
            scratch_event: ScratchEvent::new(),
            statistics: GameStatistics::new(),
        }
    }

    pub fn on_button_press(&mut self, button_code: u32) -> Result<u32, GamepadError> {
        self.button_manager.on_press(button_code)?;
        self.statistics.increment_notes();
        Ok(self.statistics.notes_count())
    }

    pub fn on_button_release(&mut self, button_code: u32) -> Result<(u32, Duration), GamepadError> {
        if let Some(release_time) = self.button_manager.on_release(button_code)? {
            self.statistics.add_release_time(release_time);
        }

        Ok((
            self.statistics.notes_count(),
            self.statistics.average_release_time(),
        ))
    }

    pub fn on_scratch_spin(
        &mut self,
        new_axis: i32,
    ) -> Result<(Option<Direction>, u32), GamepadError> {
        let direction = self.scratch_event.on_spin(new_axis)?;

        if let Some(ref dir) = direction {
            if *dir != Direction::Neutral {
                self.statistics.increment_notes();
            }
        }

        Ok((direction, self.statistics.notes_count()))
    }

    pub fn scratch_event(&self) -> &ScratchEvent {
        &self.scratch_event
    }

    pub fn scratch_event_mut(&mut self) -> &mut ScratchEvent {
        &mut self.scratch_event
    }

    pub fn notes_count(&self) -> u32 {
        self.statistics.notes_count()
    }

    pub fn difference_notes_count(&self) -> u32 {
        self.statistics.difference_notes_count()
    }

    pub fn set_last_saved_count(&mut self) {
        self.statistics.set_last_saved_count();
    }

    pub fn average_release_time(&self) -> Duration {
        self.statistics.average_release_time()
    }
}
