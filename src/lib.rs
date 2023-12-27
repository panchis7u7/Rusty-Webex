// std.
use std::error::Error;
use std::net::TcpStream;
use std::sync::Arc;

use futures_util::join;
// http.
use http::HeaderMap;
use http::HeaderValue;

use log::warn;
// reqwest.
use reqwest::header::ACCEPT;
use reqwest::header::CONTENT_TYPE;
use reqwest::Client;

// rustls.
use rustls::RootCertStore;

// Tokio.
use tokio::sync::mpsc;
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;
use tokio::sync::Mutex;
use tokio_tungstenite::MaybeTlsStream;
use tokio_tungstenite::{
    connect_async, connect_async_tls_with_config, tungstenite::protocol::Message, WebSocketStream,
};

// Rocket.
use rocket::serde::json::Json;
use rocket::{fs::FileServer, get, post, routes, Build, Rocket, State};
use rocket::{tokio, Error as RocketError, Ignite};

// logging.
use log::{debug, error, info};

// Future
use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use types::DeviceDetails;
use types::{Device, DevicesDetails};
use uuid::Uuid;

// Own modules, crates and type imports.
use crate::error::WebexWebSocketError;
use crate::types::{MessageEventResponse, Publish, Register, RegisterResponse, Response};
use parser::Parser;
use types::{Argument, Callback, Message as OwnMessage, MessageOut};
pub mod adaptive_card;
pub mod error;
mod parser;
pub mod service;
pub mod types;

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
        }
    }

    pub async fn launch(
        self,
        on_message: fn() -> (),
        on_card_action: fn() -> (),
    ) -> Result<Rocket<Ignite>, RocketError> {
        let mut webex_ws =
            WebexWebSocketClient::new(self._token.as_str(), None, on_message, on_card_action);
        let _ = webex_ws.run().await;
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
// WebSocket Client.
// ###################################################################################

pub struct WebSocketClient {
    host: String,
    port: u16,
    user_id: u16,
    subscription_groups: Vec<String>,
    _client: Client,
    _headers: HeaderMap,
}

// TODO: Move the WebSocketClient requests to the service.
impl WebSocketClient {
    // ----------------------------------------------------------------------------
    // Function for generating a new websocket client instance struct.
    // ----------------------------------------------------------------------------

    pub fn new(
        host: &str,
        port: u16,
        user_id: u16,
        subscription_groups: Vec<String>,
    ) -> WebSocketClient {
        let mut headers = HeaderMap::new();
        headers.insert(
            CONTENT_TYPE,
            HeaderValue::from_str("application/json").unwrap(),
        );
        headers.insert(ACCEPT, HeaderValue::from_str("application/json").unwrap());

        WebSocketClient {
            host: String::from(host),
            port,
            user_id,
            subscription_groups,
            _client: Client::new(),
            _headers: headers,
        }
    }

    // ----------------------------------------------------------------------------
    // Retrieve a new websocket url from the server.
    // ----------------------------------------------------------------------------
    pub async fn register(&self, endpoint: &str) -> RegisterResponse {
        let response = self
            ._client
            .post(format!("http://{}:{}/{}", self.host, self.port, endpoint))
            .headers(self._headers.clone())
            .json(&Register {
                user_id: self.user_id,
                groups: self.subscription_groups.clone(),
            })
            .send()
            .await
            .unwrap();

        self.review_status(&response);

        let message = response
            .json::<RegisterResponse>()
            .await
            .expect("failed to convert struct from json");

        return message;
    }

    // ----------------------------------------------------------------------------
    // WebSocket message publishing.
    // ----------------------------------------------------------------------------

    pub async fn publish(
        &self,
        endpoint: &str,
        user_id: u16,
        group: String,
        message: serde_json::Value,
    ) {
        let response = self
            ._client
            .post(format!("http://{}:{}/{}", self.host, self.port, endpoint))
            .headers(self._headers.clone())
            .json(&Publish {
                user_id: user_id,
                group,
                message: message.to_string(),
            })
            .send()
            .await
            .unwrap();

        self.review_status(&response);
    }

    // ----------------------------------------------------------------------------
    // Review the status for a given response.
    // ----------------------------------------------------------------------------
    pub fn review_status(&self, response: &reqwest::Response) -> () {
        match response.status() {
            reqwest::StatusCode::OK => {
                log::debug!("Succesful request: {:?}", response)
            }
            reqwest::StatusCode::NOT_FOUND => {
                log::debug!("Got 404! Haven't found resource!: {:?}", response)
            }
            _ => {
                log::error!("Got 404! Haven't found resource!: {:?}", response)
            }
        }
    }

    // ----------------------------------------------------------------------------
    // Initialize WebSocket Client.
    // ----------------------------------------------------------------------------
    pub async fn start_ws_client(
        &self,
        registration_url: String,
    ) -> Result<(Sender<Message>, Receiver<Message>), Box<dyn Error>> {
        // Parse the registration URL as of a URL type.
        let url = url::Url::parse(&registration_url).unwrap();
        debug!("Parsed registration string: {}", url);

        // Create channels to send and receive messages
        let (sender, receiver) = mpsc::channel(32);

        // let (stdin_tx, stdin_rx) = futures_channel::mpsc::unbounded();
        // tokio::spawn(read_stdin(stdin_tx));

        let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
        info!("WebSocket handshake has been successfully completed");

        // Split the WebSocket into sender and receiver.
        let (ws_sender, ws_receiver) = ws_stream.split();

        // Spawn a task to receive messages and forward them to the receiver channel
        tokio::spawn(receive_messages(ws_receiver, sender.clone()));

        // Spawn a task to send messages
        tokio::spawn(send_messages(ws_sender));

        Ok((sender, receiver))
    }
}

// ----------------------------------------------------------------------------
// Function to receive messages from the WebSocket and forward them to the channel.
// ----------------------------------------------------------------------------
async fn receive_messages(
    ws_stream: SplitStream<
        WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
    >,
    sender: Sender<Message>,
) {
    let mut ws_stream = ws_stream;

    while let Some(message) = ws_stream.next().await {
        match message {
            Ok(msg) => {
                // Forward the received message to the channel
                if sender.send(msg).await.is_err() {
                    eprintln!("Receiver dropped, closing connection.");
                    return;
                }
            }
            Err(e) => {
                eprintln!("Error receiving message: {}", e);
            }
        }
    }
}

// ----------------------------------------------------------------------------
// Function to send messages via the WebSocket.
// ----------------------------------------------------------------------------
pub async fn send_messages(
    mut ws_stream: SplitSink<
        WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
        tokio_tungstenite::tungstenite::Message,
    >,
) {
    // This could be a loop where you send messages as needed
    // For the example, we're just sending one message and then exiting
    // let message = Message::Text("Hello, WebSocket Server!".into());

    // if let Err(e) = ws_stream.send(message).await {
    //     eprintln!("Error sending message: {}", e);
    // }
}

// ###################################################################################
// Webex Web Socket Client.
// ###################################################################################

pub(crate) struct WebexWebSocketClient {
    access_token: String,
    webex_client: WebexClient,
    device_url: String,
    device_info: Option<DeviceDetails>,
    on_message: fn() -> (),
    on_card_action: fn() -> (),
}

impl WebexWebSocketClient {
    pub(crate) const DEFAULT_DEVICE_URL: &'static str = "https://wdm-a.wbx2.com/wdm/api/v1";

    pub(crate) fn new(
        access_token: &str,
        device_url: Option<&str>,
        on_message: fn() -> (),
        on_card_action: fn() -> (),
    ) -> WebexWebSocketClient {
        WebexWebSocketClient {
            access_token: String::from(access_token),
            webex_client: WebexClient::new(access_token),
            device_url: String::from(device_url.unwrap_or(Self::DEFAULT_DEVICE_URL)),
            device_info: None,
            on_message: on_message,
            on_card_action: on_card_action,
        }
    }

    // ----------------------------------------------------------------------------
    // Run the websocket client fetch loop.
    // ----------------------------------------------------------------------------
    pub(crate) async fn run(&mut self) -> Result<(), WebexWebSocketError> {
        if self.device_info.is_none() {
            if self.get_device_info_or_create(Some(true)).await.is_err() {
                error!("[WebexWebSocketClient - run]: Unable to fetch or create a new device.");
                return Err(WebexWebSocketError::CreationError);
            }
        }

        // Unwrap since we already verified for any errors.
        let unwrapped_device_info = self.device_info.clone().unwrap();

        // Parse the registration URL as of a URL type.
        let url = url::Url::parse(unwrapped_device_info.web_socket_url.as_str()).unwrap();
        debug!(
            "[WebexWebSocketClient - run]: Parsed registration string: {}",
            url
        );

        // Add certificates from the native certificate store
        let mut roots = RootCertStore::empty();
        for cert in rustls_native_certs::load_native_certs().expect("Could not load platform certs")
        {
            let cert = rustls::Certificate(cert.0);
            roots.add(&cert).unwrap();
        }

        let config = rustls::ClientConfig::builder()
            .with_safe_defaults()
            .with_root_certificates(roots)
            .with_no_client_auth();

        // Create a TlS Connector with the configured root certificates
        let tls_connector = tokio_tungstenite::Connector::Rustls(Arc::new(config));

        // Connect to the websocket using the tls connector configured and wss URL.
        let (mut ws_stream, _) =
            connect_async_tls_with_config(url, None, true, Some(tls_connector))
                .await
                .expect("Failed to connect");

        info!("[WebexWebSocketClient - run]: WebSocket handshake has been successfully completed");
        info!("[WebexWebSocketClient - run]: WebSocket opened!");

        // Generate authentication data.
        let auth_data = &serde_json::json!({
            "id": Uuid::new_v4().to_string(),
            "type": "authorization",
            "data": {"token": format!("Bearer {}", self.access_token)}
        });

        // Print authentication data for verification.
        debug!("Auth data: {}", auth_data);

        // Send authentication data to the webex cloud device.
        let _ = ws_stream.send(Message::Text(auth_data.to_string())).await;

        // Read for incoming webex messages.
        loop {
            tokio::select! {
                ws_msg = ws_stream.next() => {
                    match ws_msg {
                        Some(msg) => match msg {
                            Ok(msg) => match msg {
                                Message::Text(x) => info!("Text message received {:?}",x),
                                Message::Binary(x) => info!("Binary message received {:?}",x),
                                Message::Ping(x) => debug!("Ping {:?}",x),
                                Message::Pong(x) => debug!("Pong {:?}",x),
                                Message::Close(x) => warn!("Close message received {:?}",x),
                                Message::Frame(x) => debug!("Frame message received {:?}",x),
                            }
                            , Err(_) => {error!("Server went away!"); break;}
                        },
                        None => {info!("No message!"); break;}
                    }
                }
            }
        }

        // Gracefuly close the websocket connection.
        let _ = ws_stream.send(Message::Close((None)));
        let close = ws_stream.next().await;
        info!("Server close message: {:?}", close);
        Ok(())
    }

    // ----------------------------------------------------------------------------
    // Get device info from the webex cloud; if it doesn't exist, one will be created.
    // ----------------------------------------------------------------------------
    pub(crate) async fn get_device_info_or_create(
        &mut self,
        check_existing: Option<bool>,
    ) -> Result<DeviceDetails, WebexWebSocketError> {
        let device_data: Device = Device {
            device_name: String::from("rust-websocket-client"),
            device_type: String::from("DESKTOP"),
            localized_model: String::from("rust"),
            model: String::from("rust"),
            name: String::from("rust-spark-client"),
            system_name: String::from("rust-spark-client"),
            system_version: String::from("0.1"),
        };

        // Check if a existing device is related to the specific token.
        if check_existing.unwrap_or(true) {
            debug!("[WebexWebSocketClient - get_device_info_or_create]: Retrieving device list");
            let devices = self.webex_client.get_devices().await;
            if devices.is_some() {
                let devices = devices.unwrap();
                for device in devices.devices {
                    if device.name == device_data.name {
                        debug!("[WebexWebSocketClient - get_device_info_or_create]: Device information: {}", device.name);
                        self.device_info = Some(device.clone());
                        return Ok(device);
                    }
                }
            }
        }

        info!("[WebexWebSocketClient - get_device_info_or_create]: Device does not exist, creating...");

        // Create a new device.
        let new_device = self.webex_client.create_device(device_data).await;
        if new_device.is_none() {
            return Err(WebexWebSocketError::CreationError);
        }
        self.device_info = new_device.clone();

        info!("[WebexWebSocketClient - get_device_info_or_create]: Registered new device.");

        Ok(new_device.unwrap().clone())
    }

    // ----------------------------------------------------------------------------
    /**
    * In order to geo-locate the correct DC to fetch the message from, you need to use the base64 Id of the
    message.
    @param activity: incoming websocket data
    @return: base 64 message id
    */
    // ----------------------------------------------------------------------------
    pub(crate) async fn get_base64_message_id(self) {}
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
