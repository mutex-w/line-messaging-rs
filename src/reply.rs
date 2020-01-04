use failure::Fail;
use log::{debug, error};
use reqwest::header;
use reqwest::Client;
use serde::Serialize;

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Reply {
    pub(crate) reply_token: String,
    pub messages: Vec<ReplyMessage>,
    notification_disabled: bool,
}

impl Reply {
    pub fn new(messages: Vec<ReplyMessage>, notification_disabled: bool) -> Self {
        Reply {
            reply_token: "".to_owned(),
            messages,
            notification_disabled,
        }
    }
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type")]
pub enum ReplyMessage {
    Text { text: String },
}

pub(crate) fn respond(access_token: &str, reply: &Reply) -> ReplyResult<()> {
    debug!(
        "リプライのリクエストを行います。アクセストークン[{}], リプライ[{:?}]",
        access_token, reply
    );
    let client = Client::new();
    let res = client
        .post("https://api.line.me/v2/bot/message/reply")
        // .headers(headers)
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, format!("Bearer {}", access_token))
        .json(reply)
        .send()
        .expect("リプライのリクエストでエラー発生");
    if res.status() == 200 {
        debug!("リプライのリクエストに成功しました。")
    } else {
        error!(
            "リプライのリクエストに失敗しました。ステータス[{}]",
            res.status()
        )
    }
    Ok(())
}

type ReplyResult<T> = Result<T, ReplyError>;

#[derive(Debug, Fail)]
pub enum ReplyError {
    #[fail(display = "Request error: {}", error)]
    Reqwest { error: reqwest::Error },
}

impl From<reqwest::Error> for ReplyError {
    fn from(error: reqwest::Error) -> Self {
        ReplyError::Reqwest { error }
    }
}
