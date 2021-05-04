use serde::{Deserialize, Serialize};
use si_data::{NatsTxn, PgTxn};

use crate::{
    lodash,
    workflow::{WorkflowError, WorkflowResult},
    Edge, EdgeKind, Entity, Resource, WorkflowContext,
};

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SelectionEntryPredecessor {
    pub entity: Entity,
    pub resource: Resource,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SelectionEntry {
    pub entity: Entity,
    pub resource: Resource,
    pub context: Vec<SelectionEntryPredecessor>,
}

impl SelectionEntry {
    pub async fn new(
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        entity_id: impl AsRef<str>,
        system: &Entity,
    ) -> WorkflowResult<Self> {
        let entity_id = entity_id.as_ref();
        let entity: Entity = Entity::for_head(&txn, &entity_id)
            .await
            .map_err(|e| WorkflowError::Entity(e.to_string()))?;
        let system_id = &system.id[..];
        let predecessor_edges =
            Edge::direct_predecessor_edges_by_object_id(&txn, &EdgeKind::Configures, &entity.id)
                .await?;
        let mut context: Vec<SelectionEntryPredecessor> = Vec::new();
        for edge in predecessor_edges {
            let edge_entity = Entity::for_head(&txn, &edge.tail_vertex.object_id)
                .await
                .map_err(|e| WorkflowError::Entity(e.to_string()))?;
            let predecessor_resource = Resource::for_system(
                &txn,
                &nats,
                &edge_entity.id,
                &system_id,
                &entity.si_storable.workspace_id,
            )
            .await?;
            let predecessor = SelectionEntryPredecessor {
                entity: edge_entity,
                resource: predecessor_resource,
            };
            context.push(predecessor);
        }

        let component_edges =
            Edge::direct_predecessor_edges_by_object_id(&txn, &EdgeKind::Component, &entity.id)
                .await?;
        for component_edge in component_edges {
            let component_entity = Entity::for_head(&txn, &component_edge.tail_vertex.object_id)
                .await
                .map_err(|e| WorkflowError::Entity(e.to_string()))?;
            let component_resource = Resource::for_system(
                &txn,
                &nats,
                &component_entity.id,
                &system_id,
                &entity.si_storable.workspace_id,
            )
            .await?;
            let component_predecessor = SelectionEntryPredecessor {
                entity: component_entity.clone(),
                resource: component_resource,
            };
            context.push(component_predecessor);
            let component_edge_properties = match component_entity.properties.get(&system.id) {
                Some(prop) => prop,
                None => continue,
            };
            let implementation_entity_id =
                match lodash::get(&component_edge_properties, &vec!["implementation"])? {
                    Some(entity_id_json) => match entity_id_json.as_str() {
                        Some(entity_id_str) => String::from(entity_id_str),
                        None => {
                            dbg!("no value for implementation");
                            continue;
                        }
                    },
                    None => continue,
                };
            let implementation_edge_entity = Entity::for_head(&txn, &implementation_entity_id)
                .await
                .map_err(|e| WorkflowError::Entity(e.to_string()))?;
            let implementation_successor_resource = Resource::for_system(
                &txn,
                &nats,
                &implementation_entity_id,
                &system_id,
                &entity.si_storable.workspace_id,
            )
            .await?;
            let implementation_successor = SelectionEntryPredecessor {
                entity: implementation_edge_entity.clone(),
                resource: implementation_successor_resource,
            };
            context.push(implementation_successor);

            let crosswire_edges = Edge::direct_successor_edges_by_object_id(
                &txn,
                &EdgeKind::Deployment,
                &component_entity.id,
            )
            .await?;
            for deployment_edge in crosswire_edges {
                let deployment_edge_entity =
                    Entity::for_head(&txn, &deployment_edge.head_vertex.object_id)
                        .await
                        .map_err(|e| WorkflowError::Entity(e.to_string()))?;
                let deployment_successor_resource = Resource::for_system(
                    &txn,
                    &nats,
                    &deployment_edge_entity.id,
                    &system_id,
                    &entity.si_storable.workspace_id,
                )
                .await?;
                let deployment_successor = SelectionEntryPredecessor {
                    entity: deployment_edge_entity.clone(),
                    resource: deployment_successor_resource,
                };
                context.push(deployment_successor);
                let deployment_edge_properties =
                    match deployment_edge_entity.properties.get(&system.id) {
                        Some(prop) => prop,
                        None => continue,
                    };
                let implementation_entity_id =
                    match lodash::get(&deployment_edge_properties, &vec!["implementation"])? {
                        Some(entity_id_json) => match entity_id_json.as_str() {
                            Some(entity_id_str) => String::from(entity_id_str),
                            None => {
                                dbg!("no value for implementation");
                                continue;
                            }
                        },
                        None => continue,
                    };
                let implementation_edge_entity = Entity::for_head(&txn, &implementation_entity_id)
                    .await
                    .map_err(|e| WorkflowError::Entity(e.to_string()))?;
                let implementation_successor_resource = Resource::for_system(
                    &txn,
                    &nats,
                    &implementation_entity_id,
                    &system_id,
                    &entity.si_storable.workspace_id,
                )
                .await?;
                let implementation_successor = SelectionEntryPredecessor {
                    entity: implementation_edge_entity.clone(),
                    resource: implementation_successor_resource,
                };
                context.push(implementation_successor);

                // Crosswire all components of this deployment edge implementation!
                let crosswire_edges = Edge::all_predecessor_edges_by_object_id(
                    &txn,
                    &EdgeKind::Configures,
                    &implementation_entity_id,
                )
                .await?;
                for crosswire_edge in crosswire_edges {
                    dbg!(&crosswire_edge);
                    // Sometimes edges are created but never saved! this can cause
                    // some very strange failures when you try and run things.
                    let crosswire_edge_entity =
                        Entity::for_head(&txn, &crosswire_edge.tail_vertex.object_id)
                            .await
                            .map_err(|e| WorkflowError::Entity(e.to_string()))?;
                    let crosswire_resource = Resource::for_system(
                        &txn,
                        &nats,
                        &crosswire_edge_entity.id,
                        &system_id,
                        &entity.si_storable.workspace_id,
                    )
                    .await?;
                    let crosswire_successor = SelectionEntryPredecessor {
                        entity: crosswire_edge_entity.clone(),
                        resource: crosswire_resource,
                    };
                    context.push(crosswire_successor);
                }
            }
        }

        let resource = Resource::for_system(
            &txn,
            &nats,
            &entity.id,
            &system_id,
            &entity.si_storable.workspace_id,
        )
        .await?;
        Ok(SelectionEntry {
            entity: entity.clone(),
            resource,
            context,
        })
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub enum SelectorDepth {
    Immediate,
    All,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub enum SelectorDirection {
    Input,
    Output,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Selector {
    types: Option<Vec<String>>,
    from_property: Option<Vec<String>>,
    depth: Option<SelectorDepth>,
    edge_kind: Option<EdgeKind>,
    direction: Option<SelectorDirection>,
}

impl Selector {
    pub fn new() -> Self {
        Selector {
            types: None,
            from_property: None,
            depth: None,
            edge_kind: None,
            direction: None,
        }
    }

    // Resolution of a selector:
    // * If from_property is set, then create a selectionEntry from that entity ID from the
    //   context entity.
    // * Otherwise, edgeKind, depth, and direction must be set. Fetch the selectionEntry
    //   for each entity that has a matching edge, optionally filtered by the types
    //   array.
    pub async fn resolve(
        &self,
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        ctx: &WorkflowContext,
    ) -> WorkflowResult<Vec<SelectionEntry>> {
        let mut results = vec![];

        let root_entity = match ctx.entity {
            Some(ref entity) => entity,
            None => return Err(WorkflowError::NoSelectorOrEntity),
        };
        let system = match ctx.system {
            Some(ref system) => system,
            None => return Err(WorkflowError::SystemRequired),
        };

        // If from_property is set, then the selection entry is just the entity from that
        // property on the context entity.
        if let Some(from_property) = &self.from_property {
            let properties = root_entity
                .properties
                .get(&system.id)
                .ok_or(WorkflowError::NoPropertiesForSystem)?;
            let selected_entity_id_json = lodash::get(properties, from_property)?
                .ok_or(WorkflowError::PropertyNotFound(from_property.clone()))?;
            let selected_entity_id = selected_entity_id_json
                .as_str()
                .ok_or(WorkflowError::PropertyNotAString(from_property.clone()))?;
            let selection_entry =
                SelectionEntry::new(&txn, &nats, &selected_entity_id, &system).await?;
            results.push(selection_entry);
        // Otherwise, we take edge_kind, depth and direction, and query for the successors
        // or predecessors, optionally filtered by type.
        } else if self.edge_kind.is_some() {
            let edge_kind = self
                .edge_kind
                .as_ref()
                .ok_or(WorkflowError::EdgeKindMissing)?;
            let depth = self.depth.as_ref().ok_or(WorkflowError::DepthMissing)?;
            let direction = self
                .direction
                .as_ref()
                .ok_or(WorkflowError::DirectionMissing)?;

            let mut edges: Vec<Edge> = match (direction, depth) {
                (&SelectorDirection::Input, &SelectorDepth::Immediate) => {
                    let edges = Edge::direct_predecessor_edges_by_object_id(
                        &txn,
                        &edge_kind,
                        &root_entity.id,
                    )
                    .await?;
                    edges
                }
                (&SelectorDirection::Input, &SelectorDepth::All) => {
                    let edges =
                        Edge::all_predecessor_edges_by_object_id(&txn, &edge_kind, &root_entity.id)
                            .await?;
                    edges
                }
                (&SelectorDirection::Output, &SelectorDepth::Immediate) => {
                    let edges = Edge::direct_successor_edges_by_object_id(
                        &txn,
                        &edge_kind,
                        &root_entity.id,
                    )
                    .await?;
                    edges
                }
                (&SelectorDirection::Output, &SelectorDepth::All) => {
                    let edges =
                        Edge::all_successor_edges_by_object_id(&txn, &edge_kind, &root_entity.id)
                            .await?;
                    edges
                }
            };
            dbg!("resolved selectors pre filter");
            dbg!(&self);
            dbg!(&edges);
            if let Some(ref types) = self.types {
                match direction {
                    &SelectorDirection::Input => {
                        edges = edges
                            .into_iter()
                            .filter(|edge| {
                                if let Some(_) =
                                    types.iter().find(|t| t == &&edge.tail_vertex.object_type)
                                {
                                    return true;
                                } else {
                                    return false;
                                }
                            })
                            .collect();
                    }
                    &SelectorDirection::Output => {
                        edges = edges
                            .into_iter()
                            .filter(|edge| {
                                if let Some(_) =
                                    types.iter().find(|t| t == &&edge.head_vertex.object_type)
                                {
                                    return true;
                                } else {
                                    return false;
                                }
                            })
                            .collect();
                    }
                }
            }
            dbg!("resolved selectors");
            dbg!(&self);
            dbg!(&edges);
            for edge in edges.into_iter() {
                let edge_object_id = match direction {
                    &SelectorDirection::Input => edge.tail_vertex.object_id,
                    &SelectorDirection::Output => edge.head_vertex.object_id,
                };
                let selected_entry =
                    SelectionEntry::new(&txn, &nats, &edge_object_id, &system).await?;
                results.push(selected_entry);
            }
        // Or the current entity is the selected target
        } else {
            let selection_entry =
                SelectionEntry::new(&txn, &nats, &root_entity.id, &system).await?;
            results.push(selection_entry);
        }
        Ok(results)
    }
}
