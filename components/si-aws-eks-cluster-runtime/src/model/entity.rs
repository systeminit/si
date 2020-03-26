use crate::error::AwsEksClusterRuntimeError;
pub use crate::protobuf::create_entity_request::TagRequest;
pub use crate::protobuf::entity::{Bool, State, Tag};
pub use crate::protobuf::entity_event::NextState;
use crate::protobuf::{Constraints, CreateEntityRequest, PickComponentReply, PickComponentRequest};
pub use crate::protobuf::{Entity, EntityEvent};
use chrono::prelude::{DateTime, Utc};
use si_account::Workspace;
use si_data::{error::DataError, Reference, Storable};
use std::convert::TryFrom;
use uuid::Uuid;

const DEFAULT_NODEGROUP_DESIRED_SIZE: u32 = 2;
const DEFAULT_NODEGROUP_DISK_SIZE: u32 = 20;
const DEFAULT_NODEGROUP_INSTANCE_TYPE: &str = "t3.medium";
const DEFAULT_NODEGROUP_MAX_SIZE: u32 = 2;
const DEFAULT_NODEGROUP_MIN_SIZE: u32 = 2;
const DEFAULT_ENDPOINT_PRIVATE_ACCESS: Bool = Bool::False;
const DEFAULT_ENDPOINT_PUBLIC_ACCESS: Bool = Bool::True;

impl From<&TagRequest> for Tag {
    fn from(tagreq: &TagRequest) -> Tag {
        Tag {
            key: tagreq.key.clone(),
            value: tagreq.value.clone(),
        }
    }
}

#[derive(thiserror::Error, Debug)]
#[error("unknown Bool value")]
pub struct UnknownBoolError(());

#[derive(thiserror::Error, Debug)]
#[error("invalid Bool value: {0}")]
pub struct InvalidBoolError(i32);

impl TryFrom<Bool> for bool {
    type Error = UnknownBoolError;

    fn try_from(value: Bool) -> std::result::Result<Self, Self::Error> {
        match value {
            Bool::Unknown => Err(UnknownBoolError(())),
            Bool::True => Ok(true),
            Bool::False => Ok(false),
        }
    }
}

impl TryFrom<i32> for Bool {
    type Error = InvalidBoolError;

    fn try_from(value: i32) -> std::result::Result<Self, Self::Error> {
        Self::from_i32(value).ok_or(InvalidBoolError(value))
    }
}

impl From<bool> for Bool {
    fn from(value: bool) -> Self {
        if value {
            Self::True
        } else {
            Self::False
        }
    }
}

impl Storable for Entity {
    fn get_id(&self) -> &str {
        &self.id
    }

    fn set_id(&mut self, id: impl Into<String>) {
        self.id = id.into();
    }

    fn type_name() -> &'static str {
        "aws_eks_cluster_runtime_entity"
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
                AwsEksClusterRuntimeError::InvalidMissingDisplayName.to_string(),
            ));
        }
        if self.name == "" {
            return Err(DataError::ValidationError(
                AwsEksClusterRuntimeError::InvalidMissingName.to_string(),
            ));
        }
        Ok(())
    }

    fn get_tenant_ids(&self) -> &[String] {
        &self.tenant_ids
    }

    fn add_to_tenant_ids(&mut self, id: impl Into<String>) {
        self.tenant_ids.push(id.into());
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
            "kubernetes_version",
            "cloudwatch_logs",
            "node_group_ssh_key_id",
            "node_group_aws_instance_type",
            "node_group_disk_size_gib",
            "node_group_minimum_size",
            "node_group_maximum_size",
            "node_group_desired_size",
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

        let mut tags = vec![];
        for tagreq in req.tags.iter() {
            tags.push(Tag::from(tagreq));
        }

        let mut e = Entity {
            tenant_ids: vec![
                workspace.billing_account_id.clone(),
                workspace.organization_id.clone(),
                workspace.id.clone(),
            ],
            name: req.name.clone(),
            display_name: req.display_name.clone(),
            description: req.description.clone(),
            component_id: component.id,
            integration_id: component.integration_id,
            integration_service_id: component.integration_service_id,
            workspace_id: workspace.id,
            organization_id: workspace.organization_id,
            billing_account_id: workspace.billing_account_id,
            constraints,
            implicit_constraints,
            kubernetes_version: component.kubernetes_version,
            cloudwatch_logs: req.cloudwatch_logs,
            nodegroup_ssh_key_id: req.nodegroup_ssh_key_id.to_string(),
            tags,
            ..Default::default()
        };

        // TODO(fnicho): this becomes a question/requirement, but for now we'll set the default
        e.endpoint_private_access = DEFAULT_ENDPOINT_PRIVATE_ACCESS.into();
        // TODO(fnicho): this becomes a question/requirement, but for now we'll set the default
        e.endpoint_public_access = DEFAULT_ENDPOINT_PUBLIC_ACCESS.into();
        e.nodegroup_instance_type = if req.nodegroup_instance_type.is_empty() {
            DEFAULT_NODEGROUP_INSTANCE_TYPE.to_string()
        } else {
            req.nodegroup_instance_type.clone()
        };
        e.nodegroup_disk_size = if req.nodegroup_disk_size == 0 {
            DEFAULT_NODEGROUP_DISK_SIZE
        } else {
            req.nodegroup_disk_size
        };
        e.nodegroup_min_size = if req.nodegroup_min_size == 0 {
            DEFAULT_NODEGROUP_MIN_SIZE
        } else {
            req.nodegroup_min_size
        };
        e.nodegroup_max_size = if req.nodegroup_max_size == 0 {
            DEFAULT_NODEGROUP_MAX_SIZE
        } else {
            req.nodegroup_max_size
        };
        e.nodegroup_desired_size = if req.nodegroup_desired_size == 0 {
            DEFAULT_NODEGROUP_DESIRED_SIZE
        } else {
            req.nodegroup_desired_size
        };

        e
    }
}

impl From<&PickComponentRequest> for Constraints {
    fn from(pcr: &PickComponentRequest) -> Self {
        Constraints {
            name: pcr.name.clone(),
            display_name: pcr.display_name.clone(),
            integration_id: pcr.integration_id.clone(),
            integration_service_id: pcr.integration_service_id.clone(),
            kubernetes_version: pcr.kubernetes_version.clone(),
        }
    }
}

impl Storable for EntityEvent {
    fn get_id(&self) -> &str {
        &self.id
    }

    fn set_id(&mut self, id: impl Into<String>) {
        self.id = id.into();
    }

    fn type_name() -> &'static str {
        "aws_eks_cluster_runtime_entity_event"
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
                AwsEksClusterRuntimeError::InvalidMissingActionName.to_string(),
            ));
        }
        Ok(())
    }

    fn get_tenant_ids(&self) -> &[String] {
        &self.tenant_ids
    }

    fn add_to_tenant_ids(&mut self, id: impl Into<String>) {
        self.tenant_ids.push(id.into());
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

    pub fn result_topic(&self) -> String {
        format!("{}/result", self.topic_prefix())
    }

    pub fn finalized_topic(&self) -> String {
        format!("{}/finalized", self.topic_prefix())
    }

    fn topic_prefix(&self) -> String {
        format!(
            "{}/{}/{}/{}/{}/{}/{}/{}/{}",
            self.billing_account_id,
            self.organization_id,
            self.workspace_id,
            self.integration_id,
            self.integration_service_id,
            self.entity_id,
            "action",
            self.action_name,
            self.id,
        )
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
