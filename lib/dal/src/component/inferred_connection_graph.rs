//! This module encapsulates the logic for finding the inferred incoming and outgoing connections.
//!
//! When determining what sockets can infer their connections via Frames, you must take into account:
//! the [`ComponentType`], and the [`SocketArity`], in addition to where a [`Component`] exists
//! relative to its parents, children, siblings, and all components in the 'tree'.
//!
//! A [`InferredConnectionGraph`] can be assembled by providing one or many [`ComponentId`]s, building
//! one or many trees that where the [`Component`] exists, then interrogating each [`Component`]'s [`InputSocket`]s
//! to find which [`OutputSocket`]s they can be connected to.
//!
//! As [`OutputSocket`]s do not care which [`InputSocket`]s are pulling data from them, we
//! build this mapping only from the perspective of the [`InputSocket`] and use that mapping to hydrate both
//! the Incoming and Outgoing Inferred Connections for a given [`ComponentId`]

use std::collections::{
    BTreeSet,
    HashMap,
    HashSet,
};

use petgraph::{
    prelude::*,
    visit::{
        Control,
        DfsEvent,
    },
};
use serde::{
    Deserialize,
    Serialize,
};
use telemetry::prelude::*;
use thiserror::Error;

use crate::{
    Component,
    ComponentError,
    ComponentId,
    ComponentType,
    DalContext,
    InputSocket,
    InputSocketId,
    OutputSocket,
    OutputSocketId,
    SocketArity,
    WorkspaceSnapshotError,
    socket::{
        connection_annotation::ConnectionAnnotation,
        input::InputSocketError,
        output::OutputSocketError,
    },
};

#[remain::sorted]
#[derive(Debug, Error)]
pub enum InferredConnectionGraphError {
    #[error("Component error: {0}")]
    Component(#[from] Box<ComponentError>),
    #[error("InputSocket error: {0}")]
    InputSocket(#[from] InputSocketError),
    #[error("Missing graph node")]
    MissingGraphNode,
    #[error("Orphaned Component")]
    OrphanedComponent(ComponentId),
    #[error("OutputSocket error: {0}")]
    OutputSocket(#[from] OutputSocketError),
    #[error("Unable to compute costs for inferred connections")]
    UnableToComputeCost,
    #[error("Unsupported Component type {0} for Component {1}")]
    UnsupportedComponentType(ComponentType, ComponentId),
    #[error("WorkspaceSnapshot error: {0}")]
    WorkspaceSnapshot(#[from] WorkspaceSnapshotError),
}

pub type InferredConnectionGraphResult<T> = Result<T, InferredConnectionGraphError>;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct InferredConnectionGraph {
    down_component_graph: StableDiGraph<InferredConnectionGraphNodeWeight, ()>,
    up_component_graph: StableDiGraph<InferredConnectionGraphNodeWeight, ()>,
    index_by_component_id: HashMap<ComponentId, NodeIndex>,

    #[serde(skip)]
    inferred_connections_by_component_and_input_socket:
        HashMap<ComponentId, HashMap<InputSocketId, Vec<InferredConnection>>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct InferredConnectionGraphNodeWeight {
    component: Component,
    component_type: ComponentType,
    input_sockets: Vec<InputSocket>,
    output_sockets: Vec<OutputSocket>,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct InferredConnection {
    pub source_component_id: ComponentId,
    pub output_socket_id: OutputSocketId,
    pub destination_component_id: ComponentId,
    pub input_socket_id: InputSocketId,
}

#[derive(Debug, Clone, Copy)]
struct PotentialInferredConnectionMatch {
    component_id: ComponentId,
    output_socket_id: OutputSocketId,
}

impl InferredConnectionGraph {
    #[instrument(
        name = "component.inferred_connection_graph.new",
        level = "debug",
        skip(ctx)
    )]
    pub async fn new(ctx: &DalContext) -> InferredConnectionGraphResult<Self> {
        let mut down_component_graph = StableDiGraph::new();
        let mut index_by_component_id = HashMap::new();

        for component in Component::list(ctx).await.map_err(Box::new)? {
            let component_id = component.id();
            let component_type = match component.get_type(ctx).await {
                Ok(comp_type) => comp_type,
                Err(e) => {
                    // New components are frequently incompletely set up. If we can't get the type,
                    // then it's pretty much guaranteed to not be a Frame (yet), or the child of a
                    // Frame (yet).
                    debug!("{}", e);
                    continue;
                }
            };
            let schema_variant_id = ctx
                .workspace_snapshot()?
                .schema_variant_id_for_component_id(component_id)
                .await
                .map_err(Box::new)?;
            let input_sockets = InputSocket::list(ctx, schema_variant_id).await?;
            let output_sockets = OutputSocket::list(ctx, schema_variant_id).await?;

            let component_weight = InferredConnectionGraphNodeWeight {
                component,
                component_type,
                input_sockets,
                output_sockets,
            };

            let node_index = down_component_graph.add_node(component_weight);
            index_by_component_id.insert(component_id, node_index);
        }
        // Gather the "frame contains" information for all Components to build the edges of the
        // graph.
        for (&component_id, &source_node_index) in &index_by_component_id {
            for target_component_id in ctx
                .workspace_snapshot()?
                .frame_contains_components(component_id)
                .await
                .map_err(Box::new)?
            {
                let destination_node_index = *index_by_component_id
                    .get(&target_component_id)
                    .ok_or_else(|| {
                        InferredConnectionGraphError::OrphanedComponent(target_component_id)
                    })?;
                down_component_graph.add_edge(source_node_index, destination_node_index, ());
            }
        }

        let mut up_component_graph = down_component_graph.clone();
        up_component_graph.reverse();

        Ok(Self {
            down_component_graph,
            up_component_graph,
            index_by_component_id,
            inferred_connections_by_component_and_input_socket: HashMap::new(),
        })
    }

    #[instrument(
        name = "component.inferred_connection_graph.inferred_connections_for_all_components",
        level = "debug",
        skip(self, ctx)
    )]
    pub async fn inferred_connections_for_all_components(
        &mut self,
        ctx: &DalContext,
    ) -> InferredConnectionGraphResult<Vec<InferredConnection>> {
        let mut results = Vec::new();
        for component_id in Component::list_ids(ctx).await.map_err(Box::new)? {
            results.append(
                &mut self
                    .inferred_incoming_connections_for_component(ctx, component_id)
                    .await?,
            );
        }

        Ok(results)
    }

    #[instrument(
        name = "component.inferred_connection_graph.inferred_connections_for_component_stack",
        level = "debug",
        skip(self, ctx)
    )]
    pub async fn inferred_connections_for_component_stack(
        &mut self,
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> InferredConnectionGraphResult<Vec<InferredConnection>> {
        let mut inferred_connections = HashSet::new();
        let mut stack_component_ids = HashSet::new();

        fn get_component_id(
            event: DfsEvent<NodeIndex>,
            graph: &StableDiGraph<InferredConnectionGraphNodeWeight, ()>,
            collected_ids: &mut HashSet<ComponentId>,
        ) -> Control<()> {
            if let DfsEvent::Discover(node_index, _) = event {
                if let Some(node_weight) = graph.node_weight(node_index) {
                    collected_ids.insert(node_weight.component.id());
                }
            }
            Control::Continue
        }

        if let Some(&start_component_index) = self.index_by_component_id.get(&component_id) {
            petgraph::visit::depth_first_search(
                &self.up_component_graph,
                Some(start_component_index),
                |event| get_component_id(event, &self.up_component_graph, &mut stack_component_ids),
            );
            petgraph::visit::depth_first_search(
                &self.down_component_graph,
                Some(start_component_index),
                |event| {
                    get_component_id(event, &self.down_component_graph, &mut stack_component_ids)
                },
            );
        }

        for stack_component_id in stack_component_ids {
            inferred_connections.extend(
                self.inferred_incoming_connections_for_component(ctx, stack_component_id)
                    .await?,
            );
        }

        Ok(inferred_connections.iter().copied().collect())
    }

    #[instrument(
        name = "component.inferred_connection_graph.inferred_incoming_connections_for_component",
        level = "debug",
        skip(self, ctx),
        fields(si.inferred_connections.cache_hit = Empty)
    )]
    pub async fn inferred_incoming_connections_for_component(
        &mut self,
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> InferredConnectionGraphResult<Vec<InferredConnection>> {
        let span = Span::current();
        if let std::collections::hash_map::Entry::Occupied(component_cache) = self
            .inferred_connections_by_component_and_input_socket
            .entry(component_id)
        {
            let mut cached_results = Vec::new();
            for input_socket_info in component_cache.get().values() {
                cached_results.reserve(input_socket_info.len());
                cached_results.extend(input_socket_info);
            }

            span.record("si.inferred_connections.cache_hit", true);

            return Ok(cached_results);
        }
        span.record("si.inferred_connections.cache_hit", false);

        let input_sockets_with_explicit_connections: HashSet<InputSocketId> =
            Component::input_sockets_with_connections(ctx, component_id)
                .await
                .map_err(Box::new)?
                .iter()
                .copied()
                .collect();

        // `component_index` should be the same in both the `Up` and `Down` graphs since the `Up`
        // graph is created by reversing the edges in the `Down` graph, without touching the nodes
        // at all.
        let component_index =
            if let Some(&node_index) = self.index_by_component_id.get(&component_id) {
                node_index
            } else {
                // It's entirely possible we might not find a `Component` that is actually in the
                // `WorkspaceSnapshotGraph` in the graph we have built as we're only updating it when
                // a `Component` is added/removed from a `Frame`. `Component`s (including `Frame`s) that
                // are not inside of a `Frame` (or are not do not have inferred connections to anything.
                return Ok(Vec::new());
            };
        let mut input_sockets = self
            .up_component_graph
            .node_weight(component_index)
            .ok_or(InferredConnectionGraphError::MissingGraphNode)?
            .input_sockets
            .clone();
        let all_input_sockets: HashMap<InputSocketId, InputSocket> = input_sockets
            .iter()
            .cloned()
            .map(|is| (is.id(), is))
            .collect();
        // Any `InputSocket` with an explicit connection already made will not automatically infer
        // any additional connections.
        input_sockets.retain(|is| !input_sockets_with_explicit_connections.contains(&is.id()));

        // If all of the `InputSocket`s have explicit connections, then populate the cache with
        // empty entries, and return an empty list of `InferredConnection`s.
        if input_sockets.is_empty() {
            let mut empty_socket_map = HashMap::new();
            for input_socket in all_input_sockets.values() {
                empty_socket_map.insert(input_socket.id(), Vec::new());
            }
            self.inferred_connections_by_component_and_input_socket
                .insert(component_id, empty_socket_map);
            return Ok(Vec::new());
        }

        // We need to find all of the groups of `InputSocket`s that have similar shapes to each
        // other (including if an `InputSocket` is the only one with that shape). We want to
        // ensure that for each of these "exclusivity groups", a given source `Component` will
        // only occur once (regardless if it is with different, or the same `OutputSocket`). By
        // doing this, we will both prevent two similarly shaped `InputSocket` from trying to
        // pull from the same `Component` as well as prevent a single `InputSocket` from trying to
        // pull from multiple `OutputSocket` on a single `Component`.
        let mut exclusivity_groups: HashSet<BTreeSet<InputSocketId>> = HashSet::new();
        for input_socket in all_input_sockets.values() {
            let mut similarly_shaped_input_sockets = BTreeSet::new();
            for potentially_similaraly_shaped_input_socket in all_input_sockets.values() {
                if input_socket
                    .connection_annotations()
                    .iter()
                    .any(|input_annotation| {
                        potentially_similaraly_shaped_input_socket
                            .connection_annotations()
                            .iter()
                            .any(|potential_annotation| {
                                ConnectionAnnotation::target_fits_reference(
                                    input_annotation,
                                    potential_annotation,
                                )
                            })
                    })
                {
                    similarly_shaped_input_sockets
                        .insert(potentially_similaraly_shaped_input_socket.id());
                }
            }
            exclusivity_groups.insert(similarly_shaped_input_sockets);
        }

        let mut results = HashMap::new();
        // We need to calculate the potential `InferredConnection`s for _ALL_ `InputSocket`, even
        // if they already have an explicit connection since the ones with explicit connections
        // still affect the results of `InputSocket`s with no explicit connections.
        for &input_socket_id in all_input_sockets.keys() {
            let input_socket_results =
                self.raw_inferred_connections_for_input_socket(component_id, input_socket_id)?;

            results.insert(input_socket_id, input_socket_results);
        }

        // Across all of the `InputSocket`s on the `Component`, we can only use a specific
        // `OutputSocket + Component` pair once.
        let mut component_output_socket_usage = HashSet::new();
        let mut component_output_socket_reuse_violations = HashSet::new();
        for input_socket_id in all_input_sockets.keys() {
            if let Some(socket_matches) = results.get(input_socket_id) {
                for socket_match in socket_matches {
                    let component_socket_pair = (
                        socket_match.source_component_id,
                        socket_match.output_socket_id,
                    );
                    if component_output_socket_usage.contains(&component_socket_pair) {
                        component_output_socket_reuse_violations.insert(component_socket_pair);
                    }
                    component_output_socket_usage.insert(component_socket_pair);
                }
            }
        }

        // Within the "exclusivity groups" we need to make sure that there is not more than one
        // occurrence of a connection coming from a single Component, regardless of which
        // `OutputSocket` it is coming from.
        for exclusivity_group in &exclusivity_groups {
            let mut exclusivity_group_components = HashSet::new();
            let mut component_exclusivity_violations = HashSet::new();
            for group_input_socket_id in exclusivity_group {
                if let Some(socket_results) = results.get(group_input_socket_id) {
                    for inferred_connection in socket_results {
                        let source_component_id = inferred_connection.source_component_id;
                        if exclusivity_group_components.contains(&source_component_id) {
                            component_exclusivity_violations.insert(source_component_id);
                        }
                        exclusivity_group_components.insert(source_component_id);
                    }
                }
            }

            if !component_exclusivity_violations.is_empty()
                || !component_output_socket_reuse_violations.is_empty()
                || exclusivity_group.iter().any(|input_socket_id| {
                    if let Some(input_socket) = all_input_sockets.get(input_socket_id) {
                        input_socket.arity() == SocketArity::One
                    } else {
                        false
                    }
                })
            {
                for group_input_socket_id in exclusivity_group {
                    if let Some(socket_matches) = results.get_mut(group_input_socket_id) {
                        if let Some(input_socket) = all_input_sockets.get(group_input_socket_id) {
                            if input_socket.arity() == SocketArity::One && socket_matches.len() > 1
                            {
                                socket_matches.clear();
                            }
                        }
                        socket_matches.retain(|inferred_connection| {
                            !component_exclusivity_violations
                                .contains(&inferred_connection.source_component_id)
                                && !component_output_socket_reuse_violations.contains(&(
                                    inferred_connection.source_component_id,
                                    inferred_connection.output_socket_id,
                                ))
                        });
                    }
                }
            }
        }

        // We only want to keep the inferred results for the `InputSocket`s that don't already have
        // explicit connections.
        results.retain(|id, _| !input_sockets_with_explicit_connections.contains(id));
        self.inferred_connections_by_component_and_input_socket
            .entry(component_id)
            .and_modify(|component_cache| {
                // We should never really hit this case, but it's included for completeness. If
                // there were already an entry for this `Component`, we should have used it & early
                // returned a the beginning of this function.
                component_cache.clone_from(&results);
            })
            .or_insert_with(|| results.clone());

        Ok(results.values().flatten().copied().collect())
    }

    #[instrument(
        name = "component.inferred_connection_graph.inferred_connections_for_input_socket",
        level = "debug",
        skip(self, ctx)
    )]
    pub async fn inferred_connections_for_input_socket(
        &mut self,
        ctx: &DalContext,
        component_id: ComponentId,
        input_socket_id: InputSocketId,
    ) -> InferredConnectionGraphResult<Vec<InferredConnection>> {
        // The `InferredConnection`s for any given `InputSocket` on a `Component` are affected by
        // the `InferredConnection`s of all of the other `InputSocket` on that `Component` with an
        // overlaping shape. Because of this, we can't actually determine the `InferredConnection`
        // for a single `InputSocket` in isolation, and need to calculate the `InferredConnection`
        // for all `InputSocket` with overlaping shapes at the same time.
        self.inferred_incoming_connections_for_component(ctx, component_id)
            .await?;

        // Rather than iterating through all of the `InferredConnection`s across all `InputSocket`
        // of the `Component`, which might be a lot, we can go straight to the cache that we
        // maintain, and grab the results for the specific `InputSocket` we're interested in.
        if let Some(component_info) = self
            .inferred_connections_by_component_and_input_socket
            .get(&component_id)
        {
            Ok(component_info
                .get(&input_socket_id)
                .cloned()
                .unwrap_or_default())
        } else {
            Ok(Vec::new())
        }
    }

    #[instrument(
        name = "component.inferred_connection_graph.raw_inferred_connections_for_input_socket",
        level = "debug",
        skip(self)
    )]
    fn raw_inferred_connections_for_input_socket(
        &self,
        component_id: ComponentId,
        input_socket_id: InputSocketId,
    ) -> InferredConnectionGraphResult<Vec<InferredConnection>> {
        // `component_index` should be the same in both the `Up` and `Down` graphs since the `Up`
        // graph is created by reversing the edges in the `Down` graph, without touching the nodes
        // at all.
        let component_index =
            if let Some(&node_index) = self.index_by_component_id.get(&component_id) {
                node_index
            } else {
                // It's entirely possible we might not find a `Component` that is actually in the
                // `WorkspaceSnapshotGraph` in the graph we have built as we're only updating it when
                // a `Component` is added/removed from a `Frame`. `Component`s (including `Frame`s) that
                // are not inside of a `Frame` (or are not do not have inferred connections to anything.
                return Ok(Vec::new());
            };
        let node_info = self
            .down_component_graph
            .node_weight(component_index)
            .ok_or(InferredConnectionGraphError::MissingGraphNode)?;
        let component_type = node_info.component_type;
        let input_socket = node_info
            .input_sockets
            .iter()
            .find(|is| is.id() == input_socket_id)
            .ok_or_else(|| {
                ComponentError::InputSocketNotFoundForComponentId(input_socket_id, component_id)
            })
            .map_err(Box::new)?;

        // All `ComponentType` can pull their inputs in a `Frame` stack by going "up" to look for
        // a matching `OutputSocket`.
        let mut up_distances = HashMap::new();
        up_distances.insert(component_index, 0);
        let mut up_potential_matches = Vec::new();
        let allowed_match_kinds = match component_type {
            ComponentType::ConfigurationFrameDown => vec![ComponentType::ConfigurationFrameDown],
            ComponentType::Component => vec![
                // ComponentType::ConfigurationFrameUp,
                ComponentType::ConfigurationFrameDown,
            ],
            ComponentType::ConfigurationFrameUp => vec![ComponentType::ConfigurationFrameDown],
            t => {
                return Err(InferredConnectionGraphError::UnsupportedComponentType(
                    t,
                    component_id,
                ));
            }
        };
        petgraph::visit::depth_first_search(
            &self.up_component_graph,
            Some(component_index),
            |event| {
                cost_visitor(
                    event,
                    &self.up_component_graph,
                    component_id,
                    input_socket,
                    &allowed_match_kinds,
                    &mut up_distances,
                    &mut up_potential_matches,
                )
            },
        )?;

        match component_type {
            // Both `Component` and `ConfigurationFrameDown` only connect their `InputSocket`s to
            // things in the `Up` direction in the `Frame` stack. `Components`, because they are
            // the leaves in the stack and there is nothing else in the `Down` direction.
            // `ConfigurationFrameDown` because that is how we have defined their behavior.
            ComponentType::Component | ComponentType::ConfigurationFrameDown => {
                let mut inferred_connections = Vec::new();

                if !up_potential_matches.is_empty() {
                    let found_matches = closest_matches(
                        &self.up_component_graph,
                        up_distances,
                        up_potential_matches,
                    )?;
                    for found_match in found_matches {
                        inferred_connections.push(InferredConnection {
                            source_component_id: found_match.component_id,
                            output_socket_id: found_match.output_socket_id,
                            destination_component_id: component_id,
                            input_socket_id,
                        });
                    }
                }

                Ok(inferred_connections)
            }
            ComponentType::ConfigurationFrameUp => {
                // `ConfigurationFrameUp` can connect their `InputSocket`s to things in both the
                // `Up` and `Down` directions (potentially at the same time, depending on socket
                // arity), so we need to also find the `Down` costs.
                let mut down_distances = HashMap::new();
                down_distances.insert(component_index, 0);
                let mut down_potential_matches = Vec::new();
                petgraph::visit::depth_first_search(
                    &self.down_component_graph,
                    Some(component_index),
                    |event| {
                        cost_visitor(
                            event,
                            &self.down_component_graph,
                            component_id,
                            input_socket,
                            &[
                                ComponentType::ConfigurationFrameUp,
                                // ComponentType::ConfigurationFrameDown,
                                ComponentType::Component,
                            ],
                            &mut down_distances,
                            &mut down_potential_matches,
                        )
                    },
                )?;
                // We pull the entry for this component back out since we never want to hook a
                // component up to itself, and the entry for itself will always have the minimum
                // distance.
                down_distances.remove(&component_index);

                // If the `InputSocket` arity is `One`, and the frame has matches in both the up &
                // down directions, then we don't use any of them.
                if input_socket.arity() == SocketArity::One
                    && !up_potential_matches.is_empty()
                    && !down_potential_matches.is_empty()
                {
                    return Ok(Vec::new());
                }

                let mut inferred_connections = Vec::new();

                if !up_potential_matches.is_empty() {
                    let found_matches = closest_matches(
                        &self.up_component_graph,
                        up_distances,
                        up_potential_matches,
                    )?;
                    for found_match in found_matches {
                        inferred_connections.push(InferredConnection {
                            source_component_id: found_match.component_id,
                            output_socket_id: found_match.output_socket_id,
                            destination_component_id: component_id,
                            input_socket_id,
                        });
                    }
                }

                if !down_potential_matches.is_empty() {
                    let found_matches = closest_matches(
                        &self.down_component_graph,
                        down_distances,
                        down_potential_matches,
                    )?;
                    for found_match in found_matches {
                        inferred_connections.push(InferredConnection {
                            source_component_id: found_match.component_id,
                            output_socket_id: found_match.output_socket_id,
                            destination_component_id: component_id,
                            input_socket_id,
                        });
                    }
                }

                Ok(inferred_connections)
            }
            t => {
                return Err(InferredConnectionGraphError::UnsupportedComponentType(
                    t,
                    component_id,
                ));
            }
        }
    }
}

fn closest_matches(
    graph: &StableDiGraph<InferredConnectionGraphNodeWeight, ()>,
    mut distances: HashMap<NodeIndex, usize>,
    potential_matches: Vec<PotentialInferredConnectionMatch>,
) -> InferredConnectionGraphResult<Vec<PotentialInferredConnectionMatch>> {
    let potential_component_matches: HashSet<ComponentId> = potential_matches
        .iter()
        .map(|picm| picm.component_id)
        .collect();
    // We're only interested in the distances to the `Component`s that potentially match the shape
    // of the `InputSocket` we're trying to find the connection(s) of.
    distances.retain(|&node_index, _| {
        if let Some(node_weight) = graph.node_weight(node_index) {
            potential_component_matches.contains(&node_weight.component.id())
        } else {
            false
        }
    });
    // Of those `Component`s, we only care about the one(s) closest to the `InputSocket` within the
    // frame stack.
    let min_distance = *distances
        .values()
        .min()
        .ok_or_else(|| InferredConnectionGraphError::UnableToComputeCost)?;
    let mut closest_matches = Vec::new();
    distances.retain(|_, v| *v == min_distance);
    let mut match_ids = HashSet::new();
    for (match_index, _) in distances {
        match_ids.insert(
            graph
                .node_weight(match_index)
                .ok_or(InferredConnectionGraphError::MissingGraphNode)?
                .component
                .id(),
        );
    }

    for potential_match in potential_matches {
        if match_ids.contains(&potential_match.component_id) {
            closest_matches.push(potential_match);
        }
    }

    Ok(closest_matches)
}

fn cost_visitor(
    event: DfsEvent<NodeIndex>,
    graph: &StableDiGraph<InferredConnectionGraphNodeWeight, ()>,
    input_socket_component_id: ComponentId,
    input_socket: &InputSocket,
    allowed_component_types: &[ComponentType],
    distances: &mut HashMap<NodeIndex, usize>,
    socket_matches: &mut Vec<PotentialInferredConnectionMatch>,
) -> InferredConnectionGraphResult<Control<()>> {
    match event {
        DfsEvent::Discover(node_index, _) => {
            let potential_output_socket_node = match graph.node_weight(node_index) {
                Some(w) => w,
                None => return Ok(Control::Continue),
            };

            // If we're looking at the `OutputSocket`s of the same `Component` as the one we're
            // trying to match the `InputSocket` on, then just keep looking since a `Component`
            // isn't allowed to connect to itself automatically in a frame stack.
            if potential_output_socket_node.component.id() == input_socket_component_id
                || !allowed_component_types.contains(&potential_output_socket_node.component_type)
            {
                return Ok(Control::Continue);
            }

            let mut output_socket_matches = Vec::new();
            for output_socket in &potential_output_socket_node.output_sockets {
                if output_socket.fits_input(input_socket) {
                    output_socket_matches.push(output_socket.id());
                }
            }
            // If this `Component` has multiple `OutputSocket`s suitable for a connection, then
            // we don't use _any_ of the matches, _and_ we don't consider any of the potential
            // matches further in this direction.
            if output_socket_matches.len() > 1 {
                return Ok(Control::Prune);
            }

            if let Some(&output_socket_id) = output_socket_matches.first() {
                let component_id = potential_output_socket_node.component.id();
                socket_matches.push(PotentialInferredConnectionMatch {
                    component_id,
                    output_socket_id,
                });

                // Since we found a matching `OutputSocket` to use, we don't need to search in this
                // direction any further.
                return Ok(Control::Prune);
            }

            Ok(Control::Continue)
        }
        DfsEvent::Finish(_, _) => Ok(Control::Continue),
        DfsEvent::TreeEdge(source_index, target_index)
        | DfsEvent::BackEdge(source_index, target_index)
        | DfsEvent::CrossForwardEdge(source_index, target_index) => {
            // Keep track of the distance to nodes as we see the edges to them.
            if let Some(&previous_distance) = distances.get(&source_index) {
                distances.insert(target_index, previous_distance + 1);
            }
            Ok(Control::Continue)
        }
    }
}
