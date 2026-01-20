//! The Data Access Layer (DAL) for System Initiative.

#![recursion_limit = "256"]
#![warn(
    clippy::panic,
    clippy::panic_in_result_fn,
    clippy::unwrap_in_result,
    clippy::unwrap_used
)]

use rand::Rng;
use serde_with::{
    DeserializeFromStr,
    SerializeDisplay,
};
use strum::{
    Display,
    EnumString,
    VariantNames,
};
use telemetry::prelude::*;
use thiserror::Error;

pub mod action;
pub mod approval_requirement;
pub mod attribute;
pub mod audit_logging;
pub mod authentication_prototype;
pub mod billing_publish;
pub mod builtins;
pub mod cached_module;
pub mod change_set;
pub mod change_status;
pub mod code_view;
pub mod component;
pub mod context;
pub mod dependency_graph;
pub mod diagram;
pub mod entity_kind;
pub mod feature_flags;
pub mod func;
pub mod input_sources;
pub mod jetstream_streams;
pub mod job;
pub mod key_pair;
pub mod label_list;
pub mod layer_db_types;
pub mod management;
pub mod module;
pub mod pkg;
pub mod policy;
pub mod prompt_override;
pub mod prop;
pub mod property_editor;
pub mod qualification;
pub mod resource_metadata;
pub mod schema;
pub mod secret;
pub mod serde_impls;
pub mod slow_rt;
pub mod socket;
pub mod standard_accessors;
pub mod standard_connection;
pub mod status;
pub mod user;
pub mod validation;
pub mod workspace;
pub mod workspace_integrations;
pub mod workspace_snapshot;
pub mod ws_event;

pub use action::ActionPrototypeId;
pub use attribute::{
    attributes::{
        update_attributes,
        update_attributes_without_validation,
    },
    prototype::{
        AttributePrototype,
        AttributePrototypeId,
    },
    value::{
        AttributeValue,
        AttributeValueId,
    },
};
pub use builtins::{
    BuiltinsError,
    BuiltinsResult,
};
pub use change_set::{
    ChangeSet,
    ChangeSetApplyError,
    ChangeSetError,
    ChangeSetId,
    status::ChangeSetStatus,
};
pub use component::{
    Component,
    ComponentError,
    ComponentId,
};
pub use context::{
    AccessBuilder,
    Connections,
    DalContext,
    DalContextBuilder,
    DalLayerDb,
    RequestContext,
    ServicesContext,
    Transactions,
    TransactionsError,
};
pub use func::{
    Func,
    FuncError,
    FuncId,
    backend::{
        FuncBackendKind,
        FuncBackendResponseType,
    },
};
pub use jetstream_streams::{
    JetstreamStreams,
    JetstreamStreamsError,
};
pub use job::processor::{
    JobQueueProcessor,
    NatsProcessor,
};
pub use key_pair::{
    KeyPair,
    KeyPairError,
    KeyPairResult,
    PublicKey,
};
pub use label_list::{
    LabelEntry,
    LabelList,
    LabelListError,
};
pub use prop::{
    Prop,
    PropId,
    PropKind,
};
pub use schema::{
    Schema,
    SchemaError,
    SchemaId,
    SchemaVariant,
    SchemaVariantId,
    variant::{
        SchemaVariantError,
        root_prop::component_type::ComponentType,
    },
};
pub use secret::{
    EncryptedSecret,
    Secret,
    SecretAlgorithm,
    SecretCreatedPayload,
    SecretDefinitionView,
    SecretDefinitionViewError,
    SecretError,
    SecretId,
    SecretResult,
    SecretUpdatedPayload,
    SecretVersion,
    SecretView,
    SecretViewError,
};
pub use si_events::{
    WorkspaceSnapshotAddress,
    content_hash::ContentHash,
    ulid::Ulid,
};
pub use si_id::UserPk;
pub use si_runtime::{
    DedicatedExecutor,
    DedicatedExecutorError,
    DedicatedExecutorInitializeError,
    DedicatedExecutorJoinError,
    compute_executor,
};
pub use socket::{
    SocketArity,
    SocketKind,
    input::{
        InputSocket,
        InputSocketId,
    },
    output::{
        OutputSocket,
        OutputSocketId,
    },
};
pub use standard_connection::{
    HelperError,
    HelperResult,
};
pub use workspace::{
    Workspace,
    WorkspaceError,
    WorkspacePk,
    WorkspaceResult,
};
pub use workspace_snapshot::{
    WorkspaceSnapshot,
    WorkspaceSnapshotError,
    edge_weight::{
        EdgeWeight,
        EdgeWeightKind,
        EdgeWeightKindDiscriminants,
    },
    graph::{
        WorkspaceSnapshotGraph,
        WorkspaceSnapshotGraphVCurrent,
    },
    node_weight::NodeWeightDiscriminants,
};
pub use ws_event::{
    WsEvent,
    WsEventError,
    WsEventResult,
    WsPayload,
};

#[remain::sorted]
#[derive(Error, Debug)]
pub enum InitializationError {
    #[error("failed to initialize sodium oxide")]
    SodiumOxide,
}

pub type InitializationResult<T> = Result<T, InitializationError>;

/// Perform base initializations before using the `dal`.
pub fn init() -> InitializationResult<()> {
    sodiumoxide::init().map_err(|()| InitializationError::SodiumOxide)?;
    Ok(())
}

const NAME_CHARSET: &[u8] = b"0123456789";

pub fn generate_unique_id(length: usize) -> String {
    let mut rng = rand::thread_rng();
    (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..NAME_CHARSET.len());
            NAME_CHARSET[idx] as char
        })
        .collect()
}

pub fn generate_name_from_schema_name(schema_name: impl AsRef<str>) -> String {
    let unique_id = generate_unique_id(4);
    let schema_name = schema_name.as_ref();
    format!("{schema_name} {unique_id}")
}

pub fn generate_name() -> String {
    let unique_id = generate_unique_id(4);
    format!("si-{unique_id}")
}

#[remain::sorted]
#[derive(
    Clone,
    Debug,
    DeserializeFromStr,
    Display,
    EnumString,
    VariantNames,
    Eq,
    PartialEq,
    SerializeDisplay,
)]
#[strum(serialize_all = "camelCase")]
pub enum MigrationMode {
    BackfillFuncRuns,
    BackfillLayerCache,
    GarbageCollectSnapshots,
    Run,
    RunAndQuit,
    Skip,
}

impl Default for MigrationMode {
    fn default() -> Self {
        Self::Run
    }
}

impl MigrationMode {
    #[must_use]
    pub const fn variants() -> &'static [&'static str] {
        <MigrationMode as strum::VariantNames>::VARIANTS
    }

    pub fn is_run(&self) -> bool {
        matches!(self, Self::Run)
    }

    pub fn is_run_and_quit(&self) -> bool {
        matches!(self, Self::RunAndQuit)
    }

    pub fn is_garbage_collect_snapshots(&self) -> bool {
        matches!(self, Self::GarbageCollectSnapshots)
    }

    pub fn is_backfill_layer_cache(&self) -> bool {
        matches!(self, Self::BackfillLayerCache)
    }

    pub fn is_backfill_func_runs(&self) -> bool {
        matches!(self, Self::BackfillFuncRuns)
    }
}

#[cfg(test)]
mod tests {
    use serde::{
        Deserialize,
        Serialize,
    };

    use super::*;

    mod migration_mode {
        use super::*;

        #[test]
        fn display() {
            assert_eq!(
                "garbageCollectSnapshots",
                MigrationMode::GarbageCollectSnapshots.to_string()
            );
            assert_eq!("run", MigrationMode::Run.to_string());
            assert_eq!("runAndQuit", MigrationMode::RunAndQuit.to_string());
            assert_eq!("skip", MigrationMode::Skip.to_string());
        }

        #[test]
        fn from_str() {
            assert_eq!(
                MigrationMode::GarbageCollectSnapshots,
                "garbageCollectSnapshots".parse().expect("failed to parse")
            );
            assert_eq!(MigrationMode::Run, "run".parse().expect("failed to parse"));
            assert_eq!(
                MigrationMode::RunAndQuit,
                "runAndQuit".parse().expect("failed to parse")
            );
            assert_eq!(
                MigrationMode::Skip,
                "skip".parse().expect("failed to parse")
            );
        }

        #[test]
        fn deserialize() {
            #[derive(Deserialize)]
            struct Test {
                mode: MigrationMode,
            }

            let test: Test =
                serde_json::from_str(r#"{"mode":"runAndQuit"}"#).expect("failed to deserialize");
            assert_eq!(MigrationMode::RunAndQuit, test.mode);
        }

        #[test]
        fn serialize() {
            #[derive(Serialize)]
            struct Test {
                mode: MigrationMode,
            }

            let test = serde_json::to_string(&Test {
                mode: MigrationMode::RunAndQuit,
            })
            .expect("failed to serialize");
            assert_eq!(r#"{"mode":"runAndQuit"}"#, test);
        }
    }
}
