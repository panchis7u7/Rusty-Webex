pub const WEBEX_URI: &str = "https://webexapis.com/v1/";

pub mod Service {

    use http::HeaderValue;
    use reqwest::header::ACCEPT;
    use reqwest::header::CONTENT_TYPE;

    mod endpoints {
        // Private crate to hold all types that the user shouldn't have to interact with.
        use crate::types::{AttachmentAction, Message, Organization, Person, Room, Team};
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

        #[derive(Deserialize)]
        pub struct ListResult<T> {
            pub items: Vec<T>,
        }
    }

    use crate::service::WEBEX_URI;
    use crate::types::{self, Message};
    use http::HeaderMap;
    use reqwest::Client;
    use std::{mem::MaybeUninit, sync::Once};

    use self::endpoints::Gettable;

    // Singleton class
    // ----------------------------------------------------------------------------
    pub struct Service {
        client: Client,
        headers: HeaderMap,
    }

    impl Service {
        fn get_instance() -> &'static Service {
            static mut instance: MaybeUninit<Service> = MaybeUninit::uninit();
            static once: Once = Once::new();

            unsafe {
                once.call_once(|| {
                    let mut headers = HeaderMap::new();
                    headers.insert(
                        CONTENT_TYPE,
                        HeaderValue::from_str("application/json").unwrap(),
                    );
                    headers.insert(ACCEPT, HeaderValue::from_str("application/json").unwrap());

                    instance.write(Service {
                        client: Client::new(),
                        headers,
                    });
                });

                instance.assume_init_ref()
            }
        }
    }

    // Review the status for the response.
    // ----------------------------------------------------------------------------
    pub fn review_status(response: &reqwest::Response) -> () {
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

    // Webex client specific functionality.
    // ----------------------------------------------------------------------------
    pub async fn send_message(token: &str, message_out: &types::MessageOut) -> Message {
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

    // Retrieve detailed information from a specific message..
    // ----------------------------------------------------------------------------
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

    //pub async fn send_card_message() -> reqwest::Response {}
}
