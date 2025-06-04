use si_events::workspace_snapshot::{
    Checksum,
    ChecksumHasher,
    EntityKind,
};

use crate::{
    checksum::FrontendChecksumInventoryItem,
    reference::ReferenceKind,
};

pub trait MaterializedView {
    fn kind() -> ReferenceKind;
    fn reference_dependencies() -> &'static [ReferenceKind];
    fn trigger_entity() -> EntityKind;
    fn definition_checksum() -> Checksum;
}

#[derive(Debug, Clone)]
pub struct MaterializedViewInventoryItem {
    kind: ReferenceKind,
    reference_dependencies: &'static [ReferenceKind],
    trigger_entity: EntityKind,
    definition_checksum: &'static ::std::sync::LazyLock<Checksum>,
}

impl MaterializedViewInventoryItem {
    pub const fn new(
        kind: ReferenceKind,
        reference_dependencies: &'static [ReferenceKind],
        trigger_entity: EntityKind,
        definition_checksum: &'static ::std::sync::LazyLock<Checksum>,
    ) -> Self {
        MaterializedViewInventoryItem {
            kind,
            reference_dependencies,
            trigger_entity,
            definition_checksum,
        }
    }

    pub fn kind(&self) -> ReferenceKind {
        self.kind
    }

    pub fn reference_dependencies(&self) -> &'static [ReferenceKind] {
        self.reference_dependencies
    }

    pub fn trigger_entity(&self) -> EntityKind {
        self.trigger_entity
    }
    pub fn definition_checksum(&self) -> Checksum {
        **self.definition_checksum
    }
}

static MATERIALIZED_VIEW_DEFINITIONS_CHECKSUM: ::std::sync::LazyLock<Checksum> =
    ::std::sync::LazyLock::new(|| {
        let mut mv_items: Vec<_> = ::inventory::iter::<MaterializedViewInventoryItem>().collect();
        mv_items.sort_by_key(|item| item.kind());
        let mut frontend_checksum_items: Vec<_> =
            ::inventory::iter::<FrontendChecksumInventoryItem>().collect();
        frontend_checksum_items.sort_by_key(|item| item.ident());

        let mut hasher = ChecksumHasher::new();
        for mv in mv_items {
            hasher.update(mv.definition_checksum().as_bytes());
        }
        for frontend_checksum_item in frontend_checksum_items {
            hasher.update(frontend_checksum_item.definition_checksum().as_bytes());
        }

        hasher.finalize()
    });

pub fn materialized_view_definitions_checksum() -> Checksum {
    *MATERIALIZED_VIEW_DEFINITIONS_CHECKSUM
}

::inventory::collect!(MaterializedViewInventoryItem);
