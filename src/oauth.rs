use failure::Fail;
use log::{debug, error};
use reqwest::header;
use serde::Deserialize;
use serde_json::Number;

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
        Err(OAuthError::ErrorResponse {
            message: e_res_body.error,
            description: e_res_body.error_description,
        })
    } else {
        error!(
            "チャンネルアクセストークン発行リクエストに失敗しました。ステータス[{}]",
            res.status()
        );
        Err(OAuthError::UnexpectedStatusResponse {
            status: u16::from(res.status()),
        })
    }
}

type OAuthResult<T> = Result<T, OAuthError>;

#[derive(Debug, Fail)]
pub enum OAuthError {
    #[fail(
        display = "Error response: {}, description: {:?}",
        message, description
    )]
    ErrorResponse {
        message: String,
        description: Option<String>,
    },
    #[fail(display = "Request error: {}", error)]
    Reqwest { error: reqwest::Error },
    #[fail(display = "Unexpected status response: status = {}", status)]
    UnexpectedStatusResponse { status: u16 },
}

impl From<reqwest::Error> for OAuthError {
    fn from(error: reqwest::Error) -> Self {
        OAuthError::Reqwest { error }
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
