use std::sync::{Arc, Mutex};

use log::error;
use parser::Parser;
use rocket::serde::json::Json;
use rocket::{fs::FileServer, get, post, routes, Build, Rocket, State};
use types::{Message, MessageOut};

use crate::types::{MessageEventResponse, Response};

pub mod adaptive_card;
mod parser;
pub mod service;
pub mod types;

// ###################################################################################
// Client that manages all interaction with the webex API's.
// ###################################################################################

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

// ###################################################################################
// Server that handdles all incoming bot requests and handles.
// ###################################################################################

struct WebexBotState {
    client: WebexClient,
    parser: Mutex<Parser>,
}

pub struct WebexBotServer {
    pub server: Rocket<Build>,
}

impl<'a> WebexBotServer {
    // ------------------------------------------------------------------------------
    // Create a new instance of the webex server.
    // ------------------------------------------------------------------------------

    pub fn new(token: &str) -> WebexBotServer {
        WebexBotServer {
            server: rocket::build()
                .mount("/", routes![signature, webhook_listener])
                .mount("/public", FileServer::from("static/"))
                .manage(Arc::new(WebexBotState {
                    client: WebexClient::new(token),
                    parser: Parser::new().into(),
                })),
        }
    }

    // ------------------------------------------------------------------------------
    // Add a command for the webex client to listent to and perform proper parsing.
    // ------------------------------------------------------------------------------

    pub fn add_command(
        mut self,
        command: &str,
        args: Vec<Box<dyn parser::Argument>>,
        callback: Box<
            dyn Fn(&WebexClient, &Message, &parser::ArgTuple, &parser::ArgTuple) -> ()
                + Send
                + Sync,
        >,
    ) {
        self.server
            .state::<WebexBotState>()
            .unwrap()
            .parser
            .lock()
            .unwrap()
            .add_command(command, args, callback);
    }
}

// #########################################################################################
// Signature for bot.
// #########################################################################################

#[get("/")]
pub fn signature() -> &'static str {
    "WebexBot Server"
}

// #########################################################################################
// Webhook root listener.
// #########################################################################################

#[post("/cats/futbolito", format = "json", data = "<data>")]
async fn webhook_listener(
    data: Json<Response<MessageEventResponse>>,
    state: &State<WebexBotState>,
) -> () {
    // Retrieve message details as this contains the text for the bot call.
    let detailed_message_info = state.client.get_message_details(&data.data.id).await;

    // Log the detailed message contents.
    log::info!("[Message info]: {:?}\n", &detailed_message_info);

    // Parse the actual plain text data/message.
    let raw_message = detailed_message_info.text.clone().unwrap();
    let parser = state.parser.lock().unwrap();
    let parsed_value = parser.parse(raw_message);

    // Check if the match was successful and execute the callback.
    match parsed_value {
        Ok(v) => (v.callback)(
            &state.client,
            &detailed_message_info,
            &v.required_arguments,
            &v.optional_arguments,
        ),
        Err(e) => {
            error!("{}", e);
        }
    }
}

// ###################################################################################
// Unit tests.
// ###################################################################################

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
