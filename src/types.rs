// own.
use crate::adaptive_card::AdaptiveCard;
use crate::WebexClient;

// serde.
use serde::{Deserialize, Serialize};

// std.
use std::collections::HashMap;
use std::convert::TryFrom;
use std::future::Future;
use std::pin::Pin;

// ###########################################################################
// Websocket user Register request, response and publish types.
// ###########################################################################

#[derive(Serialize, Deserialize)]
pub struct Register {
    pub user_id: u16,
    pub groups: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct RegisterResponse {
    pub url: String,
}

#[derive(Serialize, Deserialize)]
pub struct Publish {
    pub user_id: u16,
    pub group: String,
    pub message: String,
}

// ###########################################################################
// Tuple definition that contains the name:value mapping.
// ###########################################################################

pub type ArgTuple = Vec<(std::string::String, std::string::String)>;
pub type Callback = fn(
    WebexClient,
    Message,
    ArgTuple,
    ArgTuple,
) -> Pin<Box<dyn Future<Output = ()> + Send + Sync + 'static>>;

// ###################################################################
// Define the Argument trait
// ###################################################################

pub trait Argument: Send + Sync {
    fn name(&self) -> &str;
    fn is_required(&self) -> bool;
}

// ###################################################################
// Define the RequiredArgument struct implementing the Argument trait.
// ###################################################################

pub struct RequiredArgument<T: Send + Sync> {
    pub name: String,
    _phantom: std::marker::PhantomData<T>,
}

impl<T: Send + Sync> RequiredArgument<T> {
    pub fn new(name: &str) -> Self {
        RequiredArgument {
            name: name.to_string(),
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T: Send + Sync> Argument for RequiredArgument<T> {
    fn name(&self) -> &str {
        &self.name
    }

    fn is_required(&self) -> bool {
        true
    }
}

// ###################################################################
// Define the OptionalArgument struct implementing the Argument trait.
// ###################################################################

pub struct OptionalArgument<T> {
    pub name: String,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> OptionalArgument<T> {
    pub fn new(name: &str) -> Self {
        OptionalArgument {
            name: name.to_string(),
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T: Send + Sync> Argument for OptionalArgument<T> {
    fn name(&self) -> &str {
        &self.name
    }

    fn is_required(&self) -> bool {
        false
    }
}

// Common response information.
//-----------------------------------------------------------------------------------------------
#[derive(Deserialize, Debug)]
pub struct Response<T> {
    pub id: String,
    pub name: String,
    #[serde(alias = "targetUrl")]
    pub target_url: String,
    pub resource: String,
    pub event: String,
    pub created: String,
    #[serde(alias = "actorId")]
    pub actor_id: String,
    pub data: T,
}

#[derive(Debug, Deserialize)]
pub struct MessageEventResponse {
    pub id: String,
    #[serde(alias = "roomId")]
    pub room_id: String,
    #[serde(alias = "roomType")]
    pub room_type: String,
    #[serde(alias = "personId")]
    pub person_id: String,
    #[serde(alias = "personEmail")]
    pub person_email: String,
    #[serde(alias = "mentionedPeople")]
    pub mentioned_people: Box<[String]>,
    pub created: String,
}

// Device information.
//-----------------------------------------------------------------------------------------------
#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Device2 {
    pub id: String,
    pub display_name: String,
    pub person_id: String,
    pub org_id: String,
    pub capabilities: Vec<String>,
    pub permissions: Vec<String>,
    pub product: String,
    #[serde(rename = "type")]
    pub type_field: String,
    pub tags: Vec<String>,
    pub serial: String,
    pub software: String,
    pub primary_sip_url: String,
    pub sip_urls: Vec<String>,
    pub error_codes: Vec<String>,
    pub connection_status: String,
    pub created: String,
    pub first_seen: String,
    pub last_seen: String,
    pub managed_by: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Devices {
    pub items: Vec<Device>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Device {
    pub(crate) device_name: String,
    pub(crate) device_type: String,
    pub(crate) localized_model: String,
    pub(crate) model: String,
    pub(crate) name: String,
    pub(crate) system_name: String,
    pub(crate) system_version: String,
    pub(crate) web_socket_url: Option<String>,
}
/*
pub(crate) struct Device2 {
    pub(crate) device_name: &'static str,
    pub(crate) device_type: &'static str,
    pub(crate) localized_model: &'static str,
    pub(crate) model: &'static str,
    pub(crate) name: &'static str,
    pub(crate) system_name: &'static str,
    pub(crate) system_version: &'static str,
}
*/

// Room information.
//-----------------------------------------------------------------------------------------------
#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Room {
    pub id: String,    // A unique identifier for the room.
    pub title: String, // A user-friendly name for the room.
    #[serde(rename = "type")]
    pub room_type: String, // The room type.
    /**
     * direct - 1:1 room
     * group - group room
     */
    pub is_locked: bool, // Whether the room is moderated (locked) or not.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_id: Option<String>, // The ID for the team with which this room is associated.
    pub last_activity: String, // The date and time of the room's last activity.
    pub creator_id: String, // The ID of the person who created this room.
    pub created: String, // The date and time the room was created.
}

// Holds details about the organization an account belongs to.
//-----------------------------------------------------------------------------------------------
#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Organization {
    pub id: String,           // Id of the org.
    pub display_name: String, // Display name of the org.
    pub created: String,      // Date and time the org was created
}

// Holds details about a team that includes the account.
//-----------------------------------------------------------------------------------------------
#[derive(Deserialize, Serialize, Debug)]
pub struct Team {
    pub id: String,      // Id of the team.
    pub name: String,    // Name of the team.
    pub created: String, // Date and time the team was created.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>, // Team description.
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CatalogReply {
    pub service_links: Catalog,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Catalog {
    pub atlas: String,
    #[serde(rename = "broadworksIdpProxy")]
    pub broadworks_idp_proxy: String,
    #[serde(rename = "clientLogs")]
    pub client_logs: String,
    pub ecomm: String,
    pub fms: String,
    pub idbroker: String,
    pub idbroker_guest: String,
    pub identity: String,
    pub identity_guest_cs: String,
    pub license: String,
    #[serde(rename = "meetingRegistry")]
    pub meeting_registry: String,
    pub metrics: String,
    pub oauth_helper: String,
    pub settings_service: String,
    pub u2c: String,
    /// wdm is the url used for fetching devices.
    pub wdm: String,
    pub web_authentication: String,
    pub webex_appapi_service: String,
}

/// Destination for a `MessageOut`
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum Destination {
    RoomId(String),        // Post a message in this room.
    ToPersonId(String),    // Post a message to a person, using their user ID.
    ToPersonEmail(String), // Post a message to a person, using their email.
}

// Messaging format for an outgoing message.
//-----------------------------------------------------------------------------------------------
#[derive(Deserialize, Serialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct MessageOut {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>, // The parent message to reply to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub room_id: Option<String>, // The room ID of the message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to_person_id: Option<String>, // The person ID of the recipient when sending a private 1:1 message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to_person_email: Option<String>, // The email address of the recipient when sending a private 1:1 message.
    // TODO - should we use globalIDs? We should check this field before the message is sent
    // rolls up room_id, to_person_id, and to_person_email all in one field :)
    //#[serde(flatten)]
    //pub deliver_to: Option<Destination>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>, // The message, in plain text. If markdown is specified this parameter may be optionally used to provide alternate text for UI clients that do not support rich text. The maximum message length is 7439 bytes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub markdown: Option<String>, // The message, in Markdown format. The maximum message length is 7439 bytes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub files: Option<Vec<String>>, // The public URL to a binary file to be posted into the room. Only one file is allowed per message. https://developer.webex.com/docs/api/basics#message-attachments
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachments: Option<Vec<Attachment>>, // Content attachments to attach to the message. Only one card per message is supported.
}

impl From<Message> for MessageOut {
    fn from(message: Message) -> MessageOut {
        MessageOut {
            parent_id: message.parent_id,
            room_id: message.room_id,
            to_person_id: message.to_person_id,
            to_person_email: message.to_person_email,
            text: message.text,
            markdown: message.markdown,
            files: message.files,
            attachments: message.attachments,
        }
    }
}

// Room type.
//-----------------------------------------------------------------------------------------------
#[derive(Deserialize, Serialize, Debug, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RoomType {
    #[default]
    Direct, // 1:1 private chat.
    Group, // Group room.
}

// Webex message response type.
//-----------------------------------------------------------------------------------------------
#[derive(Deserialize, Serialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>, // The unique identifier for the message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub room_id: Option<String>, // The room ID of the message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub room_type: Option<RoomType>, // The room type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to_person_id: Option<String>, // The person ID of the recipient when sending a private 1:1 message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to_person_email: Option<String>, // The email address of the recipient when sending a private 1:1 message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>, // The message, in plain text. If markdown is specified this parameter may be optionally used to provide alternate text for UI clients that do not support rich text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub markdown: Option<String>, // The message, in Markdown format.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub html: Option<String>, // The text content of the message, in HTML format. This read-only property is used by the Webex Teams clients.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub files: Option<Vec<String>>, // Public URLs for files attached to the message. For the supported media types and the behavior of file uploads, see Message Attachments.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub person_id: Option<String>, // The person ID of the message author.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub person_email: Option<String>, // The email address of the message author.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mentioned_people: Option<Vec<String>>, // People IDs for anyone mentioned in the message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mentioned_groups: Option<Vec<String>>, // Group names for the groups mentioned in the message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachments: Option<Vec<Attachment>>, // Message content attachments attached to the message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<String>, // The date and time the message was created.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated: Option<String>, // The date and time the message was updated, if it was edited.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>, // The ID of the "parent" message (the start of the reply chain)
}

// Empty reply.
//-----------------------------------------------------------------------------------------------
#[derive(Deserialize, Serialize, Debug)]
pub(crate) struct EmptyReply {}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct Actor {
    pub id: String,
    pub object_type: String,
    pub display_name: Option<String>,
    pub org_id: Option<String>,
    pub email_address: Option<String>,
    #[serde(rename = "entryUUID")]
    pub entry_uuid: String,
    #[serde(rename = "type")]
    pub actor_type: Option<String>,
}

/// Activity types.
//-----------------------------------------------------------------------------------------------
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActivityType {
    Message(MessageActivity), // Message changed - see [`MessageActivity`] for details.
    Space(SpaceActivity), // The space the bot is in has changed - see [`SpaceActivity`] for details.
    AdaptiveCardSubmit,   // The user has submitted an [`AdaptiveCard`].
    /// Meeting event.
    /// TODO: This needs to be broken down like `Message` and `Space`, if anyone cares.
    Locus,
    Janus,       // Call event.
    StartTyping, // Someone started typing.
    Highlight,
    /// activities will contain `event.data.event_type`, otherwise if it's an Unknown
    /// `conversation.activity` type (belonging in Message or Space), the string will be
    /// `"conversation.activity.{event.data.activity.verb}"`, for example it would be
    /// `"conversation.activity.post"` for `Message(MessageActivity::Posted)`
    Unknown(String), // Unknown activity. Contains a representation of the string that failed to parse - unknown.
}
/// Specifics of what type of activity [`ActivityType::Message`] represents.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MessageActivity {
    Posted, // A message was posted.
    Shared, // A message was posted with attachments. TODO: Should this be merged with [`Self::Posted`]? Could have a field to determine
    // xs/no attachments, or we can let the user figure that out from the message instance.
    Acknowledged, // A message was acknowledged.
    Deleted,      // A message was deleted.
}
/// Specifics of what type of activity [`ActivityType::Space`] represents.
/// TODO: should we merge [`Self::Created`]/[`Self::Joined`]?
///
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpaceActivity {
    Created,             // A new space was created with the bot.
    Joined, // Bot was added to a space... or a reaction was added to a message? TODO: figure out a way to tell these events apart.
    Left,   // Bot left (was kicked out of) a space.
    Changed, // Space was changed (i.e. name change, cover image changed, space picture changed). Also includes meeting changes (meeting name or schedule).
    MeetingScheduled, // New meeting scheduled.
    Locked,  // Space became moderated.
    Unlocked, // Space became unmoderated.
    ModeratorAssigned, // A new moderator was assigned.
    ModeratorUnassigned, // A moderator was unassigned.
}

impl TryFrom<&str> for MessageActivity {
    type Error = ();
    fn try_from(s: &str) -> Result<Self, ()> {
        match s {
            "post" => Ok(Self::Posted),
            "share" => Ok(Self::Shared),
            "acknowledge" => Ok(Self::Acknowledged),
            "delete" => Ok(Self::Deleted),
            _ => Err(()),
        }
    }
}
impl TryFrom<&str> for SpaceActivity {
    type Error = ();
    fn try_from(s: &str) -> Result<Self, ()> {
        match s {
            "create" => Ok(Self::Created),
            "add" => Ok(Self::Joined),
            "leave" => Ok(Self::Left),
            "lock" => Ok(Self::Locked),
            "unlock" => Ok(Self::Unlocked),
            "update" | "assign" | "unassign" => Ok(Self::Changed),
            "schedule" => Ok(Self::MeetingScheduled),
            "assignModerator" => Ok(Self::ModeratorAssigned),
            "unassignModerator" => Ok(Self::ModeratorUnassigned),
            _ => Err(()),
        }
    }
}

// Messaging activities.
//-----------------------------------------------------------------------------------------------
impl MessageActivity {
    #[must_use]
    pub const fn is_created(&self) -> bool {
        matches!(*self, Self::Posted | Self::Shared) // True if this is a new message ([`Self::Posted`] or [`Self::Shared`]).
    }
}

// Attachment contents
//-----------------------------------------------------------------------------------------------
#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct Attachment {
    #[serde(rename = "contentType")]
    pub content_type: String, // The content type of the attachment.
    pub content: AdaptiveCard, // Adaptive Card content.
}

// Attachment action details.
//-----------------------------------------------------------------------------------------------
#[derive(Deserialize, Serialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct AttachmentAction {
    pub id: String, // A unique identifier for the action.
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub action_type: Option<String>, // The type of action performed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_id: Option<String>, // The parent message the attachment action was performed on.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inputs: Option<HashMap<String, serde_json::Value>>, // The action's inputs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub person_id: Option<String>, // The ID of the person who performed the action.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub room_id: Option<String>, // The ID of the room the action was performed within.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<String>, // The date and time the action was created.
}

// Person information.
//-----------------------------------------------------------------------------------------------
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Person {
    // A unique identifier for the person.
    pub id: String,                      // The email addresses of the person.
    pub emails: Vec<String>,             // Phone numbers for the person.
    pub phone_numbers: Vec<PhoneNumber>, // The full name of the person.
    pub display_name: String, // The nickname of the person if configured. If no nickname is configured for the person, this field will not be present.
    pub nick_name: String,    // The first name of the person.
    pub first_name: String,   // The last name of the person.
    pub last_name: String,    // The URL to the person's avatar in PNG format.
    pub avatar: String,       // The ID of the organization to which this person belongs.
    pub org_id: String,       // The date and time the person was created.
    pub created: String,      // The date and time of the person's last activity within Webex Teams.
    pub last_activity: String, // Person status.

    /**
     * active - active within the last 10 minutes.
     * call - the user is in a call.
     * DoNotDisturb - the user has manually set their status to "Do Not Disturb".
     * inactive - last activity occurred more than 10 minutes ago.
     * meeting - the user is in a meeting.
     * OutOfOffice - the user or a Hybrid Calendar service has indicated that they are "Out of Office".
     * pending - the user has never logged in; a status cannot be determined.
     * presenting - the user is sharing content.
     * unknown - the user’s status could not be determined.
     */
    pub status: String, // The type of person account, such as person or bot.

    /**
     * person- account belongs to a person
     * bot - account is a bot user
     * appuser - account is a guest user
     */

    #[serde(rename = "type")]
    pub person_type: String,
}

// Phone number information.
//-----------------------------------------------------------------------------------------------
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PhoneNumber {
    /// Phone number type
    #[serde(rename = "type")]
    pub number_type: String,
    /// Phone number
    pub value: String,
}

#[cfg(test)]
mod tests {}
