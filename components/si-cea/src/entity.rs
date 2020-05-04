use crate::component::ComponentSiProperties;
use async_trait::async_trait;
use prost::Message;
use serde::de::DeserializeOwned;
use serde::ser::Serialize;
use si_account::Workspace;
use si_data::Storable;

pub use crate::protobuf::{EntitySiProperties, EntitySiPropertiesEntityState};

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
            workspace_id: Some(
                workspace
                    .id
                    .as_ref()
                    .expect("workspace should have id")
                    .to_string(),
            ),
            organization_id: Some(
                workspace
                    .si_properties
                    .as_ref()
                    .expect("TODO: fix")
                    .organization_id
                    .as_ref()
                    .expect("workspace should have organization_id")
                    .to_string(),
            ),
            billing_account_id: Some(
                workspace
                    .si_properties
                    .as_ref()
                    .expect("TODO: fix")
                    .billing_account_id
                    .as_ref()
                    .expect("workspace should have billing_account_id")
                    .to_string(),
            ),
            version: csp.version.clone(),
        }
    }
}

#[async_trait]
pub trait Entity:
    std::fmt::Debug + Message + Serialize + DeserializeOwned + Clone + Storable + Default
{
    fn id(&self) -> &str;
    fn set_id(&mut self, id: impl Into<String>);
    fn state(&self) -> i32;
    fn set_state(&mut self, state: EntitySiPropertiesEntityState);
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

    fn set_state_transition(&mut self) {
        self.set_state(EntitySiPropertiesEntityState::Transition);
    }
}
