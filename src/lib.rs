// Third party modules.
use error::WebexClientError;
use log::{debug, error, info};
use rocket::serde::json::Json;
use rocket::Ignite;
use rocket::{fs::FileServer, get, post, routes, Build, Error as RocketError, Rocket, State};
use std::error::Error;
use std::sync::Arc;
use tokio::sync::Mutex;
use types::DeviceDetails;
use types::{Device, DevicesDetails};
use uuid::Uuid;
use websocket::PureClient;
use websocket::WebSocketClient;

// Rusty-webex modules.
use crate::types::{MessageEventResponse, Response};
use parser::Parser;
use types::{Argument, Callback, Message as OwnMessage, MessageOut};
pub mod adaptive_card;
pub mod error;
mod parser;
pub mod service;
pub mod types;
mod utils;
pub mod websocket;

// ###################################################################################
// Client that manages all interaction with the webex API's.
// ###################################################################################

#[derive(Clone)]
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

    pub async fn send_message(&self, message: &MessageOut) -> OwnMessage {
        service::send_message(&self.bearer_token, message).await
    }

    // ------------------------------------------------------------------------------
    // Retrieve all the information regarding a webex message.
    // ------------------------------------------------------------------------------

    pub async fn get_message_details(&self, message_id: &String) -> OwnMessage {
        service::get_message_details(&self.bearer_token, message_id).await
    }

    // ------------------------------------------------------------------------------
    // Retrieve all the registered devices.
    // ------------------------------------------------------------------------------

    pub(crate) async fn get_devices(&self) -> Option<DevicesDetails> {
        service::get_devices(&self.bearer_token).await
    }

    // ------------------------------------------------------------------------------
    // Create a new device.
    // ------------------------------------------------------------------------------

    pub(crate) async fn create_device(&self, device: Device) -> Option<DeviceDetails> {
        service::create_device(&self.bearer_token, device).await
    }

    // ----------------------------------------------------------------------------
    // Get device info from the webex cloud; if it doesn't exist, one will be created.
    // ----------------------------------------------------------------------------
    pub(crate) async fn get_device_info_or_create(
        &mut self,
        check_existing: Option<bool>,
    ) -> Result<DeviceDetails, WebexClientError> {
        // Default device information for the websocket communication.
        let auth_data = &serde_json::json!({
            "deviceName": "rust-websocket-client",
            "deviceType": "DESKTOP",
            "localizedModel": "rust",
            "model": "rust",
            "name": "rust-spark-client",
            "systemName": "rust-spark-client",
            "systemVersion": "0.1"
        });
        let device_data: Device = serde_json::from_value(auth_data.to_owned()).unwrap();

        // Check if a existing device is related to the specific token.
        if check_existing.unwrap_or(true) {
            debug!("[WebexWebSocketClient - get_device_info_or_create]: Retrieving device list.");
            let devices = self.get_devices().await;
            if devices.is_some() {
                let devices = devices.unwrap();
                for device in devices.devices {
                    if device.name == device_data.name {
                        debug!("[WebexWebSocketClient - get_device_info_or_create]: Device information: {}", device.name);
                        return Ok(device);
                    }
                }
            }
        }

        info!("[WebexWebSocketClient - get_device_info_or_create]: Device does not exist, creating...");

        // Create a new device.
        let new_device = self.create_device(device_data).await;
        if new_device.is_none() {
            return Err(WebexClientError::CreationError);
        }

        info!("[WebexWebSocketClient - get_device_info_or_create]: Registered new device.");

        Ok(new_device.unwrap().clone())
    }
}

// ###################################################################################
// Server that handdles all incoming bot requests and handles.
// ###################################################################################

struct WebexBotState {
    client: WebexClient,
    parser: Arc<Mutex<Parser>>,
}

pub struct WebexBotServer {
    _server: Rocket<Build>,
    _token: String,
    _device_info: Option<DeviceDetails>,
}

impl<'a> WebexBotServer {
    // ------------------------------------------------------------------------------
    // Create a new instance of the webex server.
    // ------------------------------------------------------------------------------

    pub fn new(token: &str) -> WebexBotServer {
        WebexBotServer {
            _server: rocket::build()
                .mount("/", routes![signature, webhook_listener])
                .mount("/public", FileServer::from("static/"))
                .manage(WebexBotState {
                    client: WebexClient::new(token),
                    parser: Arc::new(Mutex::new(Parser::new())),
                }),
            _token: String::from(token),
            _device_info: None,
        }
    }

    // ------------------------------------------------------------------------------
    // Execute a websocket bot server.
    // ------------------------------------------------------------------------------

    pub async fn websocket_run(
        &mut self,
        on_message: fn() -> (),
        on_card_action: fn() -> (),
    ) -> Result<(), Box<dyn Error>> {
        // Create or retrieve an already existen webex cloud device.
        if self._device_info.is_none() {
            let mut client = WebexClient::new(&self._token);
            let device = client.get_device_info_or_create(Some(true)).await;
            if device.is_err() {
                error!("[WebexBotServer - run]: Unable to fetch or create a new device.");
                return Err(Box::new(WebexClientError::CreationError));
            }
            // Save and persist the fetched device.
            self._device_info = Some(device.unwrap());
        }

        // Create the websocket connection.
        let mut webex_websocket = WebSocketClient::new(true, on_message, Some(on_card_action));

        // Connect to the remote resource.
        let _ = webex_websocket.connect(self._device_info.as_ref().unwrap().web_socket_url.clone());

        // Generate authentication data.
        let auth_data = &serde_json::json!({
            "id": Uuid::new_v4().to_string(),
            "type": "authorization",
            "data": {"token": format!("Bearer {}", self._token)}
        });

        // Print authentication data for verification.
        debug!("Auth data: {}", auth_data);

        // Send authentication data to the webex cloud device.
        let _ = webex_websocket.send(auth_data.to_string()).await;

        // Start listening for messages.
        let ws = webex_websocket.listen_for_messages(None).await;

        // close the websocket session gracefully.
        let _ = ws.unwrap().close().await;
        Ok(())
    }

    // ------------------------------------------------------------------------------
    // Execute an HTTP - webhook based bot server.
    // ------------------------------------------------------------------------------

    pub async fn run(self) -> Result<Rocket<Ignite>, RocketError> {
        self._server.launch().await
    }

    // ------------------------------------------------------------------------------
    // Add a command for the webex client to listent to and perform proper parsing.
    // ------------------------------------------------------------------------------

    pub async fn add_command(
        &'a self,
        command: &str,
        args: Vec<Box<dyn Argument>>,
        callback: Callback,
    ) {
        let server = self
            ._server
            .state::<WebexBotState>()
            .unwrap()
            .parser
            .clone();
        let mut server_unlock = server.lock().await;
        server_unlock.add_command(command, args, callback);
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
    let parser = state.parser.clone();
    let parsed_value_unlock = parser.lock().await;
    let parsed_value = parsed_value_unlock.parse(raw_message);

    // Check if the match was successful and execute the callback.
    match parsed_value {
        Ok(v) => {
            (v.callback)(
                state.client.clone(),
                detailed_message_info,
                v.required_arguments,
                v.optional_arguments,
            )
            .await
        }
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
