use crate::protobuf::create_entity_request::TagRequest;
use crate::protobuf::{
    Constraints, CreateEntityRequest, ListEntitiesReply, ListEntitiesRequest,
    ListEntityEventsReply, ListEntityEventsRequest, PickComponentReply, PickComponentRequest,
};
use si_account::Workspace;
use si_cea::entity::prelude::*;
use std::convert::TryFrom;
use std::str::FromStr;

pub use crate::protobuf::entity::{Bool, ClusterStatus, NodegroupStatus, State, Tag};
pub use crate::protobuf::{Entity, EntityEvent};

const DEFAULT_NODEGROUP_DESIRED_SIZE: u32 = 2;
const DEFAULT_NODEGROUP_DISK_SIZE: u32 = 20;
const DEFAULT_NODEGROUP_INSTANCE_TYPE: &str = "t3.medium";
const DEFAULT_NODEGROUP_MAX_SIZE: u32 = 2;
const DEFAULT_NODEGROUP_MIN_SIZE: u32 = 2;
const DEFAULT_ENDPOINT_PRIVATE_ACCESS: Bool = Bool::False;
const DEFAULT_ENDPOINT_PUBLIC_ACCESS: Bool = Bool::True;

gen_entity!(
    type_name: "aws_eks_cluster_runtime_entity",
    order_by_fields: [
        "kubernetesVersion",
        "cloudwatchLogs",
        "clusterName",
        "clusterStatus",
        "endpoint",
        "certificateAuthority",
        "endpointPrivateAccess",
        "endpointPublicAccess",
        "nodegroupName",
        "nodegroupStatus",
        "nodegroupSshKeyId",
        "nodegroupInstanceType",
        "nodegroupDiskSize",
        "nodegroupDesiredSize",
        "nodegroupMinSize",
        "nodegroupMaxSize",
        "state"
    ],
    validate_fn: |self| {
        if self.display_name == "" {
            return Err(CeaError::ValidationError("missing display name".to_string()));
        }
        if self.name == "" {
            return Err(CeaError::ValidationError("missing name".to_string()));
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
    ) -> CeaResult<Self> {
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

        db.validate_and_insert_as_new(&mut e).await?;

        Ok(e)
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

gen_entity_event!(
    type_name: "aws_eks_cluster_runtime_entity_event",
    action_names: [
        "create",
        "sync",
        "add_nodegroup"
    ]
);

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

#[derive(thiserror::Error, Debug)]
#[error("invalid ClusterStatus value: {0}")]
pub struct InvalidClusterStatusError(String);

impl FromStr for ClusterStatus {
    type Err = InvalidClusterStatusError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let r = match s {
            "CREATING" => Self::Creating,
            "ACTIVE" => Self::Active,
            "DELETING" => Self::Deleting,
            "FAILED" => Self::Failed,
            "UPDATING" => Self::Updating,
            invalid => return Err(InvalidClusterStatusError(invalid.to_string())),
        };

        Ok(r)
    }
}

#[derive(thiserror::Error, Debug)]
#[error("invalid NodegroupStatus value: {0}")]
pub struct InvalidNodegroupStatusError(String);

impl FromStr for NodegroupStatus {
    type Err = InvalidNodegroupStatusError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let r = match s {
            "CREATING" => Self::NodegroupCreating,
            "ACTIVE" => Self::NodegroupActive,
            "UPDATING" => Self::NodegroupUpdating,
            "DELETING" => Self::NodegroupDeleting,
            "CREATE_FAILED" => Self::NodegroupCreateFailed,
            "DELETE_FAILED" => Self::NodegroupDeleteFailed,
            "DEGRADED" => Self::NodegroupDegraded,
            invalid => return Err(InvalidNodegroupStatusError(invalid.to_string())),
        };

        Ok(r)
    }
}
