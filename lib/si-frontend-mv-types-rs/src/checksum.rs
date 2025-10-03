use std::collections::HashMap;

use chrono::{
    DateTime,
    Utc,
};
use si_events::{
    ActionKind,
    ActionState,
    ChangeSetStatus,
    FuncKind,
    Timestamp,
    workspace_snapshot::{
        Checksum,
        ChecksumHasher,
    },
};
use si_id::{
    ActionId,
    ActionPrototypeId,
    AttributeValueId,
    ChangeSetId,
    ComponentId,
    FuncId,
    FuncRunId,
    InputSocketId,
    ManagementPrototypeId,
    OutputSocketId,
    PropId,
    SchemaId,
    SchemaVariantId,
    SecretId,
    ViewId,
    WorkspaceId,
    WorkspacePk,
    ulid::Ulid,
};

use crate::{
    component::attribute_tree::ValidationStatus,
    definition_checksum::DefinitionChecksum,
    schema_variant::{
        SchemaVariantsByCategory,
        prop_tree::{
            Prop,
            PropKind,
            PropWidgetKind,
        },
    },
};

pub trait FrontendChecksum {
    fn checksum(&self) -> Checksum;
}

#[derive(Debug, Clone)]
pub struct FrontendChecksumInventoryItem {
    ident: &'static str,
    definition_checksum: &'static ::std::sync::LazyLock<Checksum>,
}

impl FrontendChecksumInventoryItem {
    pub const fn new(
        ident: &'static str,
        definition_checksum: &'static ::std::sync::LazyLock<Checksum>,
    ) -> Self {
        FrontendChecksumInventoryItem {
            ident,
            definition_checksum,
        }
    }

    pub fn definition_checksum(&self) -> Checksum {
        **self.definition_checksum
    }

    pub fn ident(&self) -> &'static str {
        self.ident
    }
}
::inventory::collect!(FrontendChecksumInventoryItem);

// TODO(Wendy) - Would be nice to have a default checksum once specialization is enabled in Rust
// impl<T: ToString> FrontendChecksum for T {
//     fn checksum(&self) -> Checksum {
//         FrontendChecksum::checksum(&self.to_string())
//     }
// }

// Would be nice to do this automatically as part of the macros. As an impl for a trait
// seems difficult to work around "conflicting implementations for trait" errors with
// the other trait impls for the more basic types.
impl FrontendChecksum for Ulid {
    fn checksum(&self) -> Checksum {
        FrontendChecksum::checksum(&self.to_string())
    }
}

impl FrontendChecksum for ChangeSetId {
    fn checksum(&self) -> Checksum {
        FrontendChecksum::checksum(&self.to_string())
    }
}

impl FrontendChecksum for WorkspacePk {
    fn checksum(&self) -> Checksum {
        FrontendChecksum::checksum(&self.to_string())
    }
}

impl FrontendChecksum for ChangeSetStatus {
    fn checksum(&self) -> Checksum {
        FrontendChecksum::checksum(&self.to_string())
    }
}

impl FrontendChecksum for Checksum {
    fn checksum(&self) -> Checksum {
        *self
    }
}

impl FrontendChecksum for WorkspaceId {
    fn checksum(&self) -> Checksum {
        FrontendChecksum::checksum(&self.to_string())
    }
}

impl FrontendChecksum for ViewId {
    fn checksum(&self) -> Checksum {
        FrontendChecksum::checksum(&self.to_string())
    }
}

impl FrontendChecksum for SchemaId {
    fn checksum(&self) -> Checksum {
        FrontendChecksum::checksum(&self.to_string())
    }
}

impl FrontendChecksum for ManagementPrototypeId {
    fn checksum(&self) -> Checksum {
        FrontendChecksum::checksum(&self.to_string())
    }
}

impl FrontendChecksum for FuncId {
    fn checksum(&self) -> Checksum {
        FrontendChecksum::checksum(&self.to_string())
    }
}

impl FrontendChecksum for InputSocketId {
    fn checksum(&self) -> Checksum {
        FrontendChecksum::checksum(&self.to_string())
    }
}

impl FrontendChecksum for SecretId {
    fn checksum(&self) -> Checksum {
        FrontendChecksum::checksum(&self.to_string())
    }
}

impl FrontendChecksum for OutputSocketId {
    fn checksum(&self) -> Checksum {
        FrontendChecksum::checksum(&self.to_string())
    }
}

impl FrontendChecksum for PropId {
    fn checksum(&self) -> Checksum {
        FrontendChecksum::checksum(&self.to_string())
    }
}

impl FrontendChecksum for PropKind {
    fn checksum(&self) -> Checksum {
        FrontendChecksum::checksum(&self.to_string())
    }
}

impl FrontendChecksum for Prop {
    fn checksum(&self) -> Checksum {
        let mut hasher = ChecksumHasher::new();
        hasher.update(FrontendChecksum::checksum(&self.id).as_bytes());
        hasher.update(FrontendChecksum::checksum(&self.name).as_bytes());
        hasher.update(FrontendChecksum::checksum(&self.kind).as_bytes());
        hasher.update(FrontendChecksum::checksum(&self.path).as_bytes());
        hasher.update(FrontendChecksum::checksum(&self.hidden).as_bytes());
        hasher.update(FrontendChecksum::checksum(&self.eligible_for_connection).as_bytes());
        hasher.update(FrontendChecksum::checksum(&self.create_only).as_bytes());
        hasher.update(FrontendChecksum::checksum(&self.doc_link).as_bytes());
        hasher.update(FrontendChecksum::checksum(&self.documentation).as_bytes());
        hasher.update(FrontendChecksum::checksum(&self.default_can_be_set_by_socket).as_bytes());
        hasher.update(FrontendChecksum::checksum(&self.is_origin_secret).as_bytes());
        hasher.update(FrontendChecksum::checksum(&self.widget_kind).as_bytes());
        hasher.finalize()
    }
}

impl FrontendChecksum for SchemaVariantId {
    fn checksum(&self) -> Checksum {
        FrontendChecksum::checksum(&self.to_string())
    }
}

impl FrontendChecksum for SchemaVariantsByCategory {
    fn checksum(&self) -> Checksum {
        let mut hasher = ChecksumHasher::new();
        hasher.update(FrontendChecksum::checksum(&self.display_name).as_bytes());
        hasher.update(FrontendChecksum::checksum(&self.schema_variants).as_bytes());
        hasher.finalize()
    }
}

impl FrontendChecksum for Timestamp {
    fn checksum(&self) -> Checksum {
        let mut hasher = ChecksumHasher::new();
        hasher.update(FrontendChecksum::checksum(&self.created_at).as_bytes());
        hasher.update(FrontendChecksum::checksum(&self.updated_at).as_bytes());
        hasher.finalize()
    }
}

// Generic impl for a basic type.
impl FrontendChecksum for String {
    fn checksum(&self) -> Checksum {
        let mut hasher = ChecksumHasher::new();
        hasher.update(self.as_bytes());
        hasher.finalize()
    }
}

impl FrontendChecksum for bool {
    fn checksum(&self) -> Checksum {
        let mut hasher = ChecksumHasher::new();
        hasher.update(if *self { &[1] } else { &[0] });
        hasher.finalize()
    }
}

impl<T> FrontendChecksum for Option<T>
where
    T: FrontendChecksum,
{
    fn checksum(&self) -> Checksum {
        if let Some(inner) = self {
            inner.checksum()
        } else {
            Checksum::default()
        }
    }
}

impl<T> FrontendChecksum for Vec<T>
where
    T: FrontendChecksum,
{
    fn checksum(&self) -> Checksum {
        let mut hasher = ChecksumHasher::new();
        for item in self {
            hasher.update(item.checksum().as_bytes());
        }
        hasher.finalize()
    }
}

impl<K, V> FrontendChecksum for HashMap<K, V>
where
    K: FrontendChecksum + Ord + std::hash::Hash,
    V: FrontendChecksum,
{
    fn checksum(&self) -> Checksum {
        let mut hasher = ChecksumHasher::new();
        let mut keys: Vec<&K> = self.keys().collect();
        keys.sort();
        for key in keys {
            hasher.update(key.checksum().as_bytes());
            hasher.update(
                match self.get(key) {
                    Some(val) => val.checksum(),
                    None => Checksum::default(),
                }
                .as_bytes(),
            );
        }

        hasher.finalize()
    }
}

impl<T1: FrontendChecksum, T2: FrontendChecksum> FrontendChecksum for (T1, T2) {
    fn checksum(&self) -> Checksum {
        let mut hasher = ChecksumHasher::new();
        hasher.update(self.0.checksum().as_bytes());
        hasher.update(self.1.checksum().as_bytes());
        hasher.finalize()
    }
}

impl FrontendChecksum for DateTime<Utc> {
    fn checksum(&self) -> Checksum {
        FrontendChecksum::checksum(&self.to_rfc3339())
    }
}

impl FrontendChecksum for ActionId {
    fn checksum(&self) -> Checksum {
        FrontendChecksum::checksum(&self.to_string())
    }
}

impl FrontendChecksum for ActionPrototypeId {
    fn checksum(&self) -> Checksum {
        FrontendChecksum::checksum(&self.to_string())
    }
}

impl FrontendChecksum for ComponentId {
    fn checksum(&self) -> Checksum {
        FrontendChecksum::checksum(&self.to_string())
    }
}

impl FrontendChecksum for ActionKind {
    fn checksum(&self) -> Checksum {
        FrontendChecksum::checksum(&self.to_string())
    }
}

impl FrontendChecksum for ActionState {
    fn checksum(&self) -> Checksum {
        FrontendChecksum::checksum(&self.to_string())
    }
}

impl FrontendChecksum for FuncKind {
    fn checksum(&self) -> Checksum {
        FrontendChecksum::checksum(&self.to_string())
    }
}

impl FrontendChecksum for FuncRunId {
    fn checksum(&self) -> Checksum {
        FrontendChecksum::checksum(&self.to_string())
    }
}

impl FrontendChecksum for usize {
    fn checksum(&self) -> Checksum {
        FrontendChecksum::checksum(&self.to_string())
    }
}

impl FrontendChecksum for i64 {
    fn checksum(&self) -> Checksum {
        FrontendChecksum::checksum(&self.to_string())
    }
}

impl FrontendChecksum for AttributeValueId {
    fn checksum(&self) -> Checksum {
        FrontendChecksum::checksum(&self.to_string())
    }
}

impl FrontendChecksum for PropWidgetKind {
    fn checksum(&self) -> Checksum {
        FrontendChecksum::checksum(&self.to_string())
    }
}

impl FrontendChecksum for serde_json::Value {
    fn checksum(&self) -> Checksum {
        FrontendChecksum::checksum(&self.to_string())
    }
}

impl FrontendChecksum for ValidationStatus {
    fn checksum(&self) -> Checksum {
        FrontendChecksum::checksum(&self.to_string())
    }
}

impl FrontendChecksum for u64 {
    fn checksum(&self) -> Checksum {
        FrontendChecksum::checksum(&self.to_string())
    }
}

impl FrontendChecksum for Vec<u8> {
    fn checksum(&self) -> Checksum {
        let mut hasher = ChecksumHasher::new();
        hasher.update(self.as_slice());
        hasher.finalize()
    }
}

impl FrontendChecksum for &[u8] {
    fn checksum(&self) -> Checksum {
        let mut hasher = ChecksumHasher::new();
        hasher.update(self);
        hasher.finalize()
    }
}

// ================================================================================================
// DefinitionChecksum implementations for basic types
// ================================================================================================
// DefinitionChecksum implementations for primitive types.
// Each primitive type gets a unique, static checksum that represents its type identity.

impl DefinitionChecksum for String {
    fn definition_checksum() -> Checksum {
        static CHECKSUM: ::std::sync::LazyLock<Checksum> = ::std::sync::LazyLock::new(|| {
            let mut hasher = ChecksumHasher::new();
            hasher.update(b"String");
            hasher.finalize()
        });
        *CHECKSUM
    }
}

impl DefinitionChecksum for bool {
    fn definition_checksum() -> Checksum {
        static CHECKSUM: ::std::sync::LazyLock<Checksum> = ::std::sync::LazyLock::new(|| {
            let mut hasher = ChecksumHasher::new();
            hasher.update(b"bool");
            hasher.finalize()
        });
        *CHECKSUM
    }
}

impl DefinitionChecksum for usize {
    fn definition_checksum() -> Checksum {
        static CHECKSUM: ::std::sync::LazyLock<Checksum> = ::std::sync::LazyLock::new(|| {
            let mut hasher = ChecksumHasher::new();
            hasher.update(b"usize");
            hasher.finalize()
        });
        *CHECKSUM
    }
}

impl DefinitionChecksum for i64 {
    fn definition_checksum() -> Checksum {
        static CHECKSUM: ::std::sync::LazyLock<Checksum> = ::std::sync::LazyLock::new(|| {
            let mut hasher = ChecksumHasher::new();
            hasher.update(b"i64");
            hasher.finalize()
        });
        *CHECKSUM
    }
}

impl DefinitionChecksum for u64 {
    fn definition_checksum() -> Checksum {
        static CHECKSUM: ::std::sync::LazyLock<Checksum> = ::std::sync::LazyLock::new(|| {
            let mut hasher = ChecksumHasher::new();
            hasher.update(b"u64");
            hasher.finalize()
        });
        *CHECKSUM
    }
}

impl DefinitionChecksum for Vec<u8> {
    fn definition_checksum() -> Checksum {
        static CHECKSUM: ::std::sync::LazyLock<Checksum> = ::std::sync::LazyLock::new(|| {
            let mut hasher = ChecksumHasher::new();
            hasher.update(b"Vec<u8>");
            hasher.finalize()
        });
        *CHECKSUM
    }
}

// Generic collection implementations
impl<T> DefinitionChecksum for Option<T>
where
    T: DefinitionChecksum,
{
    fn definition_checksum() -> Checksum {
        let mut hasher = ChecksumHasher::new();
        hasher.update(b"Option<");
        hasher.update(T::definition_checksum().as_bytes());
        hasher.update(b">");
        hasher.finalize()
    }
}

impl<T> DefinitionChecksum for Vec<T>
where
    T: DefinitionChecksum,
{
    fn definition_checksum() -> Checksum {
        let mut hasher = ChecksumHasher::new();
        hasher.update(b"Vec<");
        hasher.update(T::definition_checksum().as_bytes());
        hasher.update(b">");
        hasher.finalize()
    }
}

impl<K, V> DefinitionChecksum for HashMap<K, V>
where
    K: DefinitionChecksum,
    V: DefinitionChecksum,
{
    fn definition_checksum() -> Checksum {
        let mut hasher = ChecksumHasher::new();
        hasher.update(b"HashMap<");
        hasher.update(K::definition_checksum().as_bytes());
        hasher.update(b", ");
        hasher.update(V::definition_checksum().as_bytes());
        hasher.update(b">");
        hasher.finalize()
    }
}

impl<T1, T2> DefinitionChecksum for (T1, T2)
where
    T1: DefinitionChecksum,
    T2: DefinitionChecksum,
{
    fn definition_checksum() -> Checksum {
        let mut hasher = ChecksumHasher::new();
        hasher.update(b"(");
        hasher.update(T1::definition_checksum().as_bytes());
        hasher.update(b", ");
        hasher.update(T2::definition_checksum().as_bytes());
        hasher.update(b")");
        hasher.finalize()
    }
}

// ================================================================================================
// DefinitionChecksum implementations for domain types
// ================================================================================================

impl DefinitionChecksum for serde_json::Value {
    fn definition_checksum() -> Checksum {
        static CHECKSUM: ::std::sync::LazyLock<Checksum> = ::std::sync::LazyLock::new(|| {
            let mut hasher = ChecksumHasher::new();
            hasher.update(b"serde_json::Value");
            hasher.finalize()
        });
        *CHECKSUM
    }
}

impl DefinitionChecksum for DateTime<Utc> {
    fn definition_checksum() -> Checksum {
        static CHECKSUM: ::std::sync::LazyLock<Checksum> = ::std::sync::LazyLock::new(|| {
            let mut hasher = ChecksumHasher::new();
            hasher.update(b"DateTime<Utc>");
            hasher.finalize()
        });
        *CHECKSUM
    }
}

impl DefinitionChecksum for Timestamp {
    fn definition_checksum() -> Checksum {
        static CHECKSUM: ::std::sync::LazyLock<Checksum> = ::std::sync::LazyLock::new(|| {
            let mut hasher = ChecksumHasher::new();
            hasher.update(b"Timestamp");
            hasher.finalize()
        });
        *CHECKSUM
    }
}

impl DefinitionChecksum for Checksum {
    fn definition_checksum() -> Checksum {
        static CHECKSUM: ::std::sync::LazyLock<Checksum> = ::std::sync::LazyLock::new(|| {
            let mut hasher = ChecksumHasher::new();
            hasher.update(b"Checksum");
            hasher.finalize()
        });
        *CHECKSUM
    }
}

// ================================================================================================
// DefinitionChecksum implementations for ID types - each gets unique checksum
// ================================================================================================

impl DefinitionChecksum for Ulid {
    fn definition_checksum() -> Checksum {
        static CHECKSUM: ::std::sync::LazyLock<Checksum> = ::std::sync::LazyLock::new(|| {
            let mut hasher = ChecksumHasher::new();
            hasher.update(b"Ulid");
            hasher.finalize()
        });
        *CHECKSUM
    }
}

impl DefinitionChecksum for ChangeSetId {
    fn definition_checksum() -> Checksum {
        static CHECKSUM: ::std::sync::LazyLock<Checksum> = ::std::sync::LazyLock::new(|| {
            let mut hasher = ChecksumHasher::new();
            hasher.update(b"ChangeSetId");
            hasher.finalize()
        });
        *CHECKSUM
    }
}

impl DefinitionChecksum for WorkspacePk {
    fn definition_checksum() -> Checksum {
        static CHECKSUM: ::std::sync::LazyLock<Checksum> = ::std::sync::LazyLock::new(|| {
            let mut hasher = ChecksumHasher::new();
            hasher.update(b"WorkspacePk");
            hasher.finalize()
        });
        *CHECKSUM
    }
}

impl DefinitionChecksum for WorkspaceId {
    fn definition_checksum() -> Checksum {
        static CHECKSUM: ::std::sync::LazyLock<Checksum> = ::std::sync::LazyLock::new(|| {
            let mut hasher = ChecksumHasher::new();
            hasher.update(b"WorkspaceId");
            hasher.finalize()
        });
        *CHECKSUM
    }
}

impl DefinitionChecksum for ViewId {
    fn definition_checksum() -> Checksum {
        static CHECKSUM: ::std::sync::LazyLock<Checksum> = ::std::sync::LazyLock::new(|| {
            let mut hasher = ChecksumHasher::new();
            hasher.update(b"ViewId");
            hasher.finalize()
        });
        *CHECKSUM
    }
}

impl DefinitionChecksum for SchemaId {
    fn definition_checksum() -> Checksum {
        static CHECKSUM: ::std::sync::LazyLock<Checksum> = ::std::sync::LazyLock::new(|| {
            let mut hasher = ChecksumHasher::new();
            hasher.update(b"SchemaId");
            hasher.finalize()
        });
        *CHECKSUM
    }
}

impl DefinitionChecksum for SchemaVariantId {
    fn definition_checksum() -> Checksum {
        static CHECKSUM: ::std::sync::LazyLock<Checksum> = ::std::sync::LazyLock::new(|| {
            let mut hasher = ChecksumHasher::new();
            hasher.update(b"SchemaVariantId");
            hasher.finalize()
        });
        *CHECKSUM
    }
}

impl DefinitionChecksum for ManagementPrototypeId {
    fn definition_checksum() -> Checksum {
        static CHECKSUM: ::std::sync::LazyLock<Checksum> = ::std::sync::LazyLock::new(|| {
            let mut hasher = ChecksumHasher::new();
            hasher.update(b"ManagementPrototypeId");
            hasher.finalize()
        });
        *CHECKSUM
    }
}

impl DefinitionChecksum for FuncId {
    fn definition_checksum() -> Checksum {
        static CHECKSUM: ::std::sync::LazyLock<Checksum> = ::std::sync::LazyLock::new(|| {
            let mut hasher = ChecksumHasher::new();
            hasher.update(b"FuncId");
            hasher.finalize()
        });
        *CHECKSUM
    }
}

impl DefinitionChecksum for FuncRunId {
    fn definition_checksum() -> Checksum {
        static CHECKSUM: ::std::sync::LazyLock<Checksum> = ::std::sync::LazyLock::new(|| {
            let mut hasher = ChecksumHasher::new();
            hasher.update(b"FuncRunId");
            hasher.finalize()
        });
        *CHECKSUM
    }
}

impl DefinitionChecksum for InputSocketId {
    fn definition_checksum() -> Checksum {
        static CHECKSUM: ::std::sync::LazyLock<Checksum> = ::std::sync::LazyLock::new(|| {
            let mut hasher = ChecksumHasher::new();
            hasher.update(b"InputSocketId");
            hasher.finalize()
        });
        *CHECKSUM
    }
}

impl DefinitionChecksum for OutputSocketId {
    fn definition_checksum() -> Checksum {
        static CHECKSUM: ::std::sync::LazyLock<Checksum> = ::std::sync::LazyLock::new(|| {
            let mut hasher = ChecksumHasher::new();
            hasher.update(b"OutputSocketId");
            hasher.finalize()
        });
        *CHECKSUM
    }
}

impl DefinitionChecksum for SecretId {
    fn definition_checksum() -> Checksum {
        static CHECKSUM: ::std::sync::LazyLock<Checksum> = ::std::sync::LazyLock::new(|| {
            let mut hasher = ChecksumHasher::new();
            hasher.update(b"SecretId");
            hasher.finalize()
        });
        *CHECKSUM
    }
}

impl DefinitionChecksum for PropId {
    fn definition_checksum() -> Checksum {
        static CHECKSUM: ::std::sync::LazyLock<Checksum> = ::std::sync::LazyLock::new(|| {
            let mut hasher = ChecksumHasher::new();
            hasher.update(b"PropId");
            hasher.finalize()
        });
        *CHECKSUM
    }
}

impl DefinitionChecksum for ComponentId {
    fn definition_checksum() -> Checksum {
        static CHECKSUM: ::std::sync::LazyLock<Checksum> = ::std::sync::LazyLock::new(|| {
            let mut hasher = ChecksumHasher::new();
            hasher.update(b"ComponentId");
            hasher.finalize()
        });
        *CHECKSUM
    }
}

impl DefinitionChecksum for ActionId {
    fn definition_checksum() -> Checksum {
        static CHECKSUM: ::std::sync::LazyLock<Checksum> = ::std::sync::LazyLock::new(|| {
            let mut hasher = ChecksumHasher::new();
            hasher.update(b"ActionId");
            hasher.finalize()
        });
        *CHECKSUM
    }
}

impl DefinitionChecksum for ActionPrototypeId {
    fn definition_checksum() -> Checksum {
        static CHECKSUM: ::std::sync::LazyLock<Checksum> = ::std::sync::LazyLock::new(|| {
            let mut hasher = ChecksumHasher::new();
            hasher.update(b"ActionPrototypeId");
            hasher.finalize()
        });
        *CHECKSUM
    }
}

impl DefinitionChecksum for AttributeValueId {
    fn definition_checksum() -> Checksum {
        static CHECKSUM: ::std::sync::LazyLock<Checksum> = ::std::sync::LazyLock::new(|| {
            let mut hasher = ChecksumHasher::new();
            hasher.update(b"AttributeValueId");
            hasher.finalize()
        });
        *CHECKSUM
    }
}

// ================================================================================================
// DefinitionChecksum implementations for enum types
// ================================================================================================

impl DefinitionChecksum for ChangeSetStatus {
    fn definition_checksum() -> Checksum {
        static CHECKSUM: ::std::sync::LazyLock<Checksum> = ::std::sync::LazyLock::new(|| {
            let mut hasher = ChecksumHasher::new();
            hasher.update(b"ChangeSetStatus");
            hasher.finalize()
        });
        *CHECKSUM
    }
}

impl DefinitionChecksum for ActionKind {
    fn definition_checksum() -> Checksum {
        static CHECKSUM: ::std::sync::LazyLock<Checksum> = ::std::sync::LazyLock::new(|| {
            let mut hasher = ChecksumHasher::new();
            hasher.update(b"ActionKind");
            hasher.finalize()
        });
        *CHECKSUM
    }
}

impl DefinitionChecksum for ActionState {
    fn definition_checksum() -> Checksum {
        static CHECKSUM: ::std::sync::LazyLock<Checksum> = ::std::sync::LazyLock::new(|| {
            let mut hasher = ChecksumHasher::new();
            hasher.update(b"ActionState");
            hasher.finalize()
        });
        *CHECKSUM
    }
}

impl DefinitionChecksum for FuncKind {
    fn definition_checksum() -> Checksum {
        static CHECKSUM: ::std::sync::LazyLock<Checksum> = ::std::sync::LazyLock::new(|| {
            let mut hasher = ChecksumHasher::new();
            hasher.update(b"FuncKind");
            hasher.finalize()
        });
        *CHECKSUM
    }
}
