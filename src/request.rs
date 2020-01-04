use crate::event::WebhookEvent;
use failure::Fail;
use serde::Deserialize;
use std::convert::TryFrom;

#[derive(Deserialize, Debug)]
pub struct RequestBody {
    pub(crate) destination: String,
    pub(crate) events: Vec<WebhookEvent>,
    #[serde(default)]
    pub(crate) src: String,
}

impl TryFrom<String> for RequestBody {
    type Error = RequestBodyError;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        let mut body = serde_json::from_str::<RequestBody>(&s)?;
        body.src = s;
        Ok(body)
    }
}

#[derive(Debug, Fail)]
pub enum RequestBodyError {
    #[fail(display = "Parse error: {}", error)]
    Parse { error: serde_json::Error },
}

impl From<serde_json::Error> for RequestBodyError {
    fn from(error: serde_json::Error) -> Self {
        RequestBodyError::Parse { error }
    }
}
