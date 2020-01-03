use crate::channel::Channel;
use crate::event::WebhookEvent;
use crate::oauth::OAuthError;
use crate::reply::ReplyError;
use http::Request;
use log::debug;
use signature::Algorithm;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
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

    pub fn sign(&self, user_id: &str, message: &str, digest: &[u8]) -> MessagingResult<()> {
        debug!("webhookリクエストの署名検証を行います。");
        let channel = self.get_channel(user_id)?.lock().unwrap();
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

    pub fn sign_with_request(&self, user_id: &str, req: Request<String>) -> MessagingResult<()> {
        const SIGNATURE_HEADER_KEY: &str = "X-Line-Signature";
        let message = req.body();
        let digest = req
            .headers()
            .get(SIGNATURE_HEADER_KEY)
            .ok_or(MessagingError::Signature(format!(
                "{}ヘッダーが存在しません。",
                SIGNATURE_HEADER_KEY
            )))?
            .as_bytes();
        self.sign(user_id, message, digest)
    }

    pub fn handle_event(&self, user_id: &str, event: WebhookEvent) -> MessagingResult<()> {
        debug!("webhookイベントのハンドリングを行います。");
        let mut channel = self.get_channel(user_id)?.lock().unwrap();
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
