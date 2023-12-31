// Third party modules.
use async_trait::async_trait;
use futures_util::{self, SinkExt, StreamExt};
use log::{debug, info, warn};
use once_cell::sync::OnceCell;
use reqwest::Client as ReqwestClient;
use rocket::tokio;
use rustls::RootCertStore;
use std::sync::Arc;
use std::sync::{Mutex, MutexGuard};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{connect_async, connect_async_tls_with_config, WebSocketStream};
use url::Url;

// Rusty-webex modules.
use crate::error::WebSocketError;
use crate::types::DeviceDetails;
use crate::types::{RegisterResponse, RemoteTransportWebSocketServer};

#[derive(Debug)]
struct AppState {
    running: bool,
}
static APP_STATE: OnceCell<Mutex<AppState>> = OnceCell::new();

fn access_app_state() -> MutexGuard<'static, AppState> {
    APP_STATE.get().unwrap().lock().unwrap()
}

// TODO: Change the app state when there is a SIGKILL detected.
// fn change_app_state() {
//     let mut app_state = access_app_state();
//     app_state.running = !app_state.running;
// }

// ###################################################################################
// Shortened types.
// ###################################################################################

type ShortenedWebSocketStream =
    WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

// ###################################################################################
// Traits.
// ###################################################################################

#[async_trait]
pub trait Client: Sync + Send {
    async fn connect(&mut self, registration_url: String) -> Result<(), WebSocketError>;
    async fn send(&mut self, text: String) -> Result<(), WebSocketError>;
    async fn listen_for_messages(
        mut self,
        callback: Option<fn(message: serde_json::Value) -> ()>,
    ) -> Result<WebSocketClient, WebSocketError>;
    async fn close(&mut self) -> Result<(), WebSocketError>;
}

#[async_trait]
pub trait TransportClient: Client + Sync + Send {
    async fn register(self, endpoint: String) -> Result<RegisterResponse, WebSocketError>;
    async fn publish(
        &self,
        endpoint: String,
        group: String,
        message: serde_json::Value,
    ) -> Result<(), WebSocketError>;
}

/**
 * Establish a unsecure websocket connection through a ws:// connection string.
 * @param web_socket_url: Url object that contains the ws:// URL connection string.
 *
 * @return Result<WebSocketStream<>, WebexWebSocketError>, a wss_stream if succes and a proper error if not.
 */

pub(crate) async fn connect(
    web_socket_url: Url,
) -> Result<ShortenedWebSocketStream, WebSocketError> {
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
) -> Result<ShortenedWebSocketStream, WebSocketError> {
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

pub struct WebSocketClient {
    _device_info: Option<DeviceDetails>,
    _is_established: bool,
    pub(crate) _websocket_stream: Option<ShortenedWebSocketStream>,
    _is_secure: bool,
    _on_message: fn() -> (),
    _on_card_action: Option<fn() -> ()>,
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
    pub fn new(
        is_secure: bool,
        on_message: fn() -> (),
        on_card_action: Option<fn() -> ()>,
    ) -> WebSocketClient {
        if APP_STATE.get().is_none() {
            APP_STATE
                .set(Mutex::new(AppState { running: true }))
                .unwrap();
        }
        // Create a new websocket client structure.
        WebSocketClient {
            _device_info: None,
            _is_secure: is_secure,
            _is_established: false,
            _websocket_stream: None,
            _on_message: on_message,
            _on_card_action: on_card_action,
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

#[async_trait]
impl Client for WebSocketClient {
    // ------------------------------------------------------------------------------
    /**
     * Function that connects to a websocket endpoint via TLS (secure) transort using
     * HTTPS or unsecurely using HTTP
     *
     * @returns Ok on success and a WebSocketError struct on an Error.
     */
    // ------------------------------------------------------------------------------
    async fn connect(&mut self, registration_url: String) -> Result<(), WebSocketError> {
        // Parse the ws url string to a proper URL structure.
        let parsed_url = Url::parse(&registration_url).unwrap();

        // Create a new websocket connection based on security requirements.
        // TODO: Check for errors at the unwrap stage of the websocket stream creation.
        self._websocket_stream = Some(match self._is_secure {
            true => connect_secure(parsed_url).await.unwrap(),
            false => connect(parsed_url).await.unwrap(),
        });

        Ok(())
    }

    // ------------------------------------------------------------------------------
    /**
     * Function that abstracts direct message send control from end user.
     *
     * @returns ().
     */
    // ------------------------------------------------------------------------------
    async fn send(&mut self, text: String) -> Result<(), WebSocketError> {
        // Check if the websocket stream was succesfully created.
        if self._is_established {
            warn!("[WebSocketClient - close]: Gracefuly closing the websocket.");
            if let Some(mut verified_ws) = self._websocket_stream.take() {
                let _ = verified_ws.send(Message::Text(text)).await;
                return Ok(());
            }
            return Err(WebSocketError::NotDefined);
        }
        Err(WebSocketError::NotDefined)
    }

    // ------------------------------------------------------------------------------
    /**
     * Loop that is constantly listening for new messages on incoming from the connected
     * websocket endpoint stream.
     *
     * @param callback: Optional functional type parameter that represents an end user
     * defined function that is called based on a message reception.
     *
     * @returns None on success and a WebSocketError struct on an Error.
     */
    // ------------------------------------------------------------------------------
    async fn listen_for_messages(
        mut self,
        callback: Option<fn(message: serde_json::Value) -> ()>,
    ) -> Result<WebSocketClient, WebSocketError> {
        // Read for incoming webex messages.
        if self._is_established {
            if let Some(mut verified_ws) = self._websocket_stream.take() {
                loop {
                    if !access_app_state().running {
                        return Ok(self);
                    }
                    tokio::select! {
                        ws_msg = verified_ws.next() => {
                            match ws_msg {
                                Some(msg) => match msg {
                                    Ok(msg) => match msg {
                                        Message::Text(x) => {
                                            debug!("Text message received {:?}",x);
                                            callback.map(|cb| cb(serde_json::to_value(x).unwrap()));
                                        },
                                        Message::Binary(x) => debug!("[WebSocketClient - listen_for_messages]: Binary message received {:?}",x),
                                        Message::Ping(x) => debug!("[WebSocketClient - listen_for_messages]: Ping {:?}",x),
                                        Message::Pong(x) => debug!("[WebSocketClient - listen_for_messages]:mPong {:?}",x),
                                        Message::Close(x) => warn!("[WebSocketClient - listen_for_messages]: Close message received {:?}",x),
                                        Message::Frame(x) => debug!("[WebSocketClient - listen_for_messages]: Frame message received {:?}",x),
                                    }
                                    , Err(_) => { return Err(WebSocketError::AwayError) }
                                },
                                None => {warn!("No message!");}
                            }
                        }
                    }
                }
            }
            return Err(WebSocketError::NotDefined);
        }
        Err(WebSocketError::NotDefined)
    }

    // ------------------------------------------------------------------------------
    /**
     * Function that abstracts websocket close control from end user, it Gracefuly
     * closes the websocket connection.
     *
     * @return ().
     */
    // ------------------------------------------------------------------------------
    async fn close(&mut self) -> Result<(), WebSocketError> {
        if self._is_established {
            if let Some(mut verified_ws) = self._websocket_stream.take() {
                let _ = verified_ws.send(Message::Close(None));
                let close = verified_ws.next().await;
                info!(
                    "[WebSocketClient - close]: Server close message: {:?}",
                    close
                );
                return Ok(());
            }
            return Err(WebSocketError::NotDefined);
        }
        Err(WebSocketError::NotDefined)
    }
}

// ###################################################################################
// Transport WebSocket Client.
// ###################################################################################

pub struct TransportWebSocketClient {
    remote_ws_server: RemoteTransportWebSocketServer,
    _client: ReqwestClient,
    _websocket_impl: WebSocketClient,
    _websocket: Option<ShortenedWebSocketStream>,
}

impl TransportWebSocketClient {
    // ----------------------------------------------------------------------------
    // Function for generating a new websocket client instance struct.
    // ----------------------------------------------------------------------------

    pub fn new(
        remote_ws_server: RemoteTransportWebSocketServer,
        is_secure: bool,
        on_message: fn() -> (),
        on_card_action: Option<fn() -> ()>,
    ) -> TransportWebSocketClient {
        if APP_STATE.get().is_none() {
            APP_STATE
                .set(Mutex::new(AppState { running: true }))
                .unwrap();
        }

        TransportWebSocketClient {
            remote_ws_server,
            _client: ReqwestClient::new(),
            _websocket_impl: WebSocketClient::new(is_secure, on_message, on_card_action),
            _websocket: None,
        }
    }

    // ----------------------------------------------------------------------------
    // Initialize WebSocket Client.
    // ----------------------------------------------------------------------------
    pub async fn connect(&mut self, registration_url: String) -> Result<(), WebSocketError> {
        // Create a new websocket connection based on security requirements.
        // TODO: Check for errors at the unwrap stage of the websocket stream creation.
        let _ = self._websocket_impl.connect(registration_url).await;

        Ok(())
    }
}

#[async_trait]
impl TransportClient for TransportWebSocketClient {
    // ------------------------------------------------------------------------------
    // Register to the Transport Websocket Server.
    // ------------------------------------------------------------------------------
    async fn register(self, endpoint: String) -> Result<RegisterResponse, WebSocketError> {
        let cloned_server_config = self.remote_ws_server.clone();
        Ok(crate::service::websocket::register(endpoint, cloned_server_config).await)
    }

    // ------------------------------------------------------------------------------
    // Publish a message to all endpoints that share the same group.
    // ------------------------------------------------------------------------------
    async fn publish(
        &self,
        endpoint: String,
        group: String,
        message: serde_json::Value,
    ) -> Result<(), WebSocketError> {
        crate::service::websocket::publish(endpoint, group, message, &self.remote_ws_server).await;
        Ok(())
    }
}

#[async_trait]
impl Client for TransportWebSocketClient {
    async fn connect(&mut self, registration_url: String) -> Result<(), WebSocketError> {
        self._websocket_impl.connect(registration_url).await
    }

    async fn send(&mut self, text: String) -> Result<(), WebSocketError> {
        self._websocket_impl.send(text).await
    }

    async fn listen_for_messages(
        mut self,
        callback: Option<fn(message: serde_json::Value) -> ()>,
    ) -> Result<WebSocketClient, WebSocketError> {
        self._websocket_impl.listen_for_messages(callback).await
    }

    async fn close(&mut self) -> Result<(), WebSocketError> {
        self._websocket_impl.close().await
    }
}
