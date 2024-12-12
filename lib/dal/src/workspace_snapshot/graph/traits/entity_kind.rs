use anyhow::Result;
use si_events::workspace_snapshot::EntityKind;
use si_id::EntityId;

pub trait EntityKindExt {
    fn get_entity_kind_for_id(&self, id: EntityId) -> Result<EntityKind>;
}
