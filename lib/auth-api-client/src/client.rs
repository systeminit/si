use crate::types::{AuthApiClientError, AuthApiResult, WhoamiResponse};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Clone)]
pub struct AuthApiClient {
    base_url: Url,
    auth_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct UserWrap {
    user: WhoamiResponse,
}

impl AuthApiClient {
    pub fn new(base_url: Url, auth_token: &str) -> Self {
        Self {
            base_url,
            auth_token: auth_token.into(),
        }
    }

    pub async fn whoami(&self) -> AuthApiResult<WhoamiResponse> {
        let token_no_bearer = self
            .auth_token
            .strip_prefix("Bearer ")
            .ok_or(AuthApiClientError::AuthTokenNotBearer)?;

        let whoami_url = self.base_url.join("whoami")?;
        let whoami_response = reqwest::Client::new()
            .get(whoami_url)
            .header("Cookie", format!("si-auth={}", &token_no_bearer))
            .send()
            .await?
            .error_for_status()?;

        let whoami = whoami_response.json::<UserWrap>().await?.user;

        Ok(whoami)
    }
}
