// Re-export all module index types so that client users do not have to import two crates.
pub use module_index_types::*;
use reqwest::{
    StatusCode,
    header::{
        self,
        HeaderMap,
        HeaderValue,
    },
};
use si_pkg::WorkspaceExport;
use thiserror::Error;
use ulid::Ulid;
use url::Url;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum ModuleIndexClientError {
    #[error("Deserialization error: {0}")]
    Deserialization(serde_json::Error),
    #[error("Request error: {0}")]
    InvalidHeaderValue(#[from] reqwest::header::InvalidHeaderValue),
    #[error("module not found (module id: {0})")]
    ModuleNotFound(String),
    #[error("Request error: {0}")]
    Request(#[from] reqwest::Error),
    #[error("Serialization error: {0}")]
    Serialization(serde_json::Error),
    #[error("Url parse error: {0}")]
    UrlParse(#[from] url::ParseError),
}

pub type ModuleIndexClientResult<T> = Result<T, ModuleIndexClientError>;

#[derive(Debug, Clone)]
pub struct ModuleIndexClient {
    inner: reqwest::Client,
    base_url: Url,
}

impl ModuleIndexClient {
    pub fn new(base_url: Url, auth_token: &str) -> ModuleIndexClientResult<Self> {
        let headers = {
            let mut headers = HeaderMap::new();
            let mut auth_header_value = HeaderValue::from_str(&format!("Bearer {auth_token}"))?;
            auth_header_value.set_sensitive(true);
            headers.insert(header::AUTHORIZATION, auth_header_value);
            headers
        };

        let inner = reqwest::Client::builder()
            .default_headers(headers)
            .build()?;

        Ok(Self { inner, base_url })
    }

    pub fn unauthenticated_client(base_url: Url) -> ModuleIndexClientResult<Self> {
        let inner = reqwest::Client::builder().build()?;

        Ok(Self { inner, base_url })
    }

    pub async fn reject_module(
        &self,
        module_id: Ulid,
        rejected_by_display_name: String,
    ) -> ModuleIndexClientResult<ModuleRejectionResponse> {
        let reject_url = self
            .base_url
            .join("modules/")?
            .join(&format!("{}/", module_id.to_string()))?
            .join("reject")?;

        let upload_response = self
            .inner
            .post(reject_url)
            .multipart(
                reqwest::multipart::Form::new().text("rejected by user", rejected_by_display_name),
            )
            .send()
            .await?;

        if upload_response.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(ModuleIndexClientError::ModuleNotFound(
                module_id.to_string(),
            ));
        }

        let upload_response = upload_response.error_for_status()?;

        Ok(upload_response.json::<ModuleRejectionResponse>().await?)
    }

    pub async fn promote_to_builtin(
        &self,
        module_id: Ulid,
        promoted_to_builtin_by_display_name: String,
    ) -> ModuleIndexClientResult<ModulePromotedResponse> {
        let reject_url = self
            .base_url
            .join("builtins/")?
            .join(&format!("{}/", module_id.to_string()))?
            .join("promote")?;

        let promote_response = self
            .inner
            .post(reject_url)
            .multipart(
                reqwest::multipart::Form::new()
                    .text("promoted by user", promoted_to_builtin_by_display_name),
            )
            .send()
            .await?;

        if promote_response.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(ModuleIndexClientError::ModuleNotFound(
                module_id.to_string(),
            ));
        }

        let promote_response = promote_response.error_for_status()?;

        Ok(promote_response.json::<ModulePromotedResponse>().await?)
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn upload_module(
        &self,
        module_name: &str,
        module_version: &str,
        module_based_on_hash: Option<String>,
        module_schema_id: Option<String>,
        module_bytes: Vec<u8>,
        module_schema_variant_id: Option<String>,
        module_schema_variant_version: Option<String>,
        module_is_private_scoped: Option<bool>,
    ) -> ModuleIndexClientResult<ModuleDetailsResponse> {
        let module_upload_part = reqwest::multipart::Part::bytes(module_bytes)
            .file_name(format!("{module_name}_{module_version}.tar"));

        let mut multipart_form =
            reqwest::multipart::Form::new().part(MODULE_BUNDLE_FIELD_NAME, module_upload_part);

        if let Some(module_based_on_hash) = module_based_on_hash {
            multipart_form = multipart_form.part(
                MODULE_BASED_ON_HASH_FIELD_NAME,
                reqwest::multipart::Part::text(module_based_on_hash),
            );
        }

        if let Some(schema_id) = module_schema_id {
            multipart_form = multipart_form.part(
                MODULE_SCHEMA_ID_FIELD_NAME,
                reqwest::multipart::Part::text(schema_id),
            );
        }

        if let Some(schema_variant_id) = module_schema_variant_id {
            multipart_form = multipart_form.part(
                MODULE_SCHEMA_VARIANT_ID_FIELD_NAME,
                reqwest::multipart::Part::text(schema_variant_id),
            );
        }

        if let Some(schema_variant_version) = module_schema_variant_version {
            multipart_form = multipart_form.part(
                MODULE_SCHEMA_VARIANT_VERSION_FIELD_NAME,
                reqwest::multipart::Part::text(schema_variant_version),
            );
        }

        if let Some(is_private_scoped) = module_is_private_scoped {
            multipart_form = multipart_form.part(
                MODULE_IS_PRIVATE_SCOPED_FIELD_NAME,
                reqwest::multipart::Part::text(is_private_scoped.to_string()),
            );
        }

        let upload_url = self.base_url.join("modules")?;
        let upload_response = self
            .inner
            .post(upload_url)
            .multipart(multipart_form)
            .send()
            .await?
            .error_for_status()?;

        Ok(upload_response.json::<ModuleDetailsResponse>().await?)
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn upsert_builtin(
        &self,
        module_name: &str,
        module_version: &str,
        module_based_on_hash: Option<String>,
        module_schema_id: Option<String>,
        module_bytes: Vec<u8>,
        module_schema_variant_id: Option<String>,
        module_schema_variant_version: Option<String>,
    ) -> ModuleIndexClientResult<bool> {
        let module_upload_part = reqwest::multipart::Part::bytes(module_bytes)
            .file_name(format!("{module_name}_{module_version}.tar"));

        let mut multipart_form =
            reqwest::multipart::Form::new().part(MODULE_BUNDLE_FIELD_NAME, module_upload_part);

        if let Some(module_based_on_hash) = module_based_on_hash {
            multipart_form = multipart_form.part(
                MODULE_BASED_ON_HASH_FIELD_NAME,
                reqwest::multipart::Part::text(module_based_on_hash),
            );
        }

        if let Some(schema_id) = module_schema_id {
            multipart_form = multipart_form.part(
                MODULE_SCHEMA_ID_FIELD_NAME,
                reqwest::multipart::Part::text(schema_id),
            );
        }

        if let Some(schema_variant_id) = module_schema_variant_id {
            multipart_form = multipart_form.part(
                MODULE_SCHEMA_VARIANT_ID_FIELD_NAME,
                reqwest::multipart::Part::text(schema_variant_id),
            );
        }

        if let Some(schema_variant_version) = module_schema_variant_version {
            multipart_form = multipart_form.part(
                MODULE_SCHEMA_VARIANT_VERSION_FIELD_NAME,
                reqwest::multipart::Part::text(schema_variant_version),
            );
        }

        let upsert_url = self.base_url.join("builtins/upsert")?;
        let upsert_response = self
            .inner
            .post(upsert_url)
            .multipart(multipart_form)
            .send()
            .await?
            .error_for_status()?;

        Ok(upsert_response.json::<bool>().await?)
    }

    pub async fn download_module(&self, module_id: Ulid) -> ModuleIndexClientResult<Vec<u8>> {
        let download_url = self
            .base_url
            .join("modules/")?
            .join(&format!("{}/", module_id.to_string()))?
            .join("download")?;
        let response = self.inner.get(download_url).send().await?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(ModuleIndexClientError::ModuleNotFound(
                module_id.to_string(),
            ));
        }

        let response = response.error_for_status()?;
        let bytes = response.bytes().await?;

        Ok(bytes.to_vec())
    }

    pub async fn list_builtins(&self) -> ModuleIndexClientResult<BuiltinsDetailsResponse> {
        let url = self.base_url.join("builtins")?;
        let resp = self.inner.get(url).send().await?.error_for_status()?;

        let mut builtins = resp.json::<BuiltinsDetailsResponse>().await?;

        if builtins.modules.is_empty()
            && self.base_url.clone().as_str().contains("http://localhost")
        {
            // We want to fall back to the production module index to pull builtins from there instead
            let url = Url::parse("https://module-index.systeminit.com")?.join("builtins")?;

            let resp = self.inner.get(url).send().await?.error_for_status()?;

            builtins = resp.json::<BuiltinsDetailsResponse>().await?
        };

        Ok(builtins)
    }

    pub async fn module_details(
        &self,
        module_id: Ulid,
    ) -> ModuleIndexClientResult<ModuleDetailsResponse> {
        let details_url = self
            .base_url
            .join("modules/")?
            .join(&format!("{module_id}"))?;

        let response = self.inner.get(details_url).send().await?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(ModuleIndexClientError::ModuleNotFound(
                module_id.to_string(),
            ));
        }

        let response = response.error_for_status()?;
        let response = response.json().await?;

        Ok(response)
    }

    pub async fn get_builtin(&self, module_id: Ulid) -> ModuleIndexClientResult<Vec<u8>> {
        let download_url = self
            .base_url
            .join("modules/")?
            .join(&format!("{}/", module_id.to_string()))?
            .join("download_builtin")?;

        let mut response = self.inner.get(download_url).send().await?;

        if response.status() == StatusCode::NOT_FOUND
            && self.base_url.clone().as_str().contains("http://localhost")
        {
            // We want to fall back to the production module index to pull builtins from there instead
            let url = Url::parse("https://module-index.systeminit.com")?
                .join("modules/")?
                .join(&format!("{}/", module_id.to_string()))?
                .join("download_builtin")?;

            response = self.inner.get(url).send().await?;
        };

        let bytes = response.error_for_status()?.bytes().await?;

        Ok(bytes.to_vec())
    }

    pub async fn upload_workspace(
        &self,
        workspace_name: &str,
        workspace_version: &str,
        content: WorkspaceExport,
    ) -> ModuleIndexClientResult<()> {
        let bytes = serde_json::to_vec(&content).map_err(ModuleIndexClientError::Serialization)?;

        let upload_part = reqwest::multipart::Part::bytes(bytes)
            .file_name(format!("{workspace_name}_{workspace_version}.tar"));

        let upload_url = self.base_url.join("workspace")?;

        self.inner
            .post(upload_url)
            .multipart(reqwest::multipart::Form::new().part("workspace bundle", upload_part))
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    pub async fn download_workspace(
        &self,
        module_id: Ulid,
    ) -> ModuleIndexClientResult<WorkspaceExport> {
        let download_url = self
            .base_url
            .join("workspace/")?
            .join(&format!("{}/", module_id.to_string()))?
            .join("download")?;
        let response = self
            .inner
            .get(download_url)
            .send()
            .await?
            .error_for_status()?;

        let bytes = response.bytes().await?;

        let export_data: WorkspaceExport =
            serde_json::from_slice(&bytes).map_err(ModuleIndexClientError::Deserialization)?;

        // Deserialize back into export object
        Ok(export_data)
    }

    /// Lists all of the latest, _promoted_ [`Modules`](Model) (route: GET /modules/latest).
    pub async fn list_latest_modules(&self) -> ModuleIndexClientResult<ListLatestModulesResponse> {
        let url = self.base_url.join("modules/")?.join("latest")?;

        Ok(self
            .inner
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    // Will skip builtins
    pub async fn list_module_details(&self) -> ModuleIndexClientResult<ListModulesResponse> {
        let url = self.base_url.join("modules")?;

        Ok(self
            .inner
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }
}
