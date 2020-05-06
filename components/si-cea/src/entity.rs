use crate::component::ComponentSiProperties;
use prost::Message;
use serde::de::DeserializeOwned;
use serde::ser::Serialize;
use si_account::Workspace;
use si_data::{DataError, Result as DataResult, Storable};

pub use crate::protobuf::{EntitySiProperties, EntitySiPropertiesEntityState};

pub mod prelude {
    pub use super::{Entity, EntitySiProperties, EntitySiPropertiesEntityState};
    pub use crate::error::{CeaError, CeaResult};
    pub use crate::list::{ListReply, ListRequest};
    pub use si_data::error::DataError;
    pub use si_data::{DataQuery, ListResult, Storable as _};
}

pub trait Entity:
    Clone + std::fmt::Debug + Default + DeserializeOwned + Message + Serialize + Storable
{
    type EntityProperties;

    fn entity_state(&self) -> DataResult<EntitySiPropertiesEntityState>;
    fn set_entity_state(&mut self, state: EntitySiPropertiesEntityState);

    fn properties(&self) -> DataResult<&Self::EntityProperties>;
    fn properties_mut(&mut self) -> DataResult<&mut Self::EntityProperties>;

    fn integration_id(&self) -> DataResult<&str>;
    fn set_integration_id(&mut self, integration_id: impl Into<String>);

    fn integration_service_id(&self) -> DataResult<&str>;
    fn set_integration_service_id(&mut self, integration_service_id: impl Into<String>);

    fn component_id(&self) -> DataResult<&str>;
    fn set_component_id(&mut self, component_id: impl Into<String>);

    fn workspace_id(&self) -> DataResult<&str>;
    fn set_workspace_id(&mut self, workspace_id: impl Into<String>);

    fn organization_id(&self) -> DataResult<&str>;
    fn set_organization_id(&mut self, organization_id: impl Into<String>);

    fn billing_account_id(&self) -> DataResult<&str>;
    fn set_billing_account_id(&mut self, billing_account_id: impl Into<String>);

    fn set_entity_state_transition(&mut self) {
        self.set_entity_state(EntitySiPropertiesEntityState::Transition);
    }
}

impl EntitySiProperties {
    pub fn new(
        workspace: &Workspace,
        csp: &ComponentSiProperties,
        component_id: impl Into<String>,
    ) -> DataResult<Self> {
        Ok(Self {
            entity_state: Default::default(),
            integration_id: csp.integration_id.clone(),
            integration_service_id: csp.integration_service_id.clone(),
            component_id: Some(component_id.into()),
            workspace_id: Some(
                workspace
                    .id
                    .as_ref()
                    .ok_or_else(|| DataError::RequiredField("id".to_string()))?
                    .to_string(),
            ),
            organization_id: Some(
                workspace
                    .si_properties
                    .as_ref()
                    .ok_or_else(|| si_data::DataError::RequiredField("si_properties".to_string()))?
                    .organization_id
                    .as_ref()
                    .map(String::as_str)
                    .ok_or_else(|| {
                        si_data::DataError::RequiredField("organization_id".to_string())
                    })?
                    .to_string(),
            ),
            billing_account_id: Some(
                workspace
                    .si_properties
                    .as_ref()
                    .ok_or_else(|| si_data::DataError::RequiredField("si_properties".to_string()))?
                    .billing_account_id
                    .as_ref()
                    .map(String::as_str)
                    .ok_or_else(|| {
                        si_data::DataError::RequiredField("billing_accound_id".to_string())
                    })?
                    .to_string(),
            ),
            version: csp.version.clone(),
        })
    }
}
