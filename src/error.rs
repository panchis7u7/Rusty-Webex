use std::fmt::{self};

#[derive(Debug)]
pub enum WebSocketError {
    AwayError,
}

impl std::error::Error for WebSocketError {}

impl fmt::Display for WebSocketError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            WebSocketError::AwayError => write!(f, "Web Socket went away."),
        }
    }
}

#[derive(Debug)]
pub enum WebexClientError {
    CreationError,
}

impl std::error::Error for WebexClientError {}

impl fmt::Display for WebexClientError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            WebexClientError::CreationError => write!(f, "Failed to create a new webex device."),
        }
    }
}
