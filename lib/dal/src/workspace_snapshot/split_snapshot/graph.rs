use petgraph::{
    Direction,
    visit::{
        Control,
        DfsEvent,
    },
};
use si_events::workspace_snapshot::EntityKind;
use si_id::{
    AttributeValueId,
    ComponentId,
    EntityId,
    ulid::Ulid,
};

use crate::{
    ComponentError,
    EdgeWeightKindDiscriminants,
    WorkspaceSnapshotError,
    component::ComponentResult,
    entity_kind::{
        EntityKindError,
        EntityKindResult,
    },
    workspace_snapshot::{
        graph::traits::{
            attribute_value::AttributeValueExt,
            component::ComponentExt,
            entity_kind::EntityKindExt,
        },
        split_snapshot::SplitSnapshotGraphV1,
    },
};

pub mod attribute_value;
pub mod prop;

impl ComponentExt for SplitSnapshotGraphV1 {
    fn root_attribute_value(&self, component_id: ComponentId) -> ComponentResult<AttributeValueId> {
        let mut iter = self
            .outgoing_targets(component_id.into(), EdgeWeightKindDiscriminants::Root)
            .map_err(WorkspaceSnapshotError::from)?;

        match (iter.next(), iter.next()) {
            (Some(av_id), None) => Ok(av_id.into()),
            (Some(_), Some(_)) => Err(WorkspaceSnapshotError::TooManyEdgesOfKind(
                component_id.into(),
                EdgeWeightKindDiscriminants::Root,
            )
            .into()),
            (None, None) => Err(ComponentError::MissingRootProp(component_id)),
            (None, Some(_)) => unreachable!("iterator had none then some"),
        }
    }

    fn external_source_count(&self, component_id: ComponentId) -> ComponentResult<usize> {
        let root_av_id = self.root_attribute_value(component_id)?;

        let mut count = 0;

        petgraph::visit::depth_first_search(self, Some(root_av_id.into()), |event| {
            external_source_count_dfs_event(event, &mut count, self)
        })?;

        Ok(count)
    }
}

impl EntityKindExt for SplitSnapshotGraphV1 {
    fn get_entity_kind_for_id(&self, id: EntityId) -> EntityKindResult<EntityKind> {
        self.node_weight(id.into())
            .ok_or(EntityKindError::NodeNotFound(id))
            .map(Into::into)
    }
}

fn external_source_count_dfs_event(
    event: DfsEvent<Ulid>,
    count: &mut usize,
    graph: &SplitSnapshotGraphV1,
) -> ComponentResult<Control<()>> {
    match event {
        DfsEvent::Discover(av_id, _) => {
            let entity_kind = graph.get_entity_kind_for_id(av_id.into())?;

            // Early return if node is not an attribute value
            if !matches!(entity_kind, EntityKind::AttributeValue) {
                return Ok(petgraph::visit::Control::Prune);
            }

            let Some(prototype_id) = graph.component_prototype_id(av_id.into())? else {
                return Ok(petgraph::visit::Control::Continue);
            };

            let mut subs_count = 0;

            for edge_ref in graph.edges_directed_for_edge_weight_kind(
                prototype_id.into(),
                Direction::Outgoing,
                EdgeWeightKindDiscriminants::PrototypeArgument,
            )? {
                subs_count += graph
                    .edges_directed_for_edge_weight_kind(
                        edge_ref.target(),
                        Direction::Outgoing,
                        EdgeWeightKindDiscriminants::ValueSubscription,
                    )?
                    .count();
            }

            if subs_count > 0 {
                *count += subs_count;
                Ok(petgraph::visit::Control::Prune)
            } else {
                Ok(petgraph::visit::Control::Continue)
            }
        }
        _ => Ok(petgraph::visit::Control::Continue),
    }
}
