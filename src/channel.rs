use crate::event::WebhookEvent;
use crate::reply::Reply;
use log::debug;

pub trait HandleWebhookEvent {
    fn handle_webhook_event(&mut self, event: &WebhookEvent) -> Option<Reply>;
}

pub struct Channel {
    pub(crate) id: usize,
    pub(crate) user_id: String,
    pub(crate) secret: String,
    pub(crate) access_token: Option<String>,
    handler: Box<dyn HandleWebhookEvent + Send + 'static>,
}

impl Channel {
    pub fn new(
        id: usize,
        user_id: String,
        secret: String,
        access_token: Option<String>,
        handler: impl HandleWebhookEvent + Send + 'static,
    ) -> Self {
        let handler = Box::new(handler);
        Channel {
            id,
            user_id,
            secret,
            access_token,
            handler,
        }
    }

    pub(crate) fn handle_event(&mut self, event: WebhookEvent) -> Option<Reply> {
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
