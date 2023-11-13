// TODO: Implement websocket communication to webex.

// async.
use async_std::sync::Mutex;
use async_std::task;

// futurse.
use futures::SinkExt;

// serde.
use serde_json::json;

// std.
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

// warp
use warp::Filter;

// TODO: Direct client access.
use webex_teams_sdk::WebexApi;

// Websockets.
use websockets;

#[derive(Debug, Clone)]
struct WebexWebsocketClient {
    access_token: String,
    device_url: String,
    device_info: Option<serde_json::Value>,
    on_message: Option<Box<dyn Fn(serde_json::Value, serde_json::Value)>>,
    on_card_action: Option<Box<dyn Fn(serde_json::Value, serde_json::Value)>>,
    websocket: Option<websockets::WebSocket>,
}

impl WebexWebsocketClient {
    fn new(access_token: &str, device_url: &str) -> Self {
        WebexWebsocketClient {
            access_token: access_token.to_string(),
            device_url: device_url.to_string(),
            device_info: None,
            on_message: None,
            on_card_action: None,
            websocket: None,
        }
    }

    fn process_incoming_websocket_message(&self, msg: &serde_json::Value) {
        // Implement processing of incoming WebSocket messages
        // Replace the following code with your message processing logic.
        if let Some(event_type) = msg.get("data").and_then(|data| data.get("eventType")) {
            match event_type {
                serde_json::Value::String(event_type) => {
                    if event_type == "conversation.activity" {
                        // Handle conversation activity
                        // ...

                        if let Some(verb) = msg
                            .get("data")
                            .and_then(|data| data.get("activity"))
                            .and_then(|activity| activity.get("verb"))
                        {
                            // Handle verb (e.g., "post" or "cardAction")
                            if verb == "post" {
                                // Handle "post" activity
                                // ...
                            } else if verb == "cardAction" {
                                // Handle "cardAction" activity
                                // ...
                            }
                        }
                    }
                }
                _ => {
                    // Handle other event types
                }
            }
        }
    }

    fn run(&self) {
        // Implement the WebSocket connection logic here
        // ...
    }

    fn get_base64_message_id(&self, activity: &serde_json::Value) -> Option<String> {
        // Implement logic to get the base64 message ID
        // ...

        // Return the base64 message ID or None
        None
    }

    fn ack_message(&self, message_id: &str) {
        // Implement message acknowledgment
        // ...
    }

    fn get_device_info(&self, check_existing: bool) -> Option<serde_json::Value> {
        // Implement logic to get device information
        // ...

        // Return the device information or None
        None
    }
}

async fn websocket_recv(client: Arc<Mutex<WebexWebsocketClient>>) {
    // Implement the WebSocket receive logic here
    // ...
}

async fn connect_and_listen(client: Arc<Mutex<WebexWebsocketClient>>) {
    // Implement the WebSocket connection and listening logic here
    // ...
}

async fn run_listener(client: Arc<Mutex<WebexWebsocketClient>>) {
    // Implement the listener logic here
    // ...
}
