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
    pub fn new(token: &str) -> WebexClient {
        WebexClient {
            bearer_token: token.to_string(),
        }
    }

    pub async fn send_message(&self, message: &MessageOut) -> Message {
        Service::send_message(&self.bearer_token, message).await
    }

    pub async fn get_message_details(&self, message_id: &String) -> Message {
        Service::get_message_details(&self.bearer_token, message_id).await
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
