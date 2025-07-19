use reqwest::IntoUrl;
use serde::{
    Deserialize,
    Serialize,
    de::DeserializeOwned,
};
use si_id::{
    AuthTokenId,
    WorkspacePk,
};
use url::Url;

use crate::types::{
    AuthApiClientError,
    AuthApiResult,
    GetAuthTokenResponse,
    StatusResponse,
    WhoamiResponse,
};

#[derive(Debug, Clone)]
pub struct AuthApiClient {
    base_url: Url,
    raw_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct UserWrap {
    user: WhoamiResponse,
}

impl AuthApiClient {
    pub fn from_bearer_token(base_url: impl IntoUrl, bearer_token: &str) -> AuthApiResult<Self> {
        let raw_token = bearer_token
            .strip_prefix("Bearer ")
            .ok_or(AuthApiClientError::AuthTokenNotBearer)?;
        Self::from_raw_token(base_url, raw_token.into())
    }

    pub fn from_raw_token(base_url: impl IntoUrl, raw_token: String) -> AuthApiResult<Self> {
        Ok(Self {
            base_url: base_url.into_url()?,
            raw_token,
        })
    }

    pub async fn status(&self) -> AuthApiResult<StatusResponse> {
        self.get("/").await
    }

    pub async fn whoami(&self) -> AuthApiResult<WhoamiResponse> {
        // TODO switch this over to use get(); cookies and bearer auth are equally supported
        // self.get("whoami").await
        let whoami_url = self.base_url.join("whoami")?;
        let whoami_response = reqwest::Client::new()
            .get(whoami_url)
            .header("Cookie", format!("si-auth={}", self.raw_token))
            .send()
            .await?
            .error_for_status()?;

        let whoami = whoami_response.json::<UserWrap>().await?.user;

        Ok(whoami)
    }

    pub async fn get_auth_token(
        &self,
        workspace_id: WorkspacePk,
        token_id: AuthTokenId,
    ) -> AuthApiResult<GetAuthTokenResponse> {
        self.get(&format!("workspaces/{workspace_id}/authTokens/{token_id}"))
            .await
    }

    async fn get<T: DeserializeOwned>(&self, path: &str) -> AuthApiResult<T> {
        let url = self.base_url.join(path)?;
        let response = reqwest::Client::new()
            .get(url)
            .bearer_auth(&self.raw_token)
            .send()
            .await?
            .error_for_status()?;
        let text = response.text().await?;
        Ok(serde_json::from_str(&text)?)
        // println!("Response: {}", response.text().await?);
        // Err(AuthApiClientError::AuthTokenNotBearer)
        // Ok(response.json::<T>().await?)
    }
}
