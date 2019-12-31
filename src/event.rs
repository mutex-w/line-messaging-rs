use serde::Deserialize;
use serde_json::Number;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type")]
pub enum WebhookEvent {
    Message {
        #[serde(flatten)]
        property: EventCommonProperty,
        message: WebhookMessage,
    },
    Follow {
        #[serde(flatten)]
        property: EventCommonProperty,
    },
    Unfollow {
        #[serde(flatten)]
        property: EventCommonProperty,
    },
    Join {
        #[serde(flatten)]
        property: EventCommonProperty,
    },
    Leave {
        #[serde(flatten)]
        property: EventCommonProperty,
    },
    MemberJoined {
        #[serde(flatten)]
        property: EventCommonProperty,
        joined: Joined,
    },
    MemberLeft {
        #[serde(flatten)]
        property: EventCommonProperty,
        left: Left,
    },
    Postback {
        #[serde(flatten)]
        property: EventCommonProperty,
        postback: Postback,
    },
    Beacon {
        #[serde(flatten)]
        property: EventCommonProperty,
        beacon: Beacon,
    },
    AccountLink {
        #[serde(flatten)]
        property: EventCommonProperty,
        link: Link,
    },
    Things {
        #[serde(flatten)]
        property: EventCommonProperty,
        things: Things,
    },
}

impl WebhookEvent {
    pub(crate) fn get_reply_token(self) -> Option<String> {
        match self {
            WebhookEvent::Message {
                property,
                message: _,
            } => property.reply_token,
            WebhookEvent::Follow { property } => property.reply_token,
            WebhookEvent::Unfollow { property } => property.reply_token,
            WebhookEvent::Join { property } => property.reply_token,
            WebhookEvent::Leave { property } => property.reply_token,
            WebhookEvent::MemberJoined {
                property,
                joined: _,
            } => property.reply_token,
            WebhookEvent::MemberLeft { property, left: _ } => property.reply_token,
            WebhookEvent::Postback {
                property,
                postback: _,
            } => property.reply_token,
            WebhookEvent::Beacon {
                property,
                beacon: _,
            } => property.reply_token,
            WebhookEvent::AccountLink { property, link: _ } => property.reply_token,
            WebhookEvent::Things {
                property,
                things: _,
            } => property.reply_token,
        }
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EventCommonProperty {
    pub(crate) reply_token: Option<String>,
    pub timestamp: Number,
    pub source: Source,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type")]
pub enum Source {
    #[serde(rename_all = "camelCase")]
    User { user_id: String },
    #[serde(rename_all = "camelCase")]
    Group {
        group_id: String,
        user_id: Option<String>,
    },
    #[serde(rename_all = "camelCase")]
    Room {
        room_id: String,
        user_id: Option<String>,
    },
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type")]
pub enum WebhookMessage {
    Text {
        id: String,
        text: String,
    },
    #[serde(rename_all = "camelCase")]
    Image {
        id: String,
        content_provider: ContentProvider,
    },
    #[serde(rename_all = "camelCase")]
    Video {
        id: String,
        duration: Number,
        content_provider: ContentProvider,
    },
    #[serde(rename_all = "camelCase")]
    Audio {
        id: String,
        duration: Number,
        content_provider: ContentProvider,
    },
    #[serde(rename_all = "camelCase")]
    File {
        id: String,
        file_name: String,
        file_size: Number,
    },
    Location {
        id: String,
        title: String,
        address: String,
        latitude: Number,
        longitude: Number,
    },
    #[serde(rename_all = "camelCase")]
    Sticker {
        id: String,
        package_id: String,
        sticker_id: String,
    },
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type")]
pub enum ContentProvider {
    Line,
    #[serde(rename_all = "camelCase")]
    External {
        original_content_url: String,
        preview_image_url: Option<String>,
    },
}

#[derive(Deserialize, Debug)]
pub struct Joined {
    pub members: Vec<Source>,
}

#[derive(Deserialize, Debug)]
pub struct Left {
    pub members: Vec<Source>,
}

#[derive(Deserialize, Debug)]
pub struct Postback {
    pub data: String,
    pub params: Params,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum Params {
    Date(String),
    Time(String),
    Datetime(String),
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type")]
pub enum Beacon {
    Enter {
        #[serde(flatten)]
        property: BeaconCommonProperty,
    },
    Leave {
        #[serde(flatten)]
        property: BeaconCommonProperty,
    },
    Banner {
        #[serde(flatten)]
        property: BeaconCommonProperty,
    },
}

#[derive(Deserialize, Debug)]
pub struct BeaconCommonProperty {
    pub hwid: String,
    pub dm: Option<String>,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "result")]
pub struct Link {
    pub result: LinkResult,
    pub nonce: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum LinkResult {
    Ok,
    Failed,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type")]
pub enum Things {
    #[serde(rename_all = "camelCase")]
    Link { device_id: String },
    #[serde(rename_all = "camelCase")]
    Unlink { device_id: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_message_event() {
        let json_str = r#"
              {
                "id": "325708",
                "type": "text",
                "text": "Hello, world"
              }
        "#;
        let message: WebhookMessage = serde_json::from_str(json_str).unwrap();
        if let WebhookMessage::Text { id, text } = &message {
            assert_eq!(id, "325708");
            assert_eq!(text, "Hello, world");
        } else {
            panic!("Not a text message!")
        }
    }
}
