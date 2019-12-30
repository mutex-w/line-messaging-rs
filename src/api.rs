use crate::reply::{Reply, ReplyError};
use crate::event::WebhookEvent;
use crate::o_auth;
use std::collections::HashMap;
use std::sync::Mutex;
use signature::Algorithm;
use crate::o_auth::OAuthError;
use std::fmt;
use std::error::Error;
use log::debug;
use crate::reply;

pub trait HandleWebhookEvent {
    fn handle_webhook_event(&mut self, event: &WebhookEvent) -> Option<Reply>;
}

pub struct Channel {
    pub id: usize,
    pub secret: String,
    pub user_id: String,
    access_token: Option<String>,
    handler: Box<dyn HandleWebhookEvent + Send + 'static>,
}

impl Channel {
    pub fn new(
        id: usize,
        secret: String,
        user_id: String,
        access_token: Option<String>,
        handler: impl HandleWebhookEvent + Send + 'static,
    ) -> Self {
        let handler = Box::new(handler);
        Channel {
            id,
            secret,
            user_id,
            access_token,
            handler,
        }
    }

    fn handle_event(&mut self, event: WebhookEvent) -> Option<Reply> {
        match self.handler.handle_webhook_event(&event) {
            None => None,
            Some(mut reply) => {
                if let Some(token) = event.get_reply_token() {
                    reply.reply_token = token;
                }
                debug!(
                    "webhookハンドラからリプライオブジェクトを受信しました。"
                );
                Some(reply)
            }
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

    fn get_channel(&self, channel_id: usize) -> LinebotResult<&Mutex<Channel>> {
        match self.channels.get(&channel_id) {
            Some(channel) => Ok(channel),
            None => Err(LinebotError::Destination(
                "宛先チャンネルIDに該当するチャンネルが存在しません。"
                    .to_owned(),
            )),
        }
    }

    pub(crate) fn sign(&self, channel_id: usize, message: &str, digest: &[u8]) -> LinebotResult<()> {
        debug!("webhookリクエストの署名検証を行います。");
        let channel = self.get_channel(channel_id)?.lock().unwrap();
        // HMAC-SHA256-BASE64アルゴリズムに基づいて署名検査を行う。
        let algorithm = Algorithm::HmacSha256Base64(&channel.secret);
        if algorithm.verify(message, digest) {
            debug!("webhookリクエストの署名検証に成功しました。");
            Ok(())
        } else {
            Err(LinebotError::Signature(
                "webhookリクエストの署名検証の結果、リクエスト元の正当性を確認できませんでした。"
                    .to_owned(),
            ))
        }
    }

    pub(crate) fn handle_event(&self, channel_id: usize, event: WebhookEvent) -> LinebotResult<()> {
        debug!("webhookイベントのハンドリングを行います。");
        let mut channel = self.get_channel(channel_id)?.lock().unwrap();
        if let Some(reply) = channel.handle_event(event) {
            let token = Self::get_access_token(&mut channel)?;
            reply::reply(&token, &reply)?;
        }
        Ok(())
    }

    fn get_access_token(channel: &mut Channel) -> LinebotResult<String> {
        match &channel.access_token {
            Some(token) => {
                debug!(
                    "既存のアクセストークンを使用します。トークン[{}]",
                    token
                );
                Ok(token.clone())
            }
            None => {
                let token = o_auth::issue_access_token(channel.id, &channel.secret)?;
                channel.access_token = Some(token.clone());
                Ok(token)
            }
        }
    }
}

type LinebotResult<T> = Result<T, LinebotError>;

#[derive(Debug)]
pub(crate) enum LinebotError {
    Destination(String),
    Signature(String),
    OAuth(OAuthError),
    Reply(ReplyError),
}

impl fmt::Display for LinebotError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LinebotError::Destination(mes) => write!(f, "Destination error: {}", mes),
            LinebotError::Signature(mes) => write!(f, "Signature error: {}", mes),
            LinebotError::OAuth(err) => write!(f, "OAuth error: {}", err),
            LinebotError::Reply(err) => write!(f, "Reply error: {}", err),
        }
    }
}

impl Error for LinebotError {
    fn description(&self) -> &str {
        match self {
            LinebotError::Destination(mes) => mes,
            LinebotError::Signature(mes) => mes,
            LinebotError::OAuth(err) => err.description(),
            LinebotError::Reply(err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&dyn Error> {
        match self {
            LinebotError::OAuth(err) => Some(err),
            LinebotError::Reply(err) => Some(err),
            _ => None,
        }
    }
}

impl From<OAuthError> for LinebotError {
    fn from(err: OAuthError) -> Self {
        LinebotError::OAuth(err)
    }
}

impl From<ReplyError> for LinebotError {
    fn from(err: ReplyError) -> Self {
        LinebotError::Reply(err)
    }
}