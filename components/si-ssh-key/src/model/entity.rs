use si_account::Workspace;
use si_data::{error::DataError, Migrateable, Reference, Storable};
use uuid::Uuid;

use crate::error::{Result, SshKeyError};
pub use crate::model::component::{KeyFormat, KeyType};
pub use crate::protobuf::Entity;
use crate::protobuf::{Constraints, CreateEntityRequest, PickComponentReply, PickComponentRequest};

impl Storable for Entity {
    fn get_id(&self) -> &str {
        &self.id
    }

    fn set_id<S: Into<String>>(&mut self, id: S) {
        self.id = id.into();
    }

    fn type_name() -> &'static str {
        "ssh_key_entity"
    }

    fn set_type_name(&mut self) {
        self.type_name = Entity::type_name().to_string();
    }

    fn generate_id(&mut self) {
        let uuid = Uuid::new_v4();
        self.id = format!("{}:{}", Entity::type_name(), uuid);
    }

    fn validate(&self) -> si_data::error::Result<()> {
        if self.display_name == "" {
            return Err(DataError::ValidationError(
                SshKeyError::InvalidMissingDisplayName.to_string(),
            ));
        }
        if self.name == "" {
            return Err(DataError::ValidationError(
                SshKeyError::InvalidMissingName.to_string(),
            ));
        }
        Ok(())
    }

    fn get_tenant_ids(&self) -> &[String] {
        &self.tenant_ids
    }

    fn add_to_tenant_ids(&mut self, id: String) {
        self.tenant_ids.push(id);
    }

    fn referential_fields(&self) -> Vec<Reference> {
        vec![
            Reference::HasOne("component_id", &self.component_id),
            Reference::HasOne("integration_id", &self.integration_id),
            Reference::HasOne("integration_service_id", &self.integration_service_id),
            Reference::HasOne("workspace_id", &self.workspace_id),
            Reference::HasOne("organization_id", &self.organization_id),
            Reference::HasOne("billing_account_id", &self.billing_account_id),
        ]
    }

    fn get_natural_key(&self) -> Option<&str> {
        Some(&self.natural_key)
    }

    fn set_natural_key(&mut self) {
        self.natural_key = format!(
            "{}:{}:{}:{}:{}",
            // This is safe *only* after the object has been created.
            self.get_tenant_ids()[0],
            self.get_tenant_ids()[1],
            self.get_tenant_ids()[2],
            Entity::type_name(),
            self.name
        );
    }

    fn order_by_fields() -> Vec<&'static str> {
        vec![
            "id",
            "naturalKey",
            "typeName",
            "displayName",
            "name",
            "description",
            "bits",
            "keyType",
            "keyFormat",
            "comment",
            "bubble_babble",
            "fingerprint",
            "random_art",
            "private_key",
            "public_key",
            "state",
        ]
    }
}

impl Entity {
    pub fn from_request_and_component(
        req: &CreateEntityRequest,
        pick_component: PickComponentReply,
        workspace: Workspace,
    ) -> Entity {
        // Safe, because we didn't error way earlier.
        let component = pick_component.component.unwrap();
        let implicit_constraints = pick_component.implicit_constraints;

        let constraints: Option<Constraints> = match &req.constraints {
            Some(c) => Some(c.into()),
            None => None,
        };

        Entity {
            name: req.name.clone(),
            display_name: req.display_name.clone(),
            description: req.description.clone(),
            workspace_id: workspace.id.clone(),
            organization_id: workspace.organization_id.clone(),
            billing_account_id: workspace.billing_account_id.clone(),
            component_id: component.id,
            integration_id: component.integration_id,
            integration_service_id: component.integration_service_id,
            constraints,
            implicit_constraints: implicit_constraints.clone(),
            key_type: component.key_type,
            key_format: component.key_format,
            bits: component.bits,
            tenant_ids: vec![
                workspace.billing_account_id,
                workspace.organization_id,
                workspace.id,
            ],
            ..Default::default()
        }
    }
}

impl From<&PickComponentRequest> for Constraints {
    fn from(pcr: &PickComponentRequest) -> Self {
        Constraints {
            name: pcr.name.clone(),
            display_name: pcr.display_name.clone(),
            integration_id: pcr.integration_id.clone(),
            integration_service_id: pcr.integration_service_id.clone(),
            key_type: pcr.key_type.clone(),
            key_format: pcr.key_format.clone(),
            bits: pcr.bits.clone(),
        }
    }
}
