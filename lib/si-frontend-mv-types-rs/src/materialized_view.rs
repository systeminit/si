use si_events::{
    materialized_view::BuildPriority,
    workspace_snapshot::{
        Change,
        Checksum,
        ChecksumHasher,
        EntityKind,
    },
};

use crate::reference::ReferenceKind;

pub trait MaterializedView {
    fn kind() -> ReferenceKind;
    fn trigger_entity() -> EntityKind;
    fn definition_checksum() -> Checksum;
    fn build_priority() -> BuildPriority;
}

#[derive(Debug, Clone)]
pub struct MaterializedViewInventoryItem {
    kind: ReferenceKind,
    trigger_entity: EntityKind,
    build_priority: BuildPriority,
    definition_checksum: &'static ::std::sync::LazyLock<Checksum>,
}

impl MaterializedViewInventoryItem {
    pub const fn new(
        kind: ReferenceKind,
        trigger_entity: EntityKind,
        build_priority: BuildPriority,
        definition_checksum: &'static ::std::sync::LazyLock<Checksum>,
    ) -> Self {
        MaterializedViewInventoryItem {
            kind,
            trigger_entity,
            build_priority,
            definition_checksum,
        }
    }

    pub fn kind(&self) -> ReferenceKind {
        self.kind
    }

    pub fn trigger_entity(&self) -> EntityKind {
        self.trigger_entity
    }

    pub fn build_priority(&self) -> BuildPriority {
        self.build_priority
    }

    pub fn definition_checksum(&self) -> Checksum {
        **self.definition_checksum
    }

    pub fn should_build_for_change(&self, change: Change) -> bool {
        change.entity_kind == self.trigger_entity
    }
}

static MATERIALIZED_VIEW_DEFINITIONS_CHECKSUM: ::std::sync::LazyLock<Checksum> =
    ::std::sync::LazyLock::new(|| {
        let mut mv_items: Vec<_> = ::inventory::iter::<MaterializedViewInventoryItem>().collect();
        mv_items.sort_by_key(|item| item.kind());

        let mut hasher = ChecksumHasher::new();
        for mv in mv_items {
            hasher.update(mv.definition_checksum().as_bytes());
        }

        hasher.finalize()
    });

pub fn materialized_view_definitions_checksum() -> Checksum {
    *MATERIALIZED_VIEW_DEFINITIONS_CHECKSUM
}

::inventory::collect!(MaterializedViewInventoryItem);
