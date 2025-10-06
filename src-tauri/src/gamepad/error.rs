use std::fmt;

#[derive(Debug)]
pub enum GamepadError {
    GilrsError(String),
    LockError(String),
    EmitError(String),
    ThreadError(String),
}

impl std::error::Error for GamepadError {}

impl fmt::Display for GamepadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            GamepadError::GilrsError(msg) => write!(f, "Gilrs error: {}", msg),
            GamepadError::LockError(msg) => write!(f, "Lock error: {}", msg),
            GamepadError::EmitError(msg) => write!(f, "Emit error: {}", msg),
            GamepadError::ThreadError(msg) => write!(f, "Thread error: {}", msg),
        }
    }
}
