use parser::Parser;
use types::{Message, MessageOut};

pub mod adaptive_card;
mod parser;
pub mod service;
pub mod types;

pub struct WebexClient {
    pub bearer_token: String,
    _parser: parser::Parser,
}

impl WebexClient {
    // Constructs a new Webex Teams context from a token.
    pub fn new(token: &str) -> WebexClient {
        WebexClient {
            bearer_token: token.to_string(),
            _parser: Parser::new(),
        }
    }

    // ------------------------------------------------------------------------------
    // Add a command for the webex client to listent to and perform proper parsing.
    // ------------------------------------------------------------------------------

    pub fn add_command(
        &mut self,
        command: &str,
        args: Vec<Box<dyn parser::Argument>>,
        callback: Box<dyn Fn(&parser::ArgTuple, &parser::ArgTuple) -> () + Send>,
    ) {
        self._parser.add_command(command, args, callback);
    }

    // ------------------------------------------------------------------------------
    // Send a webex message.
    // ------------------------------------------------------------------------------

    pub async fn send_message(&self, message: &MessageOut) -> Message {
        service::send_message(&self.bearer_token, message).await
    }

    // ------------------------------------------------------------------------------
    // Retrieve all the information regarding a webex message.
    // ------------------------------------------------------------------------------

    pub async fn get_message_details(&self, message_id: &String) -> Message {
        service::get_message_details(&self.bearer_token, message_id).await
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
