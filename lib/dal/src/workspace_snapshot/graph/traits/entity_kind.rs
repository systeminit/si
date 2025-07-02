use si_events::workspace_snapshot::EntityKind;
use si_id::EntityId;

use crate::entity_kind::EntityKindResult;

pub trait EntityKindExt {
    fn get_entity_kind_for_id(&self, id: EntityId) -> EntityKindResult<EntityKind>;
}
