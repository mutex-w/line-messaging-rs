use crate::event::WebhookEvent;
use crate::oauth::OAuthError;
use crate::reply::{Reply, ReplyError};
use log::debug;
use signature::Algorithm;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::sync::Mutex;

pub trait HandleWebhookEvent {
    fn handle_webhook_event(&mut self, event: &WebhookEvent) -> Option<Reply>;
}

pub struct Channel {
    pub id: usize,
    pub secret: String,
    access_token: Option<String>,
    handler: Box<dyn HandleWebhookEvent + Send + 'static>,
}

impl Channel {
    pub fn new(
        id: usize,
        secret: String,
        access_token: Option<String>,
        handler: impl HandleWebhookEvent + Send + 'static,
    ) -> Self {
        let handler = Box::new(handler);
        Channel {
            id,
            secret,
            access_token,
            handler,
        }
    }

    fn handle_event(&mut self, event: WebhookEvent) -> Option<Reply> {
        match self.handler.handle_webhook_event(&event) {
            Some(mut reply) => {
                if let Some(token) = event.get_reply_token() {
                    reply.reply_token = token;
                }
                debug!("webhookハンドラからリプライオブジェクトを受信しました。");
                Some(reply)
            }
            None => None,
        }
    }
}

pub struct MessagingApi {
    channels: HashMap<usize, Mutex<Channel>>,
}

impl MessagingApi {
    pub fn new() -> Self {
        MessagingApi {
            channels: HashMap::new(),
        }
    }

    pub fn add_channel(mut self, channel: Channel) -> Self {
        self.channels.insert(channel.id, Mutex::new(channel));
        self
    }

    fn get_channel(&self, channel_id: usize) -> MessagingResult<&Mutex<Channel>> {
        match self.channels.get(&channel_id) {
            Some(channel) => Ok(channel),
            None => Err(MessagingError::Destination(
                "宛先チャンネルIDに該当するチャンネルが存在しません。".to_owned(),
            )),
        }
    }

    pub fn sign(&self, channel_id: usize, message: &str, digest: &[u8]) -> MessagingResult<()> {
        debug!("webhookリクエストの署名検証を行います。");
        let channel = self.get_channel(channel_id)?.lock().unwrap();
        // HMAC-SHA256-BASE64アルゴリズムに基づいて署名検査を行う。
        let algorithm = Algorithm::HmacSha256Base64(&channel.secret);
        if algorithm.verify(message, digest) {
            debug!("webhookリクエストの署名検証に成功しました。");
            Ok(())
        } else {
            Err(MessagingError::Signature(
                "webhookリクエストの署名検証の結果、リクエスト元の正当性を確認できませんでした。"
                    .to_owned(),
            ))
        }
    }

    pub fn handle_event(&self, channel_id: usize, event: WebhookEvent) -> MessagingResult<()> {
        debug!("webhookイベントのハンドリングを行います。");
        let mut channel = self.get_channel(channel_id)?.lock().unwrap();
        if let Some(reply) = channel.handle_event(event) {
            let token = Self::get_access_token(&mut channel)?;
            crate::reply::reply(token, &reply)?;
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
}

impl fmt::Display for MessagingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MessagingError::Destination(mes) => write!(f, "Destination error: {}", mes),
            MessagingError::Signature(mes) => write!(f, "Signature error: {}", mes),
            MessagingError::OAuth(err) => write!(f, "OAuth error: {}", err),
            MessagingError::Reply(err) => write!(f, "Reply error: {}", err),
        }
    }
}

impl Error for MessagingError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            MessagingError::OAuth(err) => Some(err),
            MessagingError::Reply(err) => Some(err),
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