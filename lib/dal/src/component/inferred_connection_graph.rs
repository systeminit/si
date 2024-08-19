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

use itertools::Itertools;
use std::collections::{HashMap, HashSet, VecDeque};
use telemetry::prelude::*;

use super::{
    socket::{ComponentInputSocket, ComponentOutputSocket},
    ComponentResult,
};
use crate::{
    Component, ComponentId, ComponentType, DalContext, InputSocket, OutputSocket, OutputSocketId,
    SocketArity,
};

#[derive(Eq, PartialEq, Clone, Debug, Default)]
pub struct InferredConnections {
    pub input_sockets: HashMap<ComponentInputSocket, HashSet<ComponentOutputSocket>>,
    pub output_sockets: HashMap<ComponentOutputSocket, HashSet<ComponentInputSocket>>,
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct InferredConnection {
    pub input_socket: ComponentInputSocket,
    pub output_socket: ComponentOutputSocket,
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct InferredConnectionGraph {
    components: HashMap<ComponentId, HashSet<ComponentId>>,
    connections: HashMap<ComponentId, InferredConnections>,
}

impl InferredConnectionGraph {
    /// Assembles the [`InferredConnectionGraph`] from the perspective of the given [`ComponentId`]s by first creating a
    /// a tree representing this Component and all others in the lineage grouping (i.e walk to the top and then get all children)
    /// for all given ComponentIds.
    /// Then, find all inferred connections for everything we've found
    #[instrument(
        name = "component.inferred_connection_graph.assemble_for_components",
        level = "info",
        skip(ctx)
    )]
    pub async fn assemble_for_components(
        ctx: &DalContext,
        component_ids: Vec<ComponentId>,
    ) -> ComponentResult<Self> {
        let mut component_incoming_connections: HashMap<
            ComponentId,
            HashMap<ComponentInputSocket, HashSet<ComponentOutputSocket>>,
        > = HashMap::new();
        let mut top_parents: HashSet<ComponentId> = HashSet::new();
        let mut components: HashMap<ComponentId, HashSet<ComponentId>> = HashMap::new();

        for component_id in component_ids {
            // Check if the component_id is either a key or a value in the components HashMap
            let is_in_tree = components.contains_key(&component_id)
                || components.values().any(|x| x.contains(&component_id));

            // If this component id isn't already in the hashmap somewhere, skip it
            // since we already have accounted for it!
            if !is_in_tree {
                // Get the outermost parent of this tree
                let first_top_parent = Self::find_top_parent(ctx, component_id).await?;
                top_parents.insert(first_top_parent);
                let this_tree = Self::build_tree(ctx, first_top_parent).await?;
                components.extend(this_tree);
            }
        }

        for parent in &top_parents {
            // Walk down the tree and accumulate connections for every input socket
            let mut work_queue = VecDeque::new();
            work_queue.push_back(*parent);

            while let Some(component) = work_queue.pop_front() {
                let (input_sockets, duplicates) = Self::process_component(ctx, component).await?;
                Self::update_incoming_connections(
                    &mut component_incoming_connections,
                    component,
                    input_sockets,
                    duplicates,
                );

                // Load up next children
                if let Some(children) = components.get(&component) {
                    work_queue.extend(children.clone());
                }
            }
        }

        // Populate outgoing connections
        let component_tree = Self::populate_graph(component_incoming_connections);

        Ok(InferredConnectionGraph {
            connections: component_tree,
            components,
        })
    }

    /// Create a [`InferredConnectionGraph`] containing all [`Component`]s in this Workspace
    #[instrument(
        name = "component.inferred_connection_graph.for_workspace",
        level = "info",
        skip(ctx)
    )]
    pub async fn for_workspace(ctx: &DalContext) -> ComponentResult<Self> {
        // get all components
        let components = Component::list(ctx)
            .await?
            .into_iter()
            .map(|component| component.id())
            .collect_vec();
        Self::assemble_for_components(ctx, components).await
    }

    /// Assembles the [`InferredConnectionGraph`] from the perspective of this single [`ComponentId`] by first creating a
    /// a tree representing this Component and all others in the lineage grouping (i.e walk to the top and then get all children)
    /// Then, find all inferred connections for everything in the tree
    #[instrument(
        name = "component.inferred_connection_graph.assemble",
        level = "info",
        skip(ctx)
    )]
    pub async fn assemble(
        ctx: &DalContext,
        with_component_id: ComponentId,
    ) -> ComponentResult<Self> {
        Self::assemble_for_components(ctx, vec![with_component_id]).await
    }

    /// Assembles the a map of Incoming Connections from the perspective of this single [`ComponentId`]
    #[instrument(
        name = "component.inferred_connection_graph.assemble_incoming_only",
        level = "info",
        skip(ctx)
    )]
    pub async fn assemble_incoming_only(
        ctx: &DalContext,
        for_component_id: ComponentId,
    ) -> ComponentResult<HashMap<ComponentInputSocket, Vec<ComponentOutputSocket>>> {
        let mut component_incoming_connections: HashMap<
            ComponentId,
            HashMap<ComponentInputSocket, HashSet<ComponentOutputSocket>>,
        > = HashMap::new();
        let (input_sockets, duplicates) = Self::process_component(ctx, for_component_id).await?;
        Self::update_incoming_connections(
            &mut component_incoming_connections,
            for_component_id,
            input_sockets,
            duplicates,
        );
        let mut incoming_connections = HashMap::new();
        for (input_socket, output_sockets) in component_incoming_connections
            .get(&for_component_id)
            .unwrap_or(&HashMap::new())
        {
            let outputs = output_sockets
                .iter()
                .cloned()
                .sorted_by_key(|output| output.component_id)
                .collect();
            incoming_connections.insert(*input_socket, outputs);
        }
        Ok(incoming_connections)
    }

    /// Walk the frame contains edges to find the top most parent with the given ComponentId beneath it
    async fn find_top_parent(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> ComponentResult<ComponentId> {
        let mut parent_id = component_id;
        while let Some(parent) = Component::get_parent_by_id(ctx, parent_id).await? {
            parent_id = parent;
        }
        Ok(parent_id)
    }

    /// find all inferred incoming connections for the provided component by looping through the component's
    /// input sockets
    async fn process_component(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> ComponentResult<(
        HashMap<ComponentInputSocket, HashSet<ComponentOutputSocket>>,
        HashSet<ComponentOutputSocket>,
    )> {
        let mut input_sockets: HashMap<ComponentInputSocket, HashSet<ComponentOutputSocket>> =
            HashMap::new();
        let mut duplicate_tracker: HashSet<ComponentOutputSocket> = HashSet::new();
        let mut duplicates: HashSet<ComponentOutputSocket> = HashSet::new();

        let current_component_input_sockets =
            ComponentInputSocket::list_for_component_id(ctx, component_id).await?;

        for input_socket in current_component_input_sockets {
            let incoming_connections = Self::find_available_connections(ctx, input_socket).await?;
            // Get all existing explicit outgoing connections so that we don't create an inferred connection to the same output socket
            let existing_incoming_connections =
                Component::incoming_connections_for_id(ctx, component_id).await?;
            for incoming_connection in existing_incoming_connections {
                let component_output_socket = ComponentOutputSocket::get_by_ids_or_error(
                    ctx,
                    incoming_connection.from_component_id,
                    incoming_connection.from_output_socket_id,
                )
                .await?;
                // note: we don't care if a user has drawn multiple edges to the same output socket
                duplicate_tracker.insert(component_output_socket);
            }
            for output_socket in &incoming_connections {
                if !duplicate_tracker.insert(*output_socket) {
                    duplicates.insert(*output_socket);
                }
            }
            input_sockets
                .entry(input_socket)
                .or_insert(incoming_connections);
        }
        Ok((input_sockets, duplicates))
    }

    /// Adds the found incoming connections to the map, removing duplicate connections (if one input socket is connected to
    /// two output sockets from the same component)
    fn update_incoming_connections(
        component_incoming_connections: &mut HashMap<
            ComponentId,
            HashMap<ComponentInputSocket, HashSet<ComponentOutputSocket>>,
        >,
        component: ComponentId,
        input_sockets: HashMap<ComponentInputSocket, HashSet<ComponentOutputSocket>>,
        duplicates: HashSet<ComponentOutputSocket>,
    ) {
        for (input_socket, output_vec) in input_sockets {
            let filtered_output_vec: HashSet<ComponentOutputSocket> = output_vec
                .into_iter()
                .filter(|os| !duplicates.contains(os))
                .collect();

            // if filtered is empty, there are no connections to this input socket, don't add it
            if !filtered_output_vec.is_empty() {
                component_incoming_connections
                    .entry(component)
                    .and_modify(|connections| {
                        connections
                            .entry(input_socket)
                            .or_insert_with(|| filtered_output_vec.clone());
                    })
                    .or_insert_with(|| HashMap::from([(input_socket, filtered_output_vec)]));
            }
        }
    }

    /// Given the full list of incoming connections, use it to build the map of outgoing connections keyed by output socket
    fn populate_graph(
        component_incoming_connections: HashMap<
            ComponentId,
            HashMap<ComponentInputSocket, HashSet<ComponentOutputSocket>>,
        >,
    ) -> HashMap<ComponentId, InferredConnections> {
        let mut component_tree: HashMap<ComponentId, InferredConnections> = HashMap::new();
        for (component, incoming_connections) in component_incoming_connections {
            for (input_socket, output_sockets) in incoming_connections {
                component_tree
                    .entry(component)
                    .or_default()
                    .input_sockets
                    .entry(input_socket)
                    .and_modify(|outgoing| outgoing.extend(output_sockets.iter().cloned()))
                    .or_insert_with(|| output_sockets.clone());

                for output_socket in output_sockets {
                    component_tree
                        .entry(output_socket.component_id)
                        .or_default()
                        .output_sockets
                        .entry(output_socket)
                        .and_modify(|connections| {
                            connections.insert(input_socket);
                        })
                        .or_insert_with(|| HashSet::from([input_socket]));
                }
            }
        }

        component_tree
    }

    fn get_component_inferred_connections(&self, component_id: ComponentId) -> InferredConnections {
        let connections = match self.connections.get(&component_id) {
            Some(connections) => connections.clone(),
            None => InferredConnections {
                input_sockets: HashMap::new(),
                output_sockets: HashMap::new(),
            },
        };
        connections
    }

    /// Get all inferred incoming connections for a given component
    pub fn get_inferred_incoming_connections_to_component(
        &self,
        component_id: ComponentId,
    ) -> Vec<InferredConnection> {
        let mut inferred_incoming_connections = Vec::new();
        let incoming_connections = self.get_component_inferred_connections(component_id);
        for (input_socket, output_sockets) in incoming_connections.input_sockets {
            for output_socket in output_sockets {
                inferred_incoming_connections.push(InferredConnection {
                    input_socket,
                    output_socket,
                });
            }
        }
        inferred_incoming_connections
    }

    /// Get all inferred outgoing connections for a given [`ComponentId`]
    pub fn get_inferred_outgoing_connections_for_component(
        &self,
        component_id: ComponentId,
    ) -> Vec<InferredConnection> {
        let mut inferred_outgoing_connections = Vec::new();
        let connections = self.get_component_inferred_connections(component_id);
        for (output_socket, input_sockets) in connections.output_sockets {
            for input_socket in input_sockets {
                inferred_outgoing_connections.push(InferredConnection {
                    input_socket,
                    output_socket,
                });
            }
        }
        inferred_outgoing_connections
    }

    /// Get all inferred connections to a given [`ComponentId`] and [`OutputSocketId`]
    pub fn get_component_connections_to_output_socket(
        &self,
        component_id: ComponentId,
        output_socket_id: OutputSocketId,
    ) -> HashSet<ComponentInputSocket> {
        self.get_component_inferred_connections(component_id)
            .output_sockets
            .into_iter()
            .find_map(|(output_socket, input_sockets)| {
                if output_socket.output_socket_id == output_socket_id
                    && output_socket.component_id == component_id
                {
                    Some(input_sockets)
                } else {
                    None
                }
            })
            .unwrap_or_else(HashSet::new)
            .clone()
    }

    /// Get all inferred connections to a given [`ComponentInputSocket`]
    pub fn get_component_connections_to_input_socket(
        &self,
        component_input_socket: ComponentInputSocket,
    ) -> HashSet<ComponentOutputSocket> {
        self.get_component_inferred_connections(component_input_socket.component_id)
            .input_sockets
            .get(&component_input_socket)
            .cloned()
            .unwrap_or_default()
    }

    /// Get all inferred connections that exist for the constructed [`InferredConnectionGraph`].
    pub fn get_all_inferred_connections(&self) -> Vec<InferredConnection> {
        let mut incoming_connections = Vec::new();
        for inferred_connection in self.connections.values() {
            for (input_socket, output_sockets) in inferred_connection.input_sockets.clone() {
                for output_socket in output_sockets {
                    incoming_connections.push(InferredConnection {
                        input_socket,
                        output_socket,
                    });
                }
            }
        }
        incoming_connections
    }

    /// For the provided [`ComponentId`], build a tree of all Components that are a descendant (directly or indirectly)
    #[instrument(level = "info", skip(ctx))]
    async fn build_tree(
        ctx: &DalContext,
        first_top_parent: ComponentId,
    ) -> ComponentResult<HashMap<ComponentId, HashSet<ComponentId>>> {
        let mut resp = HashMap::new();
        let mut work_queue = VecDeque::new();
        work_queue.push_back(first_top_parent);
        while let Some(parent) = work_queue.pop_front() {
            let children = Component::get_children_for_id(ctx, parent).await?;
            resp.insert(
                parent,
                children
                    .clone()
                    .into_iter()
                    .collect::<HashSet<ComponentId>>(),
            );
            work_queue.extend(children);
        }
        Ok(resp)
    }

    /// For a given [`ComponentInputSocket`], find all incoming connections based on the [`ComponentType`]. If the
    /// [`ComponentInputSocket`] is manually configured (aka a user has drawn an edge to it), we return early as we do not
    /// mix and match Inferred/Explicit connections.
    ///
    /// [`ComponentType::Component`]s and [`ComponentType::ConfigurationFrameDown`] search up the ancestry tree
    /// and will connect to the closest matching [`ComponentOutputSocket`]
    ///
    /// [`ComponentType::ConfigurationFrameUp`] can connect to both descendants of the up frame, and ascendants of the up frame
    /// if the ascendant is a [`ComponentType::ConfigurationFrameDown`]
    async fn find_available_connections(
        ctx: &DalContext,
        component_input_socket: ComponentInputSocket,
    ) -> ComponentResult<HashSet<ComponentOutputSocket>> {
        // if this socket is manually configured, early return
        if ComponentInputSocket::is_manually_configured(ctx, component_input_socket).await? {
            return Ok(HashSet::new());
        }

        let destination_sockets =
            match Component::get_type_by_id(ctx, component_input_socket.component_id).await? {
                ComponentType::Component | ComponentType::ConfigurationFrameDown => {
                    //For a component, or a down frame, check my parents and other ancestors
                    // find the first output socket match that is a down frame and use it!
                    Self::find_closest_connection_in_ancestors(
                        ctx,
                        component_input_socket,
                        vec![ComponentType::ConfigurationFrameDown],
                    )
                    .await?
                }
                ComponentType::ConfigurationFrameUp => {
                    // An up frame's input sockets are sourced from either its children's output sockets
                    // or an ancestor.  Based on the input socket's arity, we match many (sorted by component ulid)
                    // or if the arity is single, we return none
                    let mut matches = vec![];
                    let descendant_matches = Self::find_connections_in_descendants(
                        ctx,
                        component_input_socket,
                        vec![
                            ComponentType::ConfigurationFrameUp,
                            ComponentType::Component,
                        ],
                    )
                    .await?;
                    matches.extend(descendant_matches);

                    if let [connection] = Self::find_closest_connection_in_ancestors(
                        ctx,
                        component_input_socket,
                        vec![ComponentType::ConfigurationFrameDown],
                    )
                    .await?
                    .as_slice()
                    {
                        matches.push(*connection);
                    }

                    let input_socket =
                        InputSocket::get_by_id(ctx, component_input_socket.input_socket_id).await?;
                    if input_socket.arity() == SocketArity::One && matches.len() > 1 {
                        vec![]
                    } else {
                        matches
                    }
                }
                ComponentType::AggregationFrame => vec![], // Aggregation Frames are not supported
            };
        Ok(destination_sockets
            .into_iter()
            .collect::<HashSet<ComponentOutputSocket>>())
    }

    /// For the given [`ComponentInputSocket`], discover if this set of children has any connections.  If the
    /// [`ComponentInputSocket`] is [`SocketArity::Many`], we allow multiple children to connect to it. If it's
    /// a [`SocketArity::One`], we only allow one [`ComponentOutputSocket`] to connect to it, so if we find that
    /// there are multiple connections, we return an empty list which will force the user to decide which one they
    /// want.
    ///
    /// This helper is useful so that we can early return and stop looking if we find there is a potential connection
    /// that is ambiguious (as we don't skip ambiguous connections and continue search their children)
    async fn find_connections_for_children(
        ctx: &DalContext,
        input_socket: ComponentInputSocket,
        component_types: &[ComponentType],
        children: Vec<ComponentId>,
        arity: SocketArity,
    ) -> ComponentResult<(Vec<ComponentOutputSocket>, Vec<ComponentId>)> {
        let mut output_matches = vec![];
        let mut next_children = vec![];

        for child in children {
            if component_types.contains(&Component::get_type_by_id(ctx, child).await?) {
                let matches = Self::find_connections_in_component(ctx, input_socket, child).await?;
                match arity {
                    SocketArity::One if matches.len() == 1 => {
                        output_matches.push(matches[0]);
                        return Ok((output_matches, vec![])); // Stop searching after finding one match
                    }
                    SocketArity::One if matches.len() > 1 => {
                        return Ok((vec![], vec![])); // Multiple matches found, return empty
                    }
                    SocketArity::One => {} // no matches, do nothing and keep looking
                    SocketArity::Many => {
                        output_matches.extend(matches);
                    }
                }
            }
            next_children.extend(Component::get_children_for_id(ctx, child).await?);
        }

        Ok((output_matches, next_children))
    }

    /// For the provided [`ComponentInputSocket`], find any matching [`ComponentOutputSocket`] that should
    /// drive this [`InputSocket`] by searching down the descendants of the [`Component`],
    /// checking children first and walking down until we find any matches.
    ///
    /// If the provided [`ComponentInputSocket`] has a [`SocketArity::One`], we look for only one
    /// eligible [`OutputSocket`]. If we find multiple, we won't return any, forcing the
    /// user to explicity draw the edge.
    ///
    /// If it has an [`SocketArity::Many`], we will look for multiple matches, but they must
    /// be at the same 'level' to be considered valid.
    async fn find_connections_in_descendants(
        ctx: &DalContext,
        component_input_socket: ComponentInputSocket,
        component_types: Vec<ComponentType>,
    ) -> ComponentResult<Vec<ComponentOutputSocket>> {
        let component_id = component_input_socket.component_id;
        let children = Component::get_children_for_id(ctx, component_id).await?;
        let socket_arrity = InputSocket::get_by_id(ctx, component_input_socket.input_socket_id)
            .await?
            .arity();
        //load up the children and look for matches
        let mut work_queue: VecDeque<Vec<ComponentId>> = VecDeque::new();
        work_queue.push_front(children);
        while let Some(children) = work_queue.pop_front() {
            let (matches, next_children) = Self::find_connections_for_children(
                ctx,
                component_input_socket,
                &component_types,
                children,
                socket_arrity,
            )
            .await?;

            // if there are matches found, return them and stop looking
            // otherwise, load up the next children if there are any
            if matches.is_empty() && !next_children.is_empty() {
                work_queue.push_back(next_children);
            } else {
                return Ok(matches);
            }
        }
        Ok(vec![])
    }

    /// For the provided [`ComponentInputSocket`], find the nearest [`ComponentOutputSocket`] in the ancestry tree
    /// that should drive this [`InputSocket`] (first searching parents and onwards up the ancestry tree)
    #[instrument(level = "debug", skip(ctx))]
    async fn find_closest_connection_in_ancestors(
        ctx: &DalContext,
        component_input_socket: ComponentInputSocket,
        component_types: Vec<ComponentType>,
    ) -> ComponentResult<Vec<ComponentOutputSocket>> {
        if let Some(parent_id) =
            Component::get_parent_by_id(ctx, component_input_socket.component_id).await?
        {
            let mut work_queue = VecDeque::from([parent_id]);
            while let Some(component_id) = work_queue.pop_front() {
                // see if this component is the right type

                if component_types.contains(&Component::get_type_by_id(ctx, component_id).await?) {
                    // get all output sockets for this component
                    let maybe_matches = Self::find_connections_in_component(
                        ctx,
                        component_input_socket,
                        component_id,
                    )
                    .await?;
                    {
                        if maybe_matches.len() > 1 {
                            // this ancestor has more than one match
                            // stop looking and return None to force
                            // the user to manually draw an edge to this socket
                            debug!("More than one match found: {:?}", maybe_matches);
                            return Ok(vec![]);
                        }
                        if maybe_matches.len() == 1 {
                            // this ancestor has 1 match!
                            // return and stop looking
                            return Ok(maybe_matches);
                        }
                    }
                }
                // didn't find it, so let's queue up the next parent if it exists
                if let Some(maybe_parent_id) =
                    Component::get_parent_by_id(ctx, component_id).await?
                {
                    work_queue.push_back(maybe_parent_id);
                }
            }
        }

        Ok(vec![])
    }

    /// For a given [`ComponentInputSocket`], search within the given [`ComponentId`] to see if there are any [`ComponentOutputSocket`]s
    /// Note: this does not enforce or check [`SocketArity`]. Even though we do not want one [`InputSocket`] to match multiple [`OutputSocket`]s
    /// from the same Component, that is enforced elsewhere to differentiate between 'this is an ambiguous connection' and 'there are no available
    /// connections here'
    async fn find_connections_in_component(
        ctx: &DalContext,
        input_socket_match: ComponentInputSocket,
        source_component_id: ComponentId,
    ) -> ComponentResult<Vec<ComponentOutputSocket>> {
        // check for matching output socket names for this input socket
        let parent_sv_id = Component::schema_variant_id(ctx, source_component_id).await?;
        let output_socket_ids =
            OutputSocket::list_ids_for_schema_variant(ctx, parent_sv_id).await?;
        let mut maybe_matches = vec![];

        for output_socket_id in output_socket_ids {
            if OutputSocket::fits_input_by_id(
                ctx,
                input_socket_match.input_socket_id,
                output_socket_id,
            )
            .await?
            {
                if let Some(component_output_socket) =
                    ComponentOutputSocket::get_by_ids(ctx, source_component_id, output_socket_id)
                        .await?
                {
                    maybe_matches.push(component_output_socket);
                }
            }
        }
        Ok(maybe_matches)
    }
}
