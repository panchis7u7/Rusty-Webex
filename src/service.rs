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

    // Webex client specific functionality.
    // ----------------------------------------------------------------------------
    pub async fn send_message(token: &str, message_out: &types::MessageOut) -> reqwest::Response {
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

        println!("\nService level Response: {:?}", response);

        return response;
    }
}
