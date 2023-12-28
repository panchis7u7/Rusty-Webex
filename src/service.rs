// http.
use http::HeaderValue;

// reqwest.
use reqwest::header::ACCEPT;
use reqwest::header::CONTENT_TYPE;

// ###########################################################################
// Constants.
// ###########################################################################

pub const WEBEX_URI: &'static str = "https://webexapis.com/v1/";
pub(crate) const DEFAULT_DEVICE_URL: &'static str = "https://wdm-a.wbx2.com/wdm/api/v1";

// ###########################################################################
// Endpoint containers.
// ###########################################################################

mod endpoints {
    // Private crate to hold all types that the user shouldn't have to interact with.
    use crate::types::{
        AttachmentAction, DevicesDetails, Message, Organization, Person, Room, Team,
    };
    use serde::Deserialize;
    // Trait for API types. Has to be public due to trait bounds limitations on webex API, but hidden
    // in a private crate so users don't see it.
    pub trait Gettable {
        const API_ENDPOINT: &'static str; // Endpoint to query to perform an HTTP GET request with or without an Id.
    }

    impl Gettable for Message {
        const API_ENDPOINT: &'static str = "messages";
    }

    impl Gettable for Organization {
        const API_ENDPOINT: &'static str = "organizations";
    }

    impl Gettable for AttachmentAction {
        const API_ENDPOINT: &'static str = "attachment/actions";
    }

    impl Gettable for Room {
        const API_ENDPOINT: &'static str = "rooms";
    }

    impl Gettable for Person {
        const API_ENDPOINT: &'static str = "people";
    }

    impl Gettable for Team {
        const API_ENDPOINT: &'static str = "teams";
    }

    impl Gettable for DevicesDetails {
        const API_ENDPOINT: &'static str = "devices";
    }

    #[derive(Deserialize)]
    pub struct ListResult<T> {
        pub items: Vec<T>,
    }
}

use crate::types::DeviceDetails;
use crate::types::{Device, DevicesDetails, Message, MessageOut};
use http::HeaderMap;
use reqwest::Client;
use std::{mem::MaybeUninit, sync::Once};

use self::endpoints::Gettable;

// ###########################################################################
// Singleton class
// ###########################################################################

pub struct Service {
    client: Client,
    headers: HeaderMap,
}

impl Service {
    pub fn get_instance() -> &'static Service {
        static mut INSTANCE: MaybeUninit<Service> = MaybeUninit::uninit();
        static ONCE: Once = Once::new();

        unsafe {
            ONCE.call_once(|| {
                let mut headers = HeaderMap::new();
                headers.insert(
                    CONTENT_TYPE,
                    HeaderValue::from_str("application/json").unwrap(),
                );
                headers.insert(ACCEPT, HeaderValue::from_str("application/json").unwrap());

                INSTANCE.write(Service {
                    client: Client::new(),
                    headers,
                });
            });

            INSTANCE.assume_init_ref()
        }
    }
}

// ###########################################################################
// Review the status for the response.
// ###########################################################################

pub fn review_status(response: &reqwest::Response) -> reqwest::StatusCode {
    match response.status() {
        reqwest::StatusCode::OK => {
            log::debug!("Succesful request: {:?}", response);
            reqwest::StatusCode::OK
        }
        reqwest::StatusCode::NOT_FOUND => {
            log::debug!("Got 404! Haven't found resource!: {:?}", response);
            reqwest::StatusCode::NOT_FOUND
        }
        _ => {
            log::error!("Got 404! Haven't found resource!: {:?}", response);
            reqwest::StatusCode::NOT_FOUND
        }
    }
}

// ###########################################################################
// Webex client specific functionality.
// ###########################################################################

pub async fn send_message(token: &str, message_out: &MessageOut) -> Message {
    let client_service = Service::get_instance();
    let response = client_service
        .client
        .post(format!("{}{}", WEBEX_URI, Message::API_ENDPOINT))
        .headers(client_service.headers.clone())
        .json(&message_out)
        .bearer_auth(token)
        .send()
        .await
        .unwrap();

    review_status(&response);

    let message = response
        .json::<Message>()
        .await
        .expect("failed to convert struct from json");

    return message;
}

// ###########################################################################
// Retrieve detailed information from a specific message.
// ###########################################################################

pub async fn get_message_details(token: &str, message_id: &String) -> Message {
    let client_service = Service::get_instance();
    let response = client_service
        .client
        .get(format!(
            "{}{}/{}",
            WEBEX_URI,
            Message::API_ENDPOINT,
            message_id
        ))
        .headers(client_service.headers.clone())
        .bearer_auth(token)
        .send()
        .await
        .unwrap();

    review_status(&response);

    let message = response
        .json::<Message>()
        .await
        .expect("failed to convert struct from json");

    return message;
}

// ###########################################################################
// Retrieve devices which represent Webex RoomOS devices and Webex Calling phones.
// ###########################################################################

pub(crate) async fn get_devices(token: &str) -> Option<DevicesDetails> {
    let client_service = Service::get_instance();
    let response = client_service
        .client
        .get(format!(
            "{}/{}",
            DEFAULT_DEVICE_URL,
            DevicesDetails::API_ENDPOINT
        ))
        .headers(client_service.headers.clone())
        .bearer_auth(token)
        .send()
        .await
        .unwrap();

    if review_status(&response) == reqwest::StatusCode::NOT_FOUND {
        return None;
    }

    let devices: DevicesDetails = response
        .json::<DevicesDetails>()
        .await
        .expect("[service - get_devices]: Failed to convert struct from json");

    return Some(devices);
}

// ###########################################################################
// Create a new device.
// ###########################################################################

pub(crate) async fn create_device(token: &str, device: Device) -> Option<DeviceDetails> {
    let client_service = Service::get_instance();
    let response = client_service
        .client
        .post(format!(
            "{}/{}",
            DEFAULT_DEVICE_URL,
            DevicesDetails::API_ENDPOINT
        ))
        .headers(client_service.headers.clone())
        .json(&device)
        .bearer_auth(token)
        .send()
        .await
        .unwrap();

    review_status(&response);

    let created_device = response
        .json::<DeviceDetails>()
        .await
        .expect("[service - create_device]: Failed to convert struct from json");

    return Some(created_device);
}

pub(crate) mod websocket {
    use crate::types::{Publish, Register, RegisterResponse, RemoteTransportWebSocketServer};

    use super::{review_status, Service};

    // ----------------------------------------------------------------------------
    // Retrieve a new websocket url from the server.
    // ----------------------------------------------------------------------------
    pub async fn register(
        endpoint: &str,
        websocket_server: &RemoteTransportWebSocketServer,
    ) -> RegisterResponse {
        let client_service = Service::get_instance();
        let response = client_service
            .client
            .post(format!(
                "http://{}:{}/{}",
                websocket_server.host, websocket_server.port, endpoint
            ))
            .headers(client_service.headers.clone())
            .json(&Register {
                user_id: websocket_server.user_id,
                groups: websocket_server.subscription_groups.clone(),
            })
            .send()
            .await
            .unwrap();

        review_status(&response);

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
        endpoint: &str,
        group: String,
        message: serde_json::Value,
        websocket_server: &RemoteTransportWebSocketServer,
    ) {
        let client_service = Service::get_instance();
        let response = client_service
            .client
            .post(format!(
                "http://{}:{}/{}",
                websocket_server.host, websocket_server.port, endpoint
            ))
            .headers(client_service.headers.clone())
            .json(&Publish {
                user_id: websocket_server.user_id,
                group,
                message: message.to_string(),
            })
            .send()
            .await
            .unwrap();

        review_status(&response);
    }
}
