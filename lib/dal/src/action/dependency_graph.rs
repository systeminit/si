use itertools::Itertools;
use petgraph::prelude::*;
use std::collections::{HashMap, HashSet, VecDeque};
use telemetry::prelude::*;

use crate::{
    action::{Action, ActionId},
    dependency_graph::DependencyGraph,
    Component, ComponentId, DalContext,
};

use super::{
    prototype::{ActionKind, ActionPrototype},
    ActionError, ActionResult,
};

#[derive(Debug, Clone)]
pub struct ActionDependencyGraph {
    inner: DependencyGraph<ActionId>,
}

impl Default for ActionDependencyGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl ActionDependencyGraph {
    pub fn new() -> Self {
        Self {
            inner: DependencyGraph::new(),
        }
    }

    pub fn is_acyclic(&self) -> bool {
        petgraph::algo::toposort(self.inner.graph(), None).is_ok()
    }

    /// Construct an [`ActionDependencyGraph`] of all of the queued [`Action`s][crate::action::Action]
    /// for the current [`WorkspaceSnapshot`][crate::WorkspaceSnapshot].
    #[instrument(
        level = "info",
        name = "action.dependency_graph.for_workspace",
        skip(ctx)
    )]
    pub async fn for_workspace(ctx: &DalContext) -> ActionResult<Self> {
        // * Get all ActionId -> ComponentId mappings.
        // * For each of these ComponentIds (A):
        //     * For each Input Socket:
        //         * For each source ComponentId (B):
        //           * All Actions for Component A depend on All actions for Component B
        let mut component_dependencies: StableDiGraph<ComponentId, ()> = StableDiGraph::new();
        let mut component_dependencies_index_by_id: HashMap<ComponentId, NodeIndex> =
            HashMap::new();
        // let mut component_dependencies: HashMap<ComponentId, HashSet<ComponentId>> = HashMap::new();
        // let mut component_reverse_dependencies: HashMap<ComponentId, HashSet<ComponentId>> =
        //     HashMap::new();
        let mut actions_by_component_id: HashMap<ComponentId, HashSet<ActionId>> = HashMap::new();
        let mut action_dependency_graph = Self::new();
        let mut action_kinds: HashMap<ActionId, ActionKind> = HashMap::new();

        // Need to get all actions that are still in the "queue", including those that have failed,
        // or are currently running.
        for action_id in Action::all_ids(ctx).await? {
            action_dependency_graph.inner.add_id(action_id);
            // Theoretically, we may have Actions at some point that aren't Component specific.
            if let Some(component_id) = Action::component_id(ctx, action_id).await? {
                actions_by_component_id
                    .entry(component_id)
                    .or_default()
                    .insert(action_id);
            }
            let action_prototype_id = Action::prototype_id(ctx, action_id).await?;
            let action_prototype = ActionPrototype::get_by_id(ctx, action_prototype_id).await?;
            action_kinds.insert(action_id, action_prototype.kind);
        }

        // TODO: Account for explicitly defiend dependencies between actions. These should be edges
        //       directly between two Actions, but are not implemented yet.

        // Get all inferred connections up front so we don't build this tree each time
        let workspace_snapshot = ctx.workspace_snapshot()?;
        let mut component_tree = workspace_snapshot.inferred_connection_graph(ctx).await?;
        // Action dependencies are primarily based on the data flow between Components. Things that
        // feed data into other things must have their actions run before the actions for the
        // things they are feeding data into.
        for component_id in actions_by_component_id.keys().copied() {
            let component = Component::get_by_id(ctx, component_id).await?;
            let component_index = component_dependencies_index_by_id
                .entry(component_id)
                .or_insert_with(|| component_dependencies.add_node(component_id))
                .to_owned();
            for incoming_connection in component.incoming_connections(ctx).await? {
                component_dependencies_index_by_id
                    .entry(incoming_connection.from_component_id)
                    .or_insert_with(|| {
                        component_dependencies.add_node(incoming_connection.from_component_id)
                    });
                if let Some(&source_component_index) =
                    component_dependencies_index_by_id.get(&incoming_connection.from_component_id)
                {
                    // The edges of this graph go `output_socket_component (source) ->
                    // input_socket_component (target)`, matching the flow of the data between
                    // components.
                    component_dependencies.update_edge(source_component_index, component_index, ());
                }
            }
            for inferred_connection in component_tree
                .inferred_incoming_connections_for_component(ctx, component_id)
                .await?
            {
                component_dependencies_index_by_id
                    .entry(inferred_connection.source_component_id)
                    .or_insert_with(|| {
                        component_dependencies.add_node(inferred_connection.source_component_id)
                    });
                if let Some(&source_component_index) =
                    component_dependencies_index_by_id.get(&inferred_connection.source_component_id)
                {
                    // The edges of this graph go `output_socket_component (source) ->
                    // input_socket_component (target)`, matching the flow of the data between
                    // components.
                    component_dependencies.update_edge(source_component_index, component_index, ());
                }
            }
        }

        // Each Component's Actions need to be marked as depending on the Actions that the
        // Component itself has been determined to be depending on.
        for (component_id, action_ids) in &actions_by_component_id {
            if let Some(&component_index) = component_dependencies_index_by_id.get(component_id) {
                for &component_action_id in action_ids {
                    let action_kind = action_kinds
                        .get(&component_action_id)
                        .copied()
                        .ok_or(ActionError::UnableToGetKind(component_action_id))?;
                    // Given a data flow between components of:
                    //     `Component A -> Component B`
                    //
                    // * `ActionKind::Destroy` for `Component A` would run _after_ `Actions` for
                    //   `Component A`. (A depends on the components from the `Outgoing` data flow
                    //   edges)
                    // * For all other `ActionKind`, `Actions` for `Component B` would run _after_
                    //   `Actions` for `Component A`. (`B` depends on the components from the
                    //   `Incoming` data flow edges.)
                    let dependency_direction = match action_kind {
                        ActionKind::Create
                        | ActionKind::Manual
                        | ActionKind::Refresh
                        | ActionKind::Update => Incoming,
                        ActionKind::Destroy => Outgoing,
                    };

                    for dependency_edgeref in
                        component_dependencies.edges_directed(component_index, dependency_direction)
                    {
                        let dependency_node_index = match dependency_direction {
                            Outgoing => dependency_edgeref.target(),
                            Incoming => dependency_edgeref.source(),
                        };
                        if let Some(dependency_component_id) =
                            component_dependencies.node_weight(dependency_node_index)
                        {
                            for dependency_action_id in actions_by_component_id
                                .get(dependency_component_id)
                                .cloned()
                                .unwrap_or_default()
                            {
                                action_dependency_graph
                                    .action_depends_on(component_action_id, dependency_action_id);
                            }
                        };
                    }
                }
            }
        }

        Ok(action_dependency_graph)
    }

    pub fn action_depends_on(&mut self, action_id: ActionId, depends_on_id: ActionId) {
        self.inner.id_depends_on(action_id, depends_on_id);
    }

    pub fn contains_value(&self, action_id: ActionId) -> bool {
        self.inner.contains_id(action_id)
    }
    /// gets what actions are directly dependent on a given action id
    /// ex: Create -> Update -> Delete
    /// graph.direct_dependencies_of(update.actionid) -> Create
    pub fn direct_dependencies_of(&self, action_id: ActionId) -> Vec<ActionId> {
        self.inner.direct_dependencies_of(action_id)
    }

    pub fn remove_action(&mut self, action_id: ActionId) {
        self.inner.remove_id(action_id);
    }

    pub fn cycle_on_self(&mut self, action_id: ActionId) {
        self.inner.cycle_on_self(action_id);
    }

    pub fn independent_actions(&self) -> Vec<ActionId> {
        self.inner.independent_ids()
    }

    pub fn remaining_actions(&self) -> Vec<ActionId> {
        self.inner.remaining_ids()
    }

    /// Gets all downstream dependencies for the provided ActionId. This includes the entire subgraph
    /// starting at ActionId.
    #[instrument(level = "debug", skip(self))]
    pub fn get_all_dependencies(&self, action_id: ActionId) -> Vec<ActionId> {
        let current_dependencies = self.inner.direct_reverse_dependencies_of(action_id);
        let mut all_dependencies = HashSet::new();
        let mut work_queue = VecDeque::from(current_dependencies.clone());
        while let Some(action) = work_queue.pop_front() {
            match all_dependencies.insert(action) {
                true => {
                    let next = self.inner.direct_reverse_dependencies_of(action);
                    work_queue.extend(next);
                }
                false => continue,
            }
        }
        all_dependencies.into_iter().collect_vec()
    }
}
