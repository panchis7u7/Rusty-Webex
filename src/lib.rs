use reqwest::{Error, Response};
use service::Service;
use types::{Message, MessageOut};

pub mod adaptive_card;
pub mod service;
pub mod types;

pub struct WebexClient {
    pub bearer_token: String,
}

impl WebexClient {
    // Constructs a new Webex Teams context from a token.
    pub fn new(token: &str) -> Self {
        Self {
            bearer_token: token.to_string(),
        }
    }

    pub async fn send_message(self, message: &MessageOut) -> Response {
        Service::send_message(&self.bearer_token, message).await
    }
}

#[cfg(test)]
mod tests {
    //use super::*;
    //
    //#[test]
    //fn it_works() {
    //    let result = add(2, 2);
    //    assert_eq!(result, 4);
    //}
}
