use crate::component::ComponentSiProperties;
use crate::error::CeaResult;
use crate::list::ListRequest;
use async_trait::async_trait;
use prost::Message;
use serde::de::DeserializeOwned;
use serde::ser::Serialize;
use si_account::Workspace;
use si_data::db::ListResult;
use si_data::{Db, Storable};

pub use crate::protobuf::{EntitySiProperties, EntitySiPropertiesEntityState};

pub mod codegen_prelude {
    pub use super::{Entity, EntitySiProperties};
    pub use crate::error::{CeaError, CeaResult};
    pub use crate::list::{ListReply, ListRequest};
    pub use si_account::Workspace;
    pub use si_data::error::{DataError, Result as DataResult};
    pub use si_data::{uuid_string, DataQuery, DataStorable, Db, ListResult, Reference, Storable};
}

pub mod prelude {
    pub use super::{Entity, EntitySiProperties, EntitySiPropertiesEntityState};
    pub use crate::error::{CeaError, CeaResult};
    pub use crate::list::{ListReply, ListRequest};
    pub use si_data::error::DataError;
    pub use si_data::{DataQuery, ListResult, Storable as _};
}

impl EntitySiProperties {
    pub fn new(
        workspace: &Workspace,
        csp: &ComponentSiProperties,
        component_id: impl Into<String>,
    ) -> Self {
        Self {
            entity_state: Default::default(),
            integration_id: csp.integration_id.clone(),
            integration_service_id: csp.integration_service_id.clone(),
            component_id: Some(component_id.into()),
            workspace_id: Some(workspace.id.to_string()),
            organization_id: Some(workspace.organization_id.to_string()),
            billing_account_id: Some(workspace.billing_account_id.to_string()),
            version: csp.version.clone(),
        }
    }
}

pub enum EntityState {
    Uninitialized = 0,
    Error = 1,
    Ok = 2,
    Transition = 3,
}

#[async_trait]
pub trait Entity:
    std::fmt::Debug + Message + Serialize + DeserializeOwned + Clone + Storable + Default
{
    fn id(&self) -> &str;
    fn set_id(&mut self, id: impl Into<String>);
    fn state(&self) -> i32;
    fn set_state(&mut self, state: i32);
    fn component_id(&self) -> &str;
    fn set_component_id(&mut self, component_id: impl Into<String>);
    fn integration_id(&self) -> &str;
    fn set_integration_id(&mut self, integration_id: impl Into<String>);
    fn integration_service_id(&self) -> &str;
    fn set_integration_service_id(&mut self, integration_service_id: impl Into<String>);
    fn workspace_id(&self) -> &str;
    fn set_workspace_id(&mut self, workspace_id: impl Into<String>);
    fn organization_id(&self) -> &str;
    fn set_organization_id(&mut self, organization_id: impl Into<String>);
    fn billing_account_id(&self) -> &str;
    fn set_billing_account_id(&mut self, billing_account_id: impl Into<String>);
    fn validate(&self) -> si_data::error::Result<()>;

    fn set_state_transition(&mut self) {
        self.set_state(EntityState::Transition as i32);
    }

    async fn save(&self, db: &Db) -> CeaResult<()> {
        db.upsert(self).await?;
        Ok(())
    }

    async fn get(db: &Db, id: &str) -> CeaResult<Self> {
        let entity = db.get(id).await?;
        Ok(entity)
    }

    async fn list<T: ListRequest>(db: &Db, list_request: &T) -> CeaResult<ListResult<Self>> {
        let result = match list_request.page_token() {
            Some(token) => db.list_by_page_token(token).await?,
            None => {
                db.list(
                    list_request.query(),
                    list_request.page_size(),
                    list_request.order_by(),
                    list_request.order_by_direction(),
                    list_request.scope_by_tenant_id(),
                    "",
                )
                .await?
            }
        };
        Ok(result)
    }
}
