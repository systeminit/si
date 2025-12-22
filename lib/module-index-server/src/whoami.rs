use std::{
    collections::HashMap,
    sync::Arc,
};

use auth_api_client::{
    client::AuthApiClient,
    types::AuthApiClientError,
};
use telemetry::prelude::*;
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
            let auth_api_client = match AuthApiClient::from_bearer_token(auth_api_url, token) {
                Ok(client) => client,
                Err(err) => {
                    error!("{:?}", err);
                    return Err(err.into());
                }
            };

            let whoami = match auth_api_client.whoami().await {
                Ok(whoami) => whoami,
                Err(err) => {
                    error!("{:?}", err);
                    return Err(err.into());
                }
            };

            token_map.insert(token.into(), whoami.email.clone());

            Ok(whoami.email)
        }
    }
}

pub fn is_systeminit_email(email: &str) -> bool {
    let email = email.to_lowercase();

    // Use rsplitn so you only split on the final @. The part after the last @ is the domain.
    // Attackers can insert extra @ characters in the local part. rsplitn(2, "@") gives
    // exactly two pieces when the input is valid. If you get anything else, treat the
    // email as invalid. Only accept an exact domain match.
    let parts: Vec<&str> = email.rsplitn(2, '@').collect();
    match parts.as_slice() {
        [domain, _local] => *domain == "systeminit.com",
        _ => false,
    }
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
