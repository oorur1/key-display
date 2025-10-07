use std::fmt;

#[derive(Debug)]
pub enum DatabaseError {
    ConnectionError(String),
    InitializeError(String),
}

impl std::error::Error for DatabaseError {}

impl fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DatabaseError::ConnectionError(msg) => write!(f, "Connection error: {}", msg),
            DatabaseError::InitializeError(msg) => write!(f, "Initialize error: {}", msg),
        }
    }
}
