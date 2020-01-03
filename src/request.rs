use crate::event::WebhookEvent;
use serde::Deserialize;
use std::error::Error;
use std::convert::TryFrom;
use std::fmt::{self, Formatter, Display};

#[derive(Deserialize, Debug)]
pub struct RequestBody {
    pub(crate) destination: String,
    pub(crate) events: Vec<WebhookEvent>,
    #[serde(default)]
    pub(crate) src: String,
}

impl TryFrom<String> for RequestBody {
    type Error = RequestBodyError;
    fn try_from(s: String) -> Result<Self, Self::Error>{
        let mut body = serde_json::from_str::<RequestBody>(&s)?;
        body.src = s;
        Ok(body)
    }
}

#[derive(Debug)]
pub enum RequestBodyError {
    Parse(serde_json::Error),
}

impl Display for RequestBodyError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result{
        match self {
            RequestBodyError::Parse(mes) => write!(f, "Parse error: {}", mes),
        }
    }
}

impl Error for RequestBodyError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            RequestBodyError::Parse(err) => Some(err),
        }
    } 
}

impl From<serde_json::Error> for RequestBodyError {
    fn from(err: serde_json::Error) -> Self {
        RequestBodyError::Parse(err)
    }
}