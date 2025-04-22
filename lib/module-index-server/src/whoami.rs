use std::{
    collections::HashMap,
    sync::Arc,
};

use auth_api_client::{
    client::AuthApiClient,
    types::AuthApiClientError,
};
use thiserror::Error;
use tokio::sync::Mutex;

#[remain::sorted]
#[derive(Error, Debug)]
pub enum WhoamiError {
    #[error("auth api error: {0}")]
    AuthApiClient(#[from] AuthApiClientError),
    #[error("Url parse error: {0}")]
    UrlParse(#[from] url::ParseError),
}

type WhoamiResult<T> = Result<T, WhoamiError>;

pub async fn get_email_for_auth_token(
    auth_api_url: &str,
    token: &str,
    token_map: Arc<Mutex<HashMap<String, String>>>,
) -> WhoamiResult<String> {
    let mut token_map = token_map.lock().await;

    match token_map.get(token) {
        Some(email) => Ok(email.into()),
        None => {
            let auth_api_client = AuthApiClient::from_bearer_token(auth_api_url, token)?;

            let whoami = auth_api_client.whoami().await?;

            token_map.insert(token.into(), whoami.email.clone());

            Ok(whoami.email)
        }
    }
}

pub fn is_systeminit_email(email: &str) -> bool {
    email.to_lowercase().ends_with("@systeminit.com")
}

pub async fn is_systeminit_auth_token(
    auth_api_url: &str,
    token: &str,
    token_map: Arc<Mutex<HashMap<String, String>>>,
) -> WhoamiResult<bool> {
    Ok(is_systeminit_email(
        &get_email_for_auth_token(auth_api_url, token, token_map).await?,
    ))
}
