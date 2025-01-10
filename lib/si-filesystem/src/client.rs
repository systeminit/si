use si_frontend_types::{
    ChangeSet, CreateChangeSetRequest, CreateChangeSetResponse, ListVariantsResponse,
    WorkspaceMetadata,
};
use si_id::{ChangeSetId, WorkspaceId};
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

    /// Fetches metadata about the workspace, including the active change sets
    pub async fn workspace_metadata(&self) -> SiFsClientResult<WorkspaceMetadata> {
        let response = self
            .client
            .get(format!(
                "{}/api/v2/workspaces/{}/change-sets",
                self.endpoint, self.workspace_id
            ))
            .bearer_auth(&self.token)
            .send()
            .await?
            .error_for_status()?;

        Ok(response.json().await?)
    }

    pub async fn create_change_set(&self, change_set_name: String) -> SiFsClientResult<ChangeSet> {
        let create_change_set_request = CreateChangeSetRequest { change_set_name };

        let response = self
            .client
            .post(format!(
                "{}/api/change_set/create_change_set",
                self.endpoint
            ))
            .bearer_auth(&self.token)
            .json(&create_change_set_request)
            .send()
            .await?
            .error_for_status()?;

        let response: CreateChangeSetResponse = response.json().await?;

        Ok(response.change_set)
    }

    pub async fn variants(
        &self,
        change_set_id: ChangeSetId,
    ) -> SiFsClientResult<ListVariantsResponse> {
        let response = self
            .client
            .get(format!(
                "{}/api/v2/workspaces/{}/change-sets/{change_set_id}/schema-variants",
                self.endpoint, self.workspace_id
            ))
            .bearer_auth(&self.token)
            .send()
            .await?
            .error_for_status()?;

        Ok(response.json().await?)
    }
}
