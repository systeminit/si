use chrono::prelude::{DateTime, Utc};
use si_account::Workspace;
use si_data::{error::DataError, Reference, Storable};
use uuid::Uuid;

use crate::error::SshKeyError;
pub use crate::model::component::{KeyFormat, KeyType};
pub use crate::protobuf::entity::State;
pub use crate::protobuf::entity_event::NextState;
use crate::protobuf::{Constraints, CreateEntityRequest, PickComponentReply, PickComponentRequest};
pub use crate::protobuf::{Entity, EntityEvent};

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

impl Storable for EntityEvent {
    fn get_id(&self) -> &str {
        &self.id
    }

    fn set_id<S: Into<String>>(&mut self, id: S) {
        self.id = id.into();
    }

    fn type_name() -> &'static str {
        "ssh_key_entity_event"
    }

    fn set_type_name(&mut self) {
        self.type_name = EntityEvent::type_name().to_string();
    }

    fn generate_id(&mut self) {
        let uuid = Uuid::new_v4();
        self.id = format!("{}:{}", EntityEvent::type_name(), uuid);
    }

    fn validate(&self) -> si_data::error::Result<()> {
        if self.action_name == "" {
            return Err(DataError::ValidationError(
                SshKeyError::InvalidMissingActionName.to_string(),
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
            Reference::HasOne("entity_id", &self.entity_id),
            Reference::HasOne("component_id", &self.component_id),
            Reference::HasOne("integration_id", &self.integration_id),
            Reference::HasOne("integration_service_id", &self.integration_service_id),
            Reference::HasOne("workspace_id", &self.workspace_id),
            Reference::HasOne("organization_id", &self.organization_id),
            Reference::HasOne("billing_account_id", &self.billing_account_id),
        ]
    }

    fn get_natural_key(&self) -> Option<&str> {
        None
        //Some(&self.natural_key)
    }

    fn set_natural_key(&mut self) {
        self.natural_key = self.id.clone();
        //self.natural_key = format!(
        //    "{}:{}:{}:{}:{}:{}",
        //    // This is safe *only* after the object has been created.
        //    self.get_tenant_ids()[0],
        //    self.get_tenant_ids()[1],
        //    self.get_tenant_ids()[2],
        //    self.get_tenant_ids()[3],
        //    EntityEvent::type_name(),
        //    self.name
        //);
    }

    fn order_by_fields() -> Vec<&'static str> {
        vec![
            "id",
            "naturalKey",
            "typeName",
            "userId",
            "actionName",
            "createTime",
            "updatedTime",
            "finalTime",
            "finalized",
            "entityId",
            "componentId",
            "integrationId",
            "integrationServiceId",
            "workspaceId",
            "organizationId",
            "billingAccountId",
        ]
    }
}

impl EntityEvent {
    pub fn new(user_id: &str, action_name: &str, entity: &Entity) -> EntityEvent {
        let create_time: DateTime<Utc> = Utc::now();
        let create_time_string = format!("{}", create_time);
        EntityEvent {
            user_id: user_id.to_string(),
            action_name: action_name.to_string(),
            create_time: create_time_string,
            entity_id: entity.id.clone(),
            component_id: entity.component_id.clone(),
            integration_id: entity.integration_id.clone(),
            integration_service_id: entity.integration_service_id.clone(),
            workspace_id: entity.workspace_id.clone(),
            organization_id: entity.organization_id.clone(),
            billing_account_id: entity.billing_account_id.clone(),
            input_entity: Some(entity.clone()),
            tenant_ids: vec![
                entity.billing_account_id.to_string(),
                entity.organization_id.to_string(),
                entity.workspace_id.to_string(),
                entity.id.to_string(),
            ],
            ..Default::default()
        }
    }

    pub fn log(&mut self, line: impl Into<String>) {
        self.output_lines.push(line.into());
    }

    pub fn error_log(&mut self, line: impl Into<String>) {
        self.error_lines.push(line.into());
    }

    pub fn fail(&mut self, err: impl std::error::Error) {
        let time: DateTime<Utc> = Utc::now();
        let time_string = format!("{}", time);
        self.success = false;
        self.updated_time = time_string.clone();
        self.final_time = time_string;
        self.finalized = true;
        self.error_message = err.to_string();
        if self.output_entity.is_none() {
            if self.input_entity.is_some() {
                let mut output_entity = self.input_entity.clone();
                // You're safe, because we just checked... twice!
                output_entity.as_mut().unwrap().state = State::Error as i32;
                self.output_entity = output_entity;
            }
        }
        self.log("*** Task failed ***");
    }

    pub fn success(&mut self) {
        let time: DateTime<Utc> = Utc::now();
        let time_string = format!("{}", time);
        self.success = true;
        self.updated_time = time_string.clone();
        self.final_time = time_string;
        self.finalized = true;
        if self.output_entity.is_none() {
            // What happens if there is no input entity, and no output entity?
            if self.input_entity.is_some() {
                let mut output_entity = self.input_entity.clone();
                // You're safe, because we just checked... twice!
                output_entity.as_mut().unwrap().state = State::Ok as i32;
                self.output_entity = output_entity;
            }
        } else {
            self.output_entity.as_mut().unwrap().state = State::Ok as i32;
        }
        self.log("*** Task Succeeded ***");
    }
}
