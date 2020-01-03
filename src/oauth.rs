use log::{debug, error};
use reqwest::header;
use serde::Deserialize;
use serde_json::Number;
use std::error::Error;
use std::fmt;

pub(crate) fn issue_access_token(channel_id: usize, channel_secret: &str) -> OAuthResult<String> {
    debug!("チャンネルアクセストークン発行リクエストを行います。");
    let client = reqwest::Client::new();
    let mut res = client
        .post("https://api.line.me/v2/oauth/accessToken")
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .form(&[
            ("grant_type", "client_credentials"),
            ("client_id", &channel_id.to_string()),
            ("client_secret", channel_secret),
        ])
        .send()
        .unwrap();
    debug!("チャンネルアクセストークン発行リクエストを送信しました。");
    if res.status() == 200 {
        debug!("チャンネルアクセストークン発行リクエストに成功しました。");
        let res_body: ResponseBody = res.json().unwrap();
        Ok(res_body.access_token)
    } else if res.status() == 400 {
        let e_res_body: ErrorResponseBody = res.json().unwrap();
        error!("チャンネルアクセストークン発行リクエストエラーレスポンスを受信しました。ステータス[{}], エラーレスポンス[{:?}]"
               , res.status(), e_res_body);
        Err(OAuthError::ErrorResponse(
            e_res_body.error,
            e_res_body.error_description,
        ))
    } else {
        error!(
            "チャンネルアクセストークン発行リクエストに失敗しました。ステータス[{}]",
            res.status()
        );
        Err(OAuthError::UnexpectedStatusResponse(u16::from(
            res.status(),
        )))
    }
}

type OAuthResult<T> = Result<T, OAuthError>;

#[derive(Debug)]
pub enum OAuthError {
    ErrorResponse(String, Option<String>),
    Reqwest(reqwest::Error),
    UnexpectedStatusResponse(u16),
}

impl fmt::Display for OAuthError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            OAuthError::ErrorResponse(err, err_desc) => {
                if let Some(desc) = err_desc {
                    write!(f, "Error response: {}, description: {}", err, desc)
                } else {
                    write!(f, "Error response: {}", err)
                }
            }
            OAuthError::Reqwest(err) => write!(f, "Request failed: {}", err),
            OAuthError::UnexpectedStatusResponse(status) => {
                write!(f, "Unexpected status response: status = {}", status)
            }
        }
    }
}

impl Error for OAuthError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            OAuthError::Reqwest(err) => Some(err),
            _ => None,
        }
    }
}

impl From<reqwest::Error> for OAuthError {
    fn from(err: reqwest::Error) -> Self {
        OAuthError::Reqwest(err)
    }
}

#[derive(Deserialize, Debug)]
struct ResponseBody {
    access_token: String,
    expires_in: Number,
    token_type: String,
}

#[derive(Deserialize, Debug)]
struct ErrorResponseBody {
    error: String,
    error_description: Option<String>,
}
