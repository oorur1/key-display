use std::time::Instant;

use crate::gamepad::error::GamepadError;

const AXIS_MIN_VALUE: i32 = -32768;
const AXIS_MAX_VALUE: i32 = 32767;

#[derive(PartialEq, Clone, Debug)]
pub enum Direction {
    Neutral,
    Left,
    Right,
}

impl Direction {
    pub fn as_str(&self) -> &'static str {
        match self {
            Direction::Neutral => "neutral",
            Direction::Left => "left",
            Direction::Right => "right",
        }
    }
}

#[derive(Debug)]
pub struct ScratchEvent {
    spined_time: Option<Instant>,
    direction: Direction,
    axis: i32,
}

impl ScratchEvent {
    // TODO:初期値をGilrsのGamepadから取得する
    pub fn new() -> ScratchEvent {
        ScratchEvent {
            spined_time: None,
            direction: Direction::Neutral,
            axis: 0,
        }
    }

    pub fn on_spin(&mut self, new_axis: i32) -> Result<Option<Direction>, GamepadError> {
        let new_direction = self.calculate_direction(new_axis);

        self.axis = new_axis;
        self.spined_time = Some(Instant::now());

        if self.direction == new_direction {
            return Ok(None);
        } else {
            self.direction = new_direction.clone();
            return Ok(Some(new_direction));
        }
    }

    pub fn reset_to_neutral(&mut self) -> bool {
        if self.direction != Direction::Neutral {
            self.direction = Direction::Neutral;
            self.spined_time = None;
            return true;
        }
        false
    }
    pub fn is_timeout(&self) -> bool {
        if let Some(spined_time) = self.spined_time {
            if self.direction != Direction::Neutral {
                return spined_time.elapsed() >= std::time::Duration::from_millis(100);
            }
        }
        false
    }

    pub fn axis(&self) -> i32 {
        self.axis
    }

    fn calculate_direction(&self, new_axis: i32) -> Direction {
        if new_axis == AXIS_MIN_VALUE && self.axis == AXIS_MAX_VALUE {
            Direction::Left
        } else if new_axis == AXIS_MAX_VALUE && self.axis == AXIS_MIN_VALUE {
            Direction::Right
        } else if new_axis > self.axis {
            Direction::Left
        } else if new_axis < self.axis {
            Direction::Right
        } else {
            Direction::Neutral
        }
    }
}
