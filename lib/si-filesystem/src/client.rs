use std::collections::HashMap;

use si_frontend_types::{
    fs::{
        AssetFuncs, ChangeSet, CreateChangeSetRequest, CreateChangeSetResponse, Func,
        ListChangeSetsResponse, Schema, SchemaAttributes, SetFuncCodeRequest, VariantQuery,
    },
    FuncKind,
};
use si_id::{ChangeSetId, FuncId, SchemaId, WorkspaceId};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SiFsClientError {
    #[error("reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),
}

pub type SiFsClientResult<T> = Result<T, SiFsClientError>;

#[derive(Debug, Clone)]
pub struct SiFsClient {
    token: String,
    workspace_id: WorkspaceId,
    endpoint: String,
    client: reqwest::Client,
}

const USER_AGENT: &str = "si-fs/0.0";

#[derive(Debug, Clone)]
pub struct SchemaFunc {
    pub locked: Option<Func>,
    pub unlocked: Option<Func>,
}

impl SiFsClient {
    pub fn new(
        token: String,
        workspace_id: WorkspaceId,
        endpoint: String,
    ) -> SiFsClientResult<Self> {
        Ok(Self {
            token,
            workspace_id,
            endpoint,
            client: reqwest::Client::builder().user_agent(USER_AGENT).build()?,
        })
    }

    fn fs_api_url(&self, suffix: &str) -> String {
        format!(
            "{}/api/v2/workspaces/{}/fs/{suffix}",
            self.endpoint, self.workspace_id
        )
    }

    fn fs_api_change_sets(&self, suffix: &str, change_set_id: ChangeSetId) -> String {
        format!(
            "{}/api/v2/workspaces/{}/fs/change-sets/{change_set_id}/{suffix}",
            self.endpoint, self.workspace_id
        )
    }

    /// Fetches metadata about the workspace, including the active change sets
    pub async fn list_change_sets(&self) -> SiFsClientResult<ListChangeSetsResponse> {
        let response = self
            .client
            .get(self.fs_api_url("change-sets"))
            .bearer_auth(&self.token)
            .send()
            .await?
            .error_for_status()?;

        Ok(response.json().await?)
    }

    pub async fn create_change_set(&self, name: String) -> SiFsClientResult<ChangeSet> {
        let create_change_set_request = CreateChangeSetRequest { name };

        let response = self
            .client
            .post(self.fs_api_url("change-sets/create"))
            .bearer_auth(&self.token)
            .json(&create_change_set_request)
            .send()
            .await?
            .error_for_status()?;

        let response: CreateChangeSetResponse = response.json().await?;

        Ok(response)
    }

    pub async fn schemas(&self, change_set_id: ChangeSetId) -> SiFsClientResult<Vec<Schema>> {
        let response = self
            .client
            .get(self.fs_api_change_sets("schemas", change_set_id))
            .bearer_auth(&self.token)
            .send()
            .await?
            .error_for_status()?;

        Ok(response.json().await?)
    }

    pub async fn change_set_funcs_of_kind(
        &self,
        change_set_id: ChangeSetId,
        func_kind: FuncKind,
    ) -> SiFsClientResult<Vec<Func>> {
        let kind_string = si_frontend_types::fs::kind_to_string(func_kind);

        Ok(self
            .client
            .get(self.fs_api_change_sets(&format!("funcs/{kind_string}"), change_set_id))
            .bearer_auth(&self.token)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    pub async fn asset_funcs_for_variant(
        &self,
        change_set_id: ChangeSetId,
        schema_id: SchemaId,
    ) -> SiFsClientResult<AssetFuncs> {
        let response = self
            .client
            .get(
                self.fs_api_change_sets(&format!("schemas/{schema_id}/asset_funcs"), change_set_id),
            )
            .bearer_auth(&self.token)
            .send()
            .await?
            .error_for_status()?;

        Ok(response.json().await?)
    }

    pub async fn variant_funcs_of_kind(
        &self,
        change_set_id: ChangeSetId,
        schema_id: SchemaId,
        func_kind: FuncKind,
    ) -> SiFsClientResult<HashMap<String, SchemaFunc>> {
        let kind_string = si_frontend_types::fs::kind_to_string(func_kind);

        let funcs: Vec<Func> = self
            .client
            .get(self.fs_api_change_sets(
                &format!("schemas/{schema_id}/funcs/{kind_string}"),
                change_set_id,
            ))
            .bearer_auth(&self.token)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        let mut schema_funcs: HashMap<String, SchemaFunc> = HashMap::new();

        for func in funcs {
            schema_funcs
                .entry(func.name.clone())
                .and_modify(|f| {
                    if func.is_locked {
                        f.locked = Some(func.clone());
                    } else {
                        f.unlocked = Some(func.clone());
                    }
                })
                .or_insert_with(|| {
                    if func.is_locked {
                        SchemaFunc {
                            locked: Some(func),
                            unlocked: None,
                        }
                    } else {
                        SchemaFunc {
                            locked: None,
                            unlocked: Some(func),
                        }
                    }
                });
        }

        Ok(schema_funcs)
    }

    pub async fn get_func_code(
        &self,
        change_set_id: ChangeSetId,
        func_id: FuncId,
    ) -> SiFsClientResult<String> {
        Ok(self
            .client
            .get(self.fs_api_change_sets(&format!("func-code/{func_id}"), change_set_id))
            .bearer_auth(&self.token)
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?)
    }

    pub async fn set_func_code(
        &self,
        change_set_id: ChangeSetId,
        func_id: FuncId,
        code: String,
    ) -> SiFsClientResult<()> {
        self.client
            .post(self.fs_api_change_sets(&format!("func-code/{func_id}"), change_set_id))
            .json(&SetFuncCodeRequest { code })
            .bearer_auth(&self.token)
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    pub async fn set_asset_func_code(
        &self,
        change_set_id: ChangeSetId,
        func_id: FuncId,
        schema_id: SchemaId,
        code: String,
    ) -> SiFsClientResult<()> {
        self.client
            .post(self.fs_api_change_sets(
                &format!("schemas/{schema_id}/asset_func/{func_id}"),
                change_set_id,
            ))
            .json(&SetFuncCodeRequest { code })
            .bearer_auth(&self.token)
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    pub async fn install_schema(
        &self,
        change_set_id: ChangeSetId,
        schema_id: SchemaId,
    ) -> SiFsClientResult<()> {
        self.client
            .post(self.fs_api_change_sets(&format!("schemas/{schema_id}/install"), change_set_id))
            .bearer_auth(&self.token)
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    pub async fn get_schema_attrs(
        &self,
        change_set_id: ChangeSetId,
        schema_id: SchemaId,
        unlocked: bool,
    ) -> SiFsClientResult<SchemaAttributes> {
        Ok(self
            .client
            .get(self.fs_api_change_sets(&format!("schemas/{schema_id}/attrs"), change_set_id))
            .bearer_auth(&self.token)
            .query(&VariantQuery { unlocked })
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    pub async fn set_schema_attrs(
        &self,
        change_set_id: ChangeSetId,
        schema_id: SchemaId,
        attributes: SchemaAttributes,
    ) -> SiFsClientResult<()> {
        self.client
            .post(self.fs_api_change_sets(&format!("schemas/{schema_id}/attrs"), change_set_id))
            .bearer_auth(&self.token)
            .json(&attributes)
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }
}
