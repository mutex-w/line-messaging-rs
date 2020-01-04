use crate::channel::Channel;
use crate::oauth::OAuthError;
use crate::reply::{respond, ReplyError};
use crate::request::{RequestBody, RequestBodyError};
use failure::Fail;
use log::debug;
use signature::Algorithm;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::sync::Mutex;

pub struct MessagingApi {
    channels: HashMap<String, Mutex<Channel>>,
}

impl MessagingApi {
    pub fn new() -> Self {
        MessagingApi {
            channels: HashMap::new(),
        }
    }

    pub fn add_channel(mut self, channel: Channel) -> Self {
        self.channels
            .insert(channel.user_id.clone(), Mutex::new(channel));
        self
    }

    fn get_channel(&self, user_id: &str) -> MessagingResult<&Mutex<Channel>> {
        match self.channels.get(user_id) {
            Some(channel) => Ok(channel),
            None => Err(MessagingError::Destination{
                message:
                "宛先ユーザーIDに該当するチャンネルが存在しません。".to_owned(),
            }),
        }
    }

    pub fn sign(&self, message: String, digest: &[u8]) -> MessagingResult<RequestBody> {
        let body = RequestBody::try_from(message)?;
        let user_id = &body.destination;
        debug!("webhookリクエストの署名検証を行います。");
        let channel = self.get_channel(user_id)?.lock().unwrap();
        // HMAC-SHA256-BASE64アルゴリズムに基づいて署名検査を行う。
        let algorithm = Algorithm::HmacSha256Base64(&channel.secret);
        if algorithm.verify(&body.src, digest) {
            debug!("webhookリクエストの署名検証に成功しました。");
            Ok(body)
        } else {
            Err(MessagingError::Signature{message: 
                "webhookリクエストの署名検証の結果、リクエスト元の正当性を確認できませんでした。"
                    .to_owned(),
            })
        }
    }

    pub fn handle_event(&self, body: RequestBody) -> MessagingResult<()> {
        let user_id = &body.destination;
        debug!("webhookイベントのハンドリングを行います。");
        let mut channel = self.get_channel(user_id)?.lock().unwrap();
        for event in body.events {
            if let Some(reply) = channel.handle_event(event) {
                let token = Self::get_access_token(&mut channel)?;
                respond(token, &reply)?;
            }
        }
        Ok(())
    }

    fn get_access_token(channel: &mut Channel) -> MessagingResult<&str> {
        match &channel.access_token {
            Some(token) => debug!("既存のアクセストークンを使用します。トークン[{}]", token),
            None => {
                // アクセストークンが無いため新規に発番する。
                let token = crate::oauth::issue_access_token(channel.id, &channel.secret)?;
                channel.access_token = Some(token);
            }
        }
        let token = channel.access_token.as_ref().unwrap();
        Ok(token)
    }
}

pub type MessagingResult<T> = Result<T, MessagingError>;

#[derive(Debug, Fail)]
pub enum MessagingError {
    #[fail(display = "Destination error: {}", message)]
    Destination { message: String },
    #[fail(display = "Signature error: {}", message)]
    Signature { message: String },
    #[fail(display = "OAuth error: {}", error)]
    OAuth { error: OAuthError },
    #[fail(display = "Reply error: {}", error)]
    Reply { error: ReplyError },
    #[fail(display = "RequestBody error: {}", error)]
    RequestBody { error: RequestBodyError },
}

impl From<OAuthError> for MessagingError {
    fn from(error: OAuthError) -> Self {
        MessagingError::OAuth{error}
    }
}

impl From<ReplyError> for MessagingError {
    fn from(error: ReplyError) -> Self {
        MessagingError::Reply{error}
    }
}

impl From<RequestBodyError> for MessagingError {
    fn from(error: RequestBodyError) -> Self {
        MessagingError::RequestBody{error}
    }
}
