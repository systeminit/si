use si_events::workspace_snapshot::EntityKind;

use crate::reference::ReferenceKind;

pub trait MaterializedView {
    fn reference_dependencies() -> &'static [ReferenceKind];

    fn trigger_entity() -> EntityKind;
}
