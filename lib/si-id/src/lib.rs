//! Provides a centralized location for constructing identifiers for SI.

#![warn(
    bad_style,
    clippy::missing_panics_doc,
    clippy::panic,
    clippy::panic_in_result_fn,
    clippy::unwrap_in_result,
    clippy::unwrap_used,
    dead_code,
    improper_ctypes,
    missing_debug_implementations,
    missing_docs,
    no_mangle_generic_items,
    non_shorthand_field_patterns,
    overflowing_literals,
    path_statements,
    patterns_in_fns_without_body,
    unconditional_recursion,
    unreachable_pub,
    unused,
    unused_allocation,
    unused_comparisons,
    unused_parens,
    while_true
)]

#[macro_use]
pub(crate) mod macros;
pub(crate) mod conversions;
pub mod ulid;

// Please keep these alphabetically sorted!
id!(ActionPrototypeId);
id!(AttributePrototypeArgumentId);
id!(AttributePrototypeId);
id!(AuthenticationPrototypeId);
id!(DeprecatedVectorClockId);
id!(EventSessionId);
id!(FuncArgumentId);
id!(FuncExecutionPk);
id!(GeometryId);
id!(HistoryEventPk);
id!(InputSocketId);
id!(ManagementPrototypeId);
id!(ModuleId);
id!(NaxumApiTypesRequestId);
id!(OutputSocketId);
id!(PropId);
id!(PropertyEditorPropId);
id!(PropertyEditorValueId);
id!(SecretId);
id!(StaticArgumentValueId);
id!(ValidationOutputId);
id!(ViewId);
id!(WorkspaceSnapshotNodeId);

// Please keep these alphabetically sorted!
id_with_pg_types!(ActionId);
id_with_pg_types!(CachedModuleId);
id_with_pg_types!(ChangeSetId);
id_with_pg_types!(ComponentId);
id_with_pg_types!(FuncId);
id_with_pg_types!(FuncRunId);
id_with_pg_types!(SchemaId);
id_with_pg_types!(UserPk);

// Please keep these alphabetically sorted!
id_with_none!(SchemaVariantId);

// Please keep these alphabetically sorted!
id_with_none_and_pg_types!(AttributeValueId);
id_with_none_and_pg_types!(KeyPairPk);
id_with_none_and_pg_types!(WorkspacePk);
id_with_none_and_pg_types!(WorkspaceId);
