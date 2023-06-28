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

## Getting Started
To start using Rusty Webex in your project, add the following line to your Cargo.toml file:

```toml
[dependencies]
rusty-webex = "0.1.0"
```

Ensure that you have a compatible version of Rust installed. Rusty Webex requires Rust 1.51 or later.

Please refer to the documentation for detailed usage examples, API reference, and guides on how to integrate Rusty Webex into your applications.

## Examples
Here's a simple example that demonstrates how to use Rusty Webex reply to a webex message:

```rust
let client = WebexClient::new(AUTH_TOKEN);
```

Getting details of a particular message:

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

## Contributing
We welcome contributions from the open-source community. If you find a bug, have a feature request, or would like to contribute code improvements, please open an issue or submit a pull request on the GitHub repository.

Please make sure to follow the contribution guidelines when submitting code changes or feature requests.

Start leveraging the power of the Webex API in your Rust applications
