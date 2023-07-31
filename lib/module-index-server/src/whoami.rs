use auth_api_client::{client::AuthApiClient, types::AuthApiClientError};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use thiserror::Error;

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
    token: &str,
    token_map: Arc<Mutex<HashMap<String, String>>>,
) -> WhoamiResult<String> {
    let mut token_map = token_map.lock().await;

    match token_map.get(token) {
        Some(email) => Ok(email.into()),
        None => {
            let auth_api_client =
                AuthApiClient::new(auth_api_client::PROD_AUTH_API_ENDPOINT.try_into()?, token);

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
    token: &str,
    token_map: Arc<Mutex<HashMap<String, String>>>,
) -> WhoamiResult<bool> {
    Ok(is_systeminit_email(
        &get_email_for_auth_token(token, token_map).await?,
    ))
}
