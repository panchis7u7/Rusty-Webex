use std::fmt;

#[derive(Debug)]
pub enum WebexWebSocketError {
    CreationError,
}

impl std::error::Error for WebexWebSocketError {}

impl fmt::Display for WebexWebSocketError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            WebexWebSocketError::CreationError => write!(f, "Failed to create a new webex device."),
        }
    }
}
