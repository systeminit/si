use serde::{Deserialize, Serialize};
use si_data::{NatsTxn, PgTxn};

use crate::{
    lodash,
    secret::DecryptedSecret,
    workflow::{WorkflowError, WorkflowResult},
    Edge, EdgeKind, Entity, Resource, WorkflowContext,
};

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SelectionEntryPredecessor {
    pub entity: Entity,
    pub resource: Resource,
    pub secret: Option<DecryptedSecret>,
}

impl SelectionEntryPredecessor {
    async fn new(
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        entity_id: impl AsRef<str>,
        system_id: impl AsRef<str>,
        workspace_id: impl AsRef<str>,
    ) -> WorkflowResult<Self> {
        let entity_id = entity_id.as_ref();
        let system_id = system_id.as_ref();
        let workspace_id = workspace_id.as_ref();
        let entity = Entity::for_head(&txn, &entity_id)
            .await
            .map_err(|e| WorkflowError::Entity(e.to_string()))?;
        let resource =
            Resource::for_system(&txn, &nats, &entity_id, &system_id, &workspace_id).await?;
        let secret = entity
            .decrypt_secret_properties(&txn, &system_id)
            .await
            .map_err(|e| WorkflowError::Entity(e.to_string()))?;
        Ok(SelectionEntryPredecessor {
            entity,
            resource,
            secret,
        })
    }

    async fn new_from_entity(
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        entity: &Entity,
        system_id: impl AsRef<str>,
        workspace_id: impl AsRef<str>,
    ) -> WorkflowResult<Self> {
        let system_id = system_id.as_ref();
        let workspace_id = workspace_id.as_ref();
        let entity = entity.clone();
        let resource =
            Resource::for_system(&txn, &nats, &entity.id, &system_id, &workspace_id).await?;
        let secret = entity
            .decrypt_secret_properties(&txn, &system_id)
            .await
            .map_err(|e| WorkflowError::Entity(e.to_string()))?;
        Ok(SelectionEntryPredecessor {
            entity,
            resource,
            secret,
        })
    }
}

struct EntityContext {
    pub context: Vec<SelectionEntryPredecessor>,
}

impl EntityContext {
    fn new() -> Self {
        EntityContext { context: vec![] }
    }

    fn push(&mut self, entry: SelectionEntryPredecessor) {
        if !self.context.iter().any(|p| p.entity.id == entry.entity.id) {
            self.context.push(entry);
        }
    }
}

impl From<EntityContext> for Vec<SelectionEntryPredecessor> {
    fn from(context: EntityContext) -> Vec<SelectionEntryPredecessor> {
        context.context
    }
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
        // Add all your direct configure predecessor edges
        let entity_id = entity_id.as_ref();
        let entity: Entity = Entity::for_head(&txn, &entity_id)
            .await
            .map_err(|e| WorkflowError::Entity(e.to_string()))?;
        let workspace_id = &entity.si_storable.workspace_id[..];
        let system_id = &system.id[..];
        let predecessor_edges =
            Edge::direct_predecessor_edges_by_object_id(&txn, &EdgeKind::Configures, &entity.id)
                .await?;
        let mut context = EntityContext::new();
        for edge in predecessor_edges {
            let predecessor = match SelectionEntryPredecessor::new(
                &txn,
                &nats,
                &edge.tail_vertex.object_id,
                &system_id,
                &workspace_id,
            )
            .await
            {
                Ok(p) => p,
                Err(_e) => {
                    // This is not the correct way to handle this! we should check specifics.
                    continue;
                }
            };
            context.push(predecessor);
        }

        // Look up the component edge for the current entity, in order to find our
        // conceptual node.
        let concept_edges =
            Edge::direct_predecessor_edges_by_object_id(&txn, &EdgeKind::Component, &entity.id)
                .await?;
        for concept_edge in concept_edges {
            let concept_deployment_edge_entity =
                match Entity::for_head(&txn, &concept_edge.tail_vertex.object_id).await {
                    Ok(p) => p,
                    Err(_e) => {
                        // This is not the correct way to handle this! we should check specifics.
                        continue;
                    }
                };
            let concept_entity_context = SelectionEntryPredecessor::new_from_entity(
                &txn,
                &nats,
                &concept_deployment_edge_entity,
                &system_id,
                &workspace_id,
            )
            .await?;
            context.push(concept_entity_context);

            // This is the selected implementation entity id of the concept entity.
            let implementation_entity_id = match concept_deployment_edge_entity
                .get_property_as_string(&system_id, &vec!["implementation"])
                .map_err(|e| WorkflowError::Entity(e.to_string()))?
            {
                Some(id) => id,
                None => continue,
            };
            let implementation_entity_context = match SelectionEntryPredecessor::new(
                &txn,
                &nats,
                &implementation_entity_id,
                &system_id,
                &workspace_id,
            )
            .await
            {
                Ok(p) => p,
                Err(_e) => {
                    // This is not the correct way to handle this! we should check specifics.
                    continue;
                }
            };
            context.push(implementation_entity_context);

            // Given our concept entity, find all the deployment edges in its graph.
            let concept_deployment_edges = Edge::all_successor_edges_by_object_id(
                &txn,
                &EdgeKind::Deployment,
                &concept_deployment_edge_entity.id,
            )
            .await?;
            for concept_deployment_edge in concept_deployment_edges {
                let concept_deployment_edge_entity =
                    match Entity::for_head(&txn, &concept_deployment_edge.head_vertex.object_id)
                        .await
                    {
                        Ok(p) => p,
                        Err(_e) => {
                            // This is not the correct way to handle this! we should check specifics.
                            continue;
                        }
                    };

                // These are the other conceptual entities with deployment edges to our
                // primary conceptual edge.
                let concept_deployment_edge_context = SelectionEntryPredecessor::new_from_entity(
                    &txn,
                    &nats,
                    &concept_deployment_edge_entity,
                    &system_id,
                    &workspace_id,
                )
                .await?;
                context.push(concept_deployment_edge_context);

                // Whatever the implementation node is for this kubernetes cluster
                let implementation_entity_id = match concept_deployment_edge_entity
                    .get_property_as_string(&system_id, &vec!["implementation"])
                    .map_err(|e| WorkflowError::Entity(e.to_string()))?
                {
                    Some(id) => id,
                    None => continue,
                };
                let implementation_successor = match SelectionEntryPredecessor::new(
                    &txn,
                    &nats,
                    &implementation_entity_id,
                    &system_id,
                    &workspace_id,
                )
                .await
                {
                    Ok(p) => p,
                    Err(_e) => {
                        // This is not the correct way to handle this! we should check specifics.
                        continue;
                    }
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
                    let crosswire_successor = match SelectionEntryPredecessor::new(
                        &txn,
                        &nats,
                        &crosswire_edge.tail_vertex.object_id,
                        &system_id,
                        &workspace_id,
                    )
                    .await
                    {
                        Ok(p) => p,
                        Err(_e) => {
                            // This is not the correct way to handle this! we should check specifics.
                            continue;
                        }
                    };

                    // Sometimes edges are created but never saved! this can cause
                    // some very strange failures when you try and run things.
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
            context: context.into(),
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
