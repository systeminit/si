pub mod audit_log;
pub mod authentication_method;
pub mod change_batch;
pub mod content_hash;
pub mod encrypted_secret;
pub mod merkle_tree_hash;
pub mod rebase_batch_address;
pub mod split_snapshot_rebase_batch_address;
pub mod workspace_snapshot;
pub mod workspace_snapshot_address;
pub mod xxhash_type;

use rebase_batch_address::RebaseBatchAddress;
use serde::{
    Deserialize,
    Serialize,
};
pub use si_id::{
    CachedModuleId,
    ulid,
};
use split_snapshot_rebase_batch_address::SplitSnapshotRebaseBatchAddress;

mod action;
mod actor;
mod cas;
mod change_set_approval;
mod change_set_status;
mod event_session;
mod func;
mod func_execution;
mod func_run;
mod func_run_log;
pub mod materialized_view;
mod resource_metadata;
mod schema;
mod schema_variant;
mod secret;
mod socket;
mod tenancy;
mod timestamp;
mod vector_clock_id;
mod web_event;

pub use crate::{
    action::{
        ActionId,
        ActionKind,
        ActionPrototypeId,
        ActionResultState,
        ActionState,
    },
    actor::{
        Actor,
        UserPk,
    },
    authentication_method::{
        AuthenticationMethod,
        AuthenticationMethodRole,
    },
    cas::CasValue,
    change_set_approval::ChangeSetApprovalStatus,
    change_set_status::ChangeSetStatus,
    content_hash::ContentHash,
    encrypted_secret::EncryptedSecretKey,
    event_session::EventSessionId,
    func::{
        FuncArgumentId,
        FuncId,
    },
    func_execution::*,
    func_run::{
        AttributePrototypeArgumentId,
        AttributePrototypeId,
        AttributeValueId,
        ComponentId,
        FuncArgumentKind,
        FuncBackendKind,
        FuncBackendResponseType,
        FuncKind,
        FuncRun,
        FuncRunBuilder,
        FuncRunBuilderError,
        FuncRunId,
        FuncRunState,
        FuncRunValue,
        ManagementPrototypeId,
        ViewId,
    },
    func_run_log::{
        FuncRunLog,
        FuncRunLogId,
        OutputLine,
    },
    resource_metadata::{
        ResourceMetadata,
        ResourceStatus,
    },
    schema::SchemaId,
    schema_variant::{
        PropId,
        SchemaVariantId,
    },
    secret::SecretId,
    socket::{
        InputSocketId,
        OutputSocketId,
    },
    tenancy::{
        ChangeSetId,
        Tenancy,
        WorkspacePk,
    },
    timestamp::Timestamp,
    vector_clock_id::{
        VectorClockActorId,
        VectorClockChangeSetId,
        VectorClockId,
    },
    web_event::WebEvent,
    workspace_snapshot_address::WorkspaceSnapshotAddress,
};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Hash, strum::Display)]
pub enum RebaseBatchAddressKind {
    Legacy(RebaseBatchAddress),
    Split(SplitSnapshotRebaseBatchAddress),
}
