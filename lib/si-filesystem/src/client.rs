use si_frontend_types::fs::{
    ChangeSet, CreateChangeSetRequest, CreateChangeSetResponse, ListChangeSetsResponse,
    ListVariantsResponse, Schema,
};
use si_id::{ChangeSetId, SchemaId, WorkspaceId};
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

    pub async fn variants(
        &self,
        change_set_id: ChangeSetId,
        schema_id: SchemaId,
    ) -> SiFsClientResult<ListVariantsResponse> {
        let response = self
            .client
            .get(self.fs_api_change_sets(&format!("schemas/{schema_id}/variants"), change_set_id))
            .bearer_auth(&self.token)
            .send()
            .await?
            .error_for_status()?;

        Ok(response.json().await?)
    }
}
