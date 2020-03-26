use si_cea::entity::prelude::*;

use si_account::Workspace;

pub use crate::model::component::{KeyFormat, KeyType};
pub use crate::protobuf::entity::State;
pub use crate::protobuf::entity_event::NextState;
use crate::protobuf::{
    Constraints, CreateEntityRequest, ListEntitiesReply, ListEntitiesRequest,
    ListEntityEventsReply, ListEntityEventsRequest, PickComponentReply, PickComponentRequest,
};
pub use crate::protobuf::{Entity, EntityEvent};

gen_entity!(
    type_name: "ssh_key_entity",
    order_by_fields: [
        "bits",
        "keyType",
        "keyFormat",
        "comment",
        "bubble_babble",
        "fingerprint",
        "random_art",
        "private_key",
        "public_key",
        "state"
    ],
    validate_fn: |self| {
        if self.display_name == "" {
            return Err(CeaError::ValidationError("Missing display name".to_string()));
        }
        if self.name == "" {
            return Err(CeaError::ValidationError("Missing name".to_string()));
        }
        Ok(())
    }
);

impl Entity {
    pub async fn from_request_and_component(
        db: &si_data::Db,
        req: &CreateEntityRequest,
        pick_component: PickComponentReply,
        workspace: Workspace,
    ) -> si_cea::Result<Entity> {
        // Safe, because we didn't error way earlier.
        let component = pick_component.component.unwrap();
        let implicit_constraints = pick_component.implicit_constraints;

        let constraints: Option<Constraints> = match &req.constraints {
            Some(c) => Some(c.into()),
            None => None,
        };

        let mut entity = Entity {
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
        };
        db.validate_and_insert_as_new(&mut entity).await?;
        Ok(entity)
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

gen_entity_event!(
    type_name: "ssh_key_entity_event",
    action_names: [ "create", "sync" ]
);
