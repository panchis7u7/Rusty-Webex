// Third party modules.
use futures_util::{SinkExt, StreamExt};
use log::{debug, info, warn};
use reqwest::Client;
use rocket::tokio;
use rustls::RootCertStore;
use std::sync::Arc;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{connect_async, connect_async_tls_with_config, WebSocketStream};
use url::Url;

// Rusty-webex modules.
use crate::types::DeviceDetails;
use crate::types::{RegisterResponse, RemoteTransportWebSocketServer};
use crate::{error::WebSocketError, WebexClient};

/**
 * Establish a unsecure websocket connection through a ws:// connection string.
 * @param web_socket_url: Url object that contains the ws:// URL connection string.
 *
 * @return Result<WebSocketStream<>, WebexWebSocketError>, a wss_stream if succes and a proper error if not.
 */

pub(crate) async fn connect(
    web_socket_url: Url,
) -> Result<WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>, WebSocketError>
{
    // Connect to the websocket using the ws URL.
    let (ws_stream, _) = connect_async(web_socket_url)
        .await
        .expect("Failed to connect");

    info!("[WebexWebSocketClient - connect]: Insecure WebSocket handshake has been successfully completed.");
    info!("[WebexWebSocketClient - connect]: Insecure WebSocket opened!");

    Ok(ws_stream)
}

/**
 * Establish a secure websocket connection through a wss:// connection string.
 * @param web_socket_url: Url object that contains the wss:// URL connection string.
 *
 * @return Result<WebSocketStream<>, WebexWebSocketError>, a wss_stream if succes and a proper error if not.
 */

pub(crate) async fn connect_secure(
    web_socket_url: Url,
) -> Result<WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>, WebSocketError>
{
    // Add certificates from the native certificate store
    let mut roots = RootCertStore::empty();
    for cert in rustls_native_certs::load_native_certs().expect("Could not load platform certs") {
        let cert = rustls::Certificate(cert.0);
        roots.add(&cert).unwrap();
    }

    // Use the platform certs as part of the TLS client configuration.
    let config = rustls::ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(roots)
        .with_no_client_auth();

    // Create a TlS Connector with the configured root certificates
    let tls_connector = tokio_tungstenite::Connector::Rustls(Arc::new(config));

    // Connect to the websocket using the tls connector configured and wss URL.
    let (wss_stream, _) =
        connect_async_tls_with_config(web_socket_url, None, true, Some(tls_connector))
            .await
            .expect("Failed to connect");

    info!("[WebexWebSocketClient - connect_secure]: Secure WebSocket handshake has been successfully completed");
    info!("[WebexWebSocketClient - connect_secure]: Secure WebSocket opened!");

    Ok(wss_stream)
}

// ###################################################################################
// Webex Web Socket Client.
// ###################################################################################

pub(crate) struct WebSocketClient {
    _access_token: String,
    _webex_client: WebexClient,
    _device_info: Option<DeviceDetails>,
    pub(crate) _websocket:
        WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
    _on_message: fn() -> (),
    _on_card_action: fn() -> (),
}
impl WebSocketClient {
    // ------------------------------------------------------------------------------
    /**
     * Loop that blocks itself until an arrival of a new websocket message.
     * @param access_token: Bot access token for authentication.
     * @param url: Url object that contains the websocket connection string.
     * @is_secure: Manages secure or unsecure communication state over the websocket.
     * @on_message: Callback function that is called when the websocket receives a
     * new message.
     * @on_card_action: Callback function that is called when an AdaptiveCard event
     * has been triggered from a WebEx space.
     *
     * @return WebSocketClient
     */
    // ------------------------------------------------------------------------------
    pub async fn new(
        access_token: &str,
        url: String,
        is_secure: bool,
        on_message: fn() -> (),
        on_card_action: fn() -> (),
    ) -> WebSocketClient {
        let websocket_stream;

        // Parse the registration URL as of a URL type.
        let parsed_url = Url::parse(url.as_str()).unwrap();
        debug!(
            "[WebSocketClient - new]: Parsed registration string: {}",
            parsed_url
        );

        // Create a new websocket connection based on security requirements.
        // TODO: Check for errors at the unwrap stage of the websocket stream creation.
        if is_secure {
            websocket_stream = Some(connect_secure(parsed_url).await.unwrap());
        } else {
            websocket_stream = Some(connect(parsed_url).await.unwrap());
        }

        // Create a new websocket client structure.
        WebSocketClient {
            _access_token: String::from(access_token),
            _webex_client: WebexClient::new(access_token),
            _device_info: None,
            _websocket: websocket_stream.unwrap(),
            _on_message: on_message,
            _on_card_action: on_card_action,
        }
    }

    // ------------------------------------------------------------------------------
    /**
     * Function that abstracts direct message send control from end user.
     *
     * @return ().
     */
    // ------------------------------------------------------------------------------
    pub async fn send(&mut self, text: String) {
        let _ = self._websocket.send(Message::Text(text)).await;
    }

    // ------------------------------------------------------------------------------
    /**
     * Function that abstracts websocket close control from end user.
     *
     * @return ().
     */
    // ------------------------------------------------------------------------------
    pub async fn close(&mut self) {
        // Gracefuly close the websocket connection.
        let _ = self._websocket.send(Message::Close(None));
        let close = self._websocket.next().await;
        info!(
            "[WebexWebSocketClient - close]: Server close message: {:?}",
            close
        );
    }

    // ------------------------------------------------------------------------------
    /**
     * Loop that blocks itself until an arrival of a new websocket message.
     *
     * @return Result<(), WebexWebSocketError>.
     */
    // ------------------------------------------------------------------------------
    pub(crate) async fn listen_for_messages(&mut self) -> Result<(), WebSocketError> {
        // Read for incoming webex messages.
        loop {
            tokio::select! {
                ws_msg = self._websocket.next() => {
                    match ws_msg {
                        Some(msg) => match msg {
                            Ok(msg) => match msg {
                                Message::Text(x) => debug!("Text message received {:?}",x),
                                Message::Binary(x) => debug!("Binary message received {:?}",x),
                                Message::Ping(x) => debug!("Ping {:?}",x),
                                Message::Pong(x) => debug!("Pong {:?}",x),
                                Message::Close(x) => warn!("Close message received {:?}",x),
                                Message::Frame(x) => debug!("Frame message received {:?}",x),
                            }
                            , Err(_) => { return Err(WebSocketError::AwayError) }
                        },
                        None => {warn!("No message!");}
                    }
                }
            }
        }
    }

    // ----------------------------------------------------------------------------
    /**
    * In order to geo-locate the correct DC to fetch the message from, you need to use the base64 Id of the
    message.
    @param activity: incoming websocket data
    @return: base 64 message id
    */
    // ----------------------------------------------------------------------------
    pub(crate) async fn _get_base64_message_id(self) {}
}

// ###################################################################################
// Transport WebSocket Client.
// ###################################################################################

pub struct TransportWebSocketClient {
    remote_ws_server: RemoteTransportWebSocketServer,
    _client: Client,
    _websocket: Option<WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>>,
}

impl TransportWebSocketClient {
    // ----------------------------------------------------------------------------
    // Function for generating a new websocket client instance struct.
    // ----------------------------------------------------------------------------

    pub fn new(remote_ws_server: RemoteTransportWebSocketServer) -> TransportWebSocketClient {
        TransportWebSocketClient {
            remote_ws_server,
            _client: Client::new(),
            _websocket: None,
        }
    }

    // ------------------------------------------------------------------------------
    // Register to the Transport Websocket Server.
    // ------------------------------------------------------------------------------

    pub async fn register(&self, endpoint: &str) -> RegisterResponse {
        crate::service::websocket::register(endpoint, &self.remote_ws_server).await
    }

    // ------------------------------------------------------------------------------
    // Publish a message to all endpoints that share the same group.
    // ------------------------------------------------------------------------------

    pub async fn publish(&self, endpoint: &str, group: String, message: serde_json::Value) {
        crate::service::websocket::publish(endpoint, group, message, &self.remote_ws_server).await;
    }

    // ----------------------------------------------------------------------------
    // Initialize WebSocket Client.
    // ----------------------------------------------------------------------------
    pub async fn connect(
        &mut self,
        registration_url: String,
        is_secure: bool,
    ) -> Result<(), Box<WebSocketError>> {
        // Parse the ws url string to a proper URL structure.
        let parsed_url = Url::parse(&registration_url).unwrap();

        // Create a new websocket connection based on security requirements.
        // TODO: Check for errors at the unwrap stage of the websocket stream creation.
        if is_secure {
            self._websocket = Some(connect_secure(parsed_url).await.unwrap());
        } else {
            self._websocket = Some(connect(parsed_url).await.unwrap());
        }

        Ok(())
    }

    // ----------------------------------------------------------------------------
    // Function to receive messages from the WebSocket and forward them to the channel.
    // ----------------------------------------------------------------------------
    pub async fn listen_for_messages(
        self,
        callback: fn(message: serde_json::Value) -> (),
    ) -> Result<(), WebSocketError> {
        let mut websocket_stream = self._websocket.unwrap();
        // Read for incoming webex messages.
        loop {
            tokio::select! {
                ws_msg = websocket_stream.next() => {
                    match ws_msg {
                        Some(msg) => match msg {
                            Ok(msg) => match msg {
                                Message::Text(x) => {debug!("Text message received {:?}",x); callback(serde_json::to_value(x).unwrap())},
                                Message::Binary(x) => debug!("Binary message received {:?}",x),
                                Message::Ping(x) => debug!("Ping {:?}",x),
                                Message::Pong(x) => debug!("Pong {:?}",x),
                                Message::Close(x) => warn!("Close message received {:?}",x),
                                Message::Frame(x) => debug!("Frame message received {:?}",x),
                            }
                            , Err(_) => { return Err(WebSocketError::AwayError) }
                        },
                        None => {warn!("No message!");}
                    }
                }
            }
        }
    }
}
