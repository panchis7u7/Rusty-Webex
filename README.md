# Rusty Webex (SDK).
<hr>

Rusty Webex is a Rust crate that provides a simple and intuitive software development kit (SDK) for interacting with the Webex API. With Rusty Webex, you can seamlessly integrate Webex functionality into your Rust applications, making it effortless to build powerful and interactive collaboration solutions.

## Features
Easy Webex API Integration: Rusty Webex abstracts away the complexities of the Webex API, allowing you to focus on building your application logic without getting bogged down in API intricacies.

+ **Authentication Made Simple:** Authenticate your application with the Webex platform using various authentication methods supported by Webex, including OAuth2.0, API Key, or Personal Access Token. Rusty Webex handles the authentication process seamlessly, ensuring a secure and streamlined experience.

+ **Comprehensive API Coverage:** Rusty Webex provides comprehensive coverage of the Webex API, allowing you to interact with various Webex resources, including rooms, messages, people, meetings, and more. You can effortlessly create, read, update, and delete resources as needed.

+ **Webhook Support:** Set up and manage Webex webhooks to receive real-time notifications for events of interest, such as new messages, meeting updates, or membership changes. Rusty Webex simplifies the process of creating and managing webhooks, making it easy to build reactive and event-driven applications.

+ **Flexible Error Handling:** Rusty Webex provides robust error handling mechanisms, enabling you to gracefully handle API errors and failures. It offers detailed error messages and convenient error types to aid in debugging and troubleshooting.

+ **Asynchronous Operations:** Built with asynchronous programming in mind, Rusty Webex utilizes Rust's powerful async/await syntax and integrates seamlessly with popular asynchronous runtimes. This allows you to perform non-blocking API calls and efficiently handle concurrent operations.

+ **Integrated Bot/Websocket Server:** Set up and deploy complex webex bots for all your automation necessities. With the flexible and user freindly callback function syntax, interact directly via webex or with custom embedded devices using the integrated websocket server.

## Getting Started
To start using Rusty Webex in your project, add the following line to your Cargo.toml file:

```toml
[dependencies]
rusty-webex = "0.1.0"
```

Ensure that you have a compatible version of Rust installed. Rusty Webex requires Rust 1.51 or later.

Please refer to the documentation for detailed usage examples, API reference, and guides on how to integrate Rusty Webex into your applications.

## Examples
Here's a simple example that demonstrates how to use the Rusty Webex client:

Create a new webex bot server.
```rust
let server = WebexBotServer::new(
    std::env::var("TOKEN")
        .expect("The TOKEN must be set.")
        .as_str(),
);
```

```rust
let detailed_message_info = client.get_message_details(message_id).await;
```

Reply to a message:
```rust
let mut event_response_message = MessageOut::from(detailed_message_info);
event_response_message.text = Some("Hello to you!".to_string());

let event_response_message = client.send_message(&event_response_message).await;
```

### Adaptive Cards
</hr>

You can generate you own personalized adaptive with proper rust type system:

```rust
pub fn message_card(user: &String, message: &String) -> Attachment {
    Attachment {
        content_type: "application/vnd.microsoft.card.adaptive".to_string(),
        content: AdaptiveCard::new().add_body(
            CardElement::text_block(format!("{}: has sent the following message: {}", user, message))
                .set_wrap(true)
                .set_spacing(Spacing::Medium),
        ),
    }
}

event_response_message.attachments = Some(vec![templates::templates::message_card("Sebastian", "Hello from Rust, Webex!")])
```

### Bot Server
</hr>

Setup casynchronous callbacks for specific webex commands:
```rust
server.add_command("/say_hello", vec![], move |client, message, _required_args, _optional_args| {
        Box::pin(async move {
            let mut event_response_message = MessageOut::from(message);
            event_response_message.text =
                Some("Hello from the webex client!".to_string());
            client
                .send_message(&MessageOut::from(event_response_message))
                .await;
        })
    },
).await;
```

Call websocket connected devices that are registered within your websocket server:
```rust
server.add_command("/embedded", vec![Box::new(RequiredArgument::<String>::new("is_embedded"))],
move |_client, _message, _required_args, _optional_args| {
    Box::pin(async move {
            
            // Setup the websocket client for communication with the embedded device to the webex bot.
            let ws_client = WebSocketClient::new(
                "172.172.194.77", 8080, 2, vec![String::from("fut_assist")],
            );
            
            let registration_url = ws_client.register().await;
            println!("Registration URL from server: {}", &registration_url.url);
            
            // Generate sender and receiver for the websocket crated.
            let (_sender, receiver) = ws_client
                .start_ws_client(registration_url.url)
                .await
                .unwrap();
            
            // Spawn a task to listen for incoming messages
            tokio::spawn(listen_for_messages(ws_client, receiver));
        })
    },
).await;
```

Launch the server.
```rust
    let _ = server.launch().await;
```

## Contributing
We welcome contributions from the open-source community. If you find a bug, have a feature request, or would like to contribute code improvements, please open an issue or submit a pull request on the GitHub repository.

Please make sure to follow the contribution guidelines when submitting code changes or feature requests.

Start leveraging the power of the Webex API in your Rust applications
