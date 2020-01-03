use crate::channel::Channel;
use crate::oauth::OAuthError;
use crate::reply::{ReplyError, respond};
use crate::request::{RequestBody, RequestBodyError};
use log::debug;
use signature::Algorithm;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::error::Error;
use std::fmt::{self, Display, Formatter};
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
            None => Err(MessagingError::Destination(
                "宛先ユーザーIDに該当するチャンネルが存在しません。".to_owned(),
            )),
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
            Err(MessagingError::Signature(
                "webhookリクエストの署名検証の結果、リクエスト元の正当性を確認できませんでした。"
                    .to_owned(),
            ))
        }
    }

    pub fn handle_event(&self, body: RequestBody) -> MessagingResult<()> {
        let user_id = &body.destination;
        debug!("webhookイベントのハンドリングを行います。");
        let mut channel = self.get_channel(user_id)?.lock().unwrap();
        for event in body.events{
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

#[derive(Debug)]
pub enum MessagingError {
    Destination(String),
    Signature(String),
    OAuth(OAuthError),
    Reply(ReplyError),
    RequestBody(RequestBodyError),
}

impl Display for MessagingError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            MessagingError::Destination(mes) => write!(f, "Destination error: {}", mes),
            MessagingError::Signature(mes) => write!(f, "Signature error: {}", mes),
            MessagingError::OAuth(err) => write!(f, "OAuth error: {}", err),
            MessagingError::Reply(err) => write!(f, "Reply error: {}", err),
            MessagingError::RequestBody(err) => write!(f, "RequestBody error: {}", err),
        }
    }
}

impl Error for MessagingError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            MessagingError::OAuth(err) => Some(err),
            MessagingError::Reply(err) => Some(err),
            MessagingError::RequestBody(err) => Some(err),
            _ => None,
        }
    }
}

impl From<OAuthError> for MessagingError {
    fn from(err: OAuthError) -> Self {
        MessagingError::OAuth(err)
    }
}

impl From<ReplyError> for MessagingError {
    fn from(err: ReplyError) -> Self {
        MessagingError::Reply(err)
    }
}

impl From<RequestBodyError> for MessagingError {
    fn from(err: RequestBodyError) -> Self {
        MessagingError::RequestBody(err)
    }
}
