use reqwest::StatusCode;
use ulid::Ulid;
use url::Url;

use crate::types::{BuiltinsDetailsResponse, ModulePromotedResponse, ModuleRejectionResponse};
use crate::{IndexClientResult, ModuleDetailsResponse};

#[derive(Debug, Clone)]
pub struct IndexClient {
    base_url: Url,
    auth_token: String,
}

impl IndexClient {
    pub fn new(base_url: Url, auth_token: &str) -> Self {
        Self {
            base_url,
            auth_token: auth_token.to_owned(),
        }
    }

    pub fn unauthenticated_client(base_url: Url) -> Self {
        Self {
            base_url,
            auth_token: "".to_string(),
        }
    }

    pub async fn reject_module(
        &self,
        module_id: Ulid,
        rejected_by_display_name: String,
    ) -> IndexClientResult<ModuleRejectionResponse> {
        let reject_url = self
            .base_url
            .join("modules/")?
            .join(&format!("{}/", module_id.to_string()))?
            .join("reject")?;

        let upload_response = reqwest::Client::new()
            .post(reject_url)
            .multipart(
                reqwest::multipart::Form::new().text("rejected by user", rejected_by_display_name),
            )
            .bearer_auth(&self.auth_token)
            .send()
            .await?
            .error_for_status()?;

        Ok(upload_response.json::<ModuleRejectionResponse>().await?)
    }

    pub async fn promote_to_builtin(
        &self,
        module_id: Ulid,
        promoted_to_builtin_by_display_name: String,
    ) -> IndexClientResult<ModulePromotedResponse> {
        let reject_url = self
            .base_url
            .join("builtins/")?
            .join(&format!("{}/", module_id.to_string()))?
            .join("promote")?;

        let promote_response = reqwest::Client::new()
            .post(reject_url)
            .multipart(
                reqwest::multipart::Form::new()
                    .text("promoted by user", promoted_to_builtin_by_display_name),
            )
            .bearer_auth(&self.auth_token)
            .send()
            .await?
            .error_for_status()?;

        Ok(promote_response.json::<ModulePromotedResponse>().await?)
    }

    pub async fn upload_module(
        &self,
        module_name: &str,
        module_version: &str,
        module_bytes: Vec<u8>,
    ) -> IndexClientResult<ModuleDetailsResponse> {
        let module_upload_part = reqwest::multipart::Part::bytes(module_bytes)
            .file_name(format!("{module_name}_{module_version}.tar"));

        let upload_url = self.base_url.join("modules")?;
        let upload_response = reqwest::Client::new()
            .post(upload_url)
            .multipart(reqwest::multipart::Form::new().part("module bundle", module_upload_part))
            .bearer_auth(&self.auth_token)
            .send()
            .await?
            .error_for_status()?;

        Ok(upload_response.json::<ModuleDetailsResponse>().await?)
    }

    pub async fn download_module(&self, module_id: Ulid) -> IndexClientResult<Vec<u8>> {
        let download_url = self
            .base_url
            .join("modules/")?
            .join(&format!("{}/", module_id.to_string()))?
            .join("download")?;
        let response = reqwest::Client::new()
            .get(download_url)
            .bearer_auth(&self.auth_token)
            .send()
            .await?
            .error_for_status()?;

        let bytes = response.bytes().await?;

        Ok(bytes.to_vec())
    }

    pub async fn list_builtins(&self) -> IndexClientResult<BuiltinsDetailsResponse> {
        let url = self.base_url.join("builtins")?;
        let resp = reqwest::Client::new()
            .get(url)
            .bearer_auth(&self.auth_token)
            .send()
            .await?
            .error_for_status()?;

        let mut builtins = resp.json::<BuiltinsDetailsResponse>().await?;

        if builtins.modules.is_empty()
            && self.base_url.clone().as_str().contains("http://localhost")
        {
            // We want to fall back to the production module index to pull builtins from there instead
            let url = Url::parse("https://module-index.systeminit.com")?.join("builtins")?;

            let resp = reqwest::Client::new()
                .get(url)
                .bearer_auth(&self.auth_token)
                .send()
                .await?
                .error_for_status()?;

            builtins = resp.json::<BuiltinsDetailsResponse>().await?
        };

        Ok(builtins)
    }

    pub async fn get_builtin(&self, module_id: Ulid) -> IndexClientResult<Vec<u8>> {
        let download_url = self
            .base_url
            .join("modules/")?
            .join(&format!("{}/", module_id.to_string()))?
            .join("download_builtin")?;
        let mut response = reqwest::Client::new().get(download_url).send().await?;

        if response.status() == StatusCode::NOT_FOUND
            && self.base_url.clone().as_str().contains("http://localhost")
        {
            // We want to fall back to the production module index to pull builtins from there instead
            let url = Url::parse("https://module-index.systeminit.com")?
                .join("modules/")?
                .join(&format!("{}/", module_id.to_string()))?
                .join("download_builtin")?;

            let prod_response = reqwest::Client::new().get(url).send().await?;

            response = prod_response
        }

        let bytes = response.error_for_status()?.bytes().await?;

        Ok(bytes.to_vec())
    }
}
