use std::{
    collections::{
        BTreeMap,
        HashMap,
        HashSet,
        VecDeque,
        btree_map,
        hash_map::Entry,
    },
    fs::File,
    io::Write,
};

use petgraph::prelude::*;
use si_events::ulid::Ulid;
use telemetry::prelude::*;

use super::{
    AttributeValue,
    AttributeValueError,
    AttributeValueId,
    AttributeValueResult,
    subscription::ValueSubscription,
};
use crate::{
    Component,
    ComponentError,
    ComponentId,
    DalContext,
    EdgeWeightKind,
    Prop,
    PropKind,
    Secret,
    attribute::{
        prototype::{
            AttributePrototype,
            argument::AttributePrototypeArgument,
        },
        value::ValueIsFor,
    },
    component::{
        ControllingFuncData,
        socket::ComponentOutputSocket,
    },
    dependency_graph::DependencyGraph,
    workspace_snapshot::{
        DependentValueRoot,
        WorkspaceSnapshotSelector,
        edge_weight::EdgeWeightKindDiscriminants,
        node_weight::NodeWeightDiscriminants,
    },
};

#[derive(Debug, Clone)]
pub struct DependentValueGraph {
    inner: DependencyGraph<AttributeValueId>,
    values_that_need_to_execute_from_prototype_function: HashSet<AttributeValueId>,
    component_ids_for_av: BTreeMap<AttributeValueId, ComponentId>,
}

// We specifically need to track if the value is one of the child values we
// added to the graph in order to discover if a dynamically set object's
// children are inputs to a function. The other two parts of this enum are not
// used now, but may be useful information when debugging
enum WorkQueueValue {
    Initial(AttributeValueId),
    ObjectChild(AttributeValueId),
    Discovered(AttributeValueId),
}

impl WorkQueueValue {
    fn id(&self) -> AttributeValueId {
        match self {
            WorkQueueValue::Initial(id)
            | WorkQueueValue::ObjectChild(id)
            | WorkQueueValue::Discovered(id) => *id,
        }
    }
}

impl DependentValueGraph {
    /// Construct a [`DependentValueGraph`] of all the [`AttributeValueIds`](AttributeValue) who are
    /// dependent on the initial ids provided as well as all descending dependencies.
    pub async fn new(
        ctx: &DalContext,
        roots: Vec<DependentValueRoot>,
    ) -> AttributeValueResult<Self> {
        let mut dependent_value_graph = Self {
            inner: DependencyGraph::new(),
            values_that_need_to_execute_from_prototype_function: HashSet::new(),
            component_ids_for_av: BTreeMap::new(),
        };

        let values = dependent_value_graph.parse_initial_ids(ctx, roots).await?;
        dependent_value_graph
            .populate_for_values(ctx, values)
            .await?;
        Ok(dependent_value_graph)
    }

    /// Parse the set of initial ids in order to construct the list of [`values`](WorkQueueValue).
    async fn parse_initial_ids(
        &mut self,
        ctx: &DalContext,
        roots: Vec<DependentValueRoot>,
    ) -> AttributeValueResult<Vec<WorkQueueValue>> {
        let workspace_snapshot = ctx.workspace_snapshot()?;

        let mut values = Vec::with_capacity(roots.len());
        for root in roots {
            let root_ulid: Ulid = root.into();
            // It's possible that one or more of the initial ids provided by the enqueued
            // DependentValuesUpdate job may have been removed from the snapshot between when the
            // DVU job was created and when we're processing things now. This could happen if there
            // are other modifications to the snapshot before the DVU job starts executing, as the
            // job always operates on the current state of the change set's snapshot, not the state
            // at the time the job was created.
            if !workspace_snapshot.node_exists(root_ulid).await {
                debug!(%root_ulid, "missing node, skipping it in DependentValueGraph");
                continue;
            }

            self.inner.add_id(root_ulid.into());

            let node_weight = workspace_snapshot.get_node_weight(root_ulid).await?;

            match node_weight.into() {
                NodeWeightDiscriminants::AttributeValue => {
                    let initial_attribute_value_id: AttributeValueId = root_ulid.into();

                    if AttributeValue::is_set_by_dependent_function(ctx, initial_attribute_value_id)
                        .await?
                    {
                        self.values_that_need_to_execute_from_prototype_function
                            .insert(initial_attribute_value_id);
                    }

                    values.push(WorkQueueValue::Initial(initial_attribute_value_id));
                }
                NodeWeightDiscriminants::Secret => {
                    // If we are processing a secret, we don't want to add the secret itself. We
                    // only want to add the direct dependent attribute values. We also need to mark
                    // these values as needing to be processed because DVU will skip processing the
                    // first set of independent values. In most cases, we want that to happen.
                    // However, since the first set of independent values all have a secret as their
                    // parent and not an attribute value, they need to be processed in DVU.
                    let direct_dependents =
                        Secret::direct_dependent_attribute_values(ctx, root_ulid.into())
                            .await
                            .map_err(Box::new)?;
                    self.values_that_need_to_execute_from_prototype_function
                        .extend(direct_dependents.clone());
                    values.extend(
                        direct_dependents
                            .iter()
                            .map(|d| WorkQueueValue::Initial(*d)),
                    );
                }
                discrim => {
                    warn!(%discrim, %root_ulid, "skipping dependent value graph generation for unsupported node weight");
                }
            };
        }

        Ok(values)
    }

    async fn process_subscription_dependencies(
        &mut self,
        ctx: &DalContext,
        component_root_attribute_value_id: AttributeValueId,
        workspace_snapshot: &WorkspaceSnapshotSelector,
        controlling_funcs_for_component: &mut HashMap<
            ComponentId,
            HashMap<AttributeValueId, ControllingFuncData>,
        >,
    ) -> AttributeValueResult<Vec<WorkQueueValue>> {
        let current_component_id = self
            .component_id_for_av(ctx, component_root_attribute_value_id)
            .await?;

        let mut new_work_queue_values = vec![];

        for (edge, subscriber_apa_id, _subscribed_to_av_id) in workspace_snapshot
            .edges_directed(component_root_attribute_value_id, Direction::Incoming)
            .await?
        {
            if let EdgeWeightKind::ValueSubscription(path) = edge.kind {
                let subscription = ValueSubscription {
                    attribute_value_id: component_root_attribute_value_id,
                    path,
                };
                if let Some(resolved_av_id) = subscription.resolve(ctx).await? {
                    if let Some(subscriber_ap_id) = workspace_snapshot
                        .source_opt(subscriber_apa_id, EdgeWeightKind::PrototypeArgument)
                        .await?
                    {
                        for subscriber_av_ulid in workspace_snapshot
                            .incoming_sources_for_edge_weight_kind(
                                subscriber_ap_id,
                                EdgeWeightKindDiscriminants::Prototype,
                            )
                            .await?
                        {
                            let subscriber_av_id = subscriber_av_ulid.into();
                            let subscriber_component_id =
                                self.component_id_for_av(ctx, subscriber_av_id).await?;

                            let subscriber_controlling_av_id =
                                Self::get_controlling_attribute_value_id(
                                    ctx,
                                    subscriber_component_id,
                                    subscriber_av_id,
                                    controlling_funcs_for_component,
                                )
                                .await?;
                            let resolved_controlling_av_id =
                                Self::get_controlling_attribute_value_id(
                                    ctx,
                                    current_component_id,
                                    resolved_av_id,
                                    controlling_funcs_for_component,
                                )
                                .await?;

                            new_work_queue_values
                                .push(WorkQueueValue::Discovered(subscriber_av_id));

                            if subscriber_component_id != current_component_id
                                && !Component::should_data_flow_between_components(
                                    ctx,
                                    subscriber_component_id,
                                    current_component_id,
                                )
                                .await?
                            {
                                continue;
                            }

                            self.value_depends_on(
                                subscriber_controlling_av_id,
                                resolved_controlling_av_id,
                            );
                        }
                    }
                }
            }
        }

        Ok(new_work_queue_values)
    }

    /// Populate the [`DependentValueGraph`] using the provided [`values`](WorkQueueValue). This
    /// includes the entire parent tree of each value discovered, up to the root for every value's
    /// component, as well as any dependencies of values discovered while walking the graph
    /// (e.g. if a value's prototype takes one of the passed values as an input, we also need to
    /// find the values for the other inputs to the prototype, etc.). The graph also includes any
    /// inferred dependencies based on parentage, for example if a component gets its inputs from a
    /// parent frame, and that frame's output sockets change, we add those downstream input sockets
    /// to the graph.
    async fn populate_for_values(
        &mut self,
        ctx: &DalContext,
        values: Vec<WorkQueueValue>,
    ) -> AttributeValueResult<()> {
        let workspace_snapshot = ctx.workspace_snapshot()?;

        let mut controlling_funcs_for_component = HashMap::new();
        let mut work_queue = VecDeque::from_iter(values);
        let mut seen_list = HashSet::new();

        while let Some(current_attribute_value) = work_queue.pop_front() {
            if seen_list.contains(&current_attribute_value.id()) {
                continue;
            }
            seen_list.insert(current_attribute_value.id());

            let current_component_id = self
                .component_id_for_av(ctx, current_attribute_value.id())
                .await?;

            if AttributeValue::parent_id(ctx, current_attribute_value.id())
                .await?
                .is_none()
            {
                let new_work_queue_values = self
                    .process_subscription_dependencies(
                        ctx,
                        current_attribute_value.id(),
                        &workspace_snapshot,
                        &mut controlling_funcs_for_component,
                    )
                    .await?;
                work_queue.extend(new_work_queue_values);
            }

            // We need to be sure to only construct the graph out of
            // "controlling" values. However, controlled values can still be
            // inputs to functions, so we need to find the prototypes that
            // depend on them!
            let current_attribute_value_controlling_value_id =
                Self::get_controlling_attribute_value_id(
                    ctx,
                    current_component_id,
                    current_attribute_value.id(),
                    &mut controlling_funcs_for_component,
                )
                .await?;

            let value_is_for = AttributeValue::is_for(ctx, current_attribute_value.id()).await?;

            // Gather the Attribute Prototype Arguments that take the thing the
            // current value is for (prop, or socket) as an input
            let relevant_apas = {
                let attribute_prototype_argument_idxs = workspace_snapshot
                    .incoming_sources_for_edge_weight_kind(
                        value_is_for,
                        EdgeWeightKindDiscriminants::PrototypeArgumentValue,
                    )
                    .await?;

                let mut relevant_apas = vec![];
                for apa_idx in attribute_prototype_argument_idxs {
                    let apa = workspace_snapshot
                        .get_node_weight(apa_idx)
                        .await?
                        .get_attribute_prototype_argument_node_weight()?;
                    // If there are no targets, this is a schema-level attribute prototype argument
                    // TODO (jkeiser) the above comment is false; there can be apas on component-specific
                    // prototypes. Presumably we check this elsewhere.
                    relevant_apas.push(apa);
                }
                relevant_apas
            };

            match value_is_for {
                ValueIsFor::Prop(prop_id) => {
                    let prop = Prop::get_by_id(ctx, prop_id).await?;
                    if prop.kind == PropKind::Object {
                        // The children of an object might themselves be the
                        // input to another function, so we have to add them to
                        // the calculation of the graph, as we encounter them.
                        // We use `seen_list` to ensure we don't reprocess these
                        // values or the parents of these values.
                        for child_value_id in AttributeValue::get_child_av_ids_in_order(
                            ctx,
                            current_attribute_value.id(),
                        )
                        .await?
                        {
                            if !seen_list.contains(&child_value_id) {
                                work_queue.push_back(WorkQueueValue::ObjectChild(child_value_id));
                            }
                        }
                    }
                }
                // Check if this value is an output socket as the attribute
                // value might have implicit dependendcies based on the ancestry
                // (aka frames/nested frames) note: we filter out non-deleted
                // targets if the source component is set to be deleted
                ValueIsFor::OutputSocket(_) => {
                    let maybe_values_depend_on =
                        match ComponentOutputSocket::find_inferred_connections(
                            ctx,
                            current_attribute_value.id(),
                        )
                        .await
                        {
                            Ok(values) => values,
                            // When we first run dvu, the component type might not be set yet.
                            // In this case, we can assume there aren't downstream inputs that need to
                            // be queued up.
                            Err(ComponentError::ComponentMissingTypeValueMaterializedView(_)) => {
                                vec![]
                            }
                            Err(err) => return Err(AttributeValueError::Component(Box::new(err))),
                        };

                    for component_input_socket in maybe_values_depend_on {
                        // Both "deleted" and not deleted Components can feed data into
                        // "deleted" Components. **ONLY** not deleted Components can feed
                        // data into not deleted Components.
                        let destination_component_id = self
                            .component_id_for_av(ctx, current_attribute_value.id())
                            .await?;
                        if Component::should_data_flow_between_components(
                            ctx,
                            destination_component_id,
                            component_input_socket.component_id,
                        )
                        .await
                        .map_err(|e| AttributeValueError::Component(Box::new(e)))?
                        {
                            work_queue.push_back(WorkQueueValue::Discovered(
                                component_input_socket.attribute_value_id,
                            ));
                            self.value_depends_on(
                                component_input_socket.attribute_value_id,
                                current_attribute_value.id(),
                            );
                        }
                    }
                }
                _ => {}
            }

            // Find the values that are set by the prototype for the relevant
            // AttributePrototypeArguments, and declare that these values depend
            // on the value of the current value
            //
            // TODO: This code is very expensive, especially as the graph grows.
            for apa in relevant_apas {
                let prototype_id =
                    AttributePrototypeArgument::prototype_id(ctx, apa.id().into()).await?;

                let attribute_value_ids =
                    AttributePrototype::attribute_value_ids(ctx, prototype_id).await?;

                for attribute_value_id in attribute_value_ids {
                    let component_id = self.component_id_for_av(ctx, attribute_value_id).await?;
                    if component_id != current_component_id {
                        continue;
                    }

                    // If the input to this function is a value that is a child of another dynamic
                    // function, we should just depend on the controlling value in the dependency
                    // graph, since we can't guarantee that the controlled value won't be destroyed
                    // when "populating" nested value
                    let controlling_attribute_value_id = Self::get_controlling_attribute_value_id(
                        ctx,
                        component_id,
                        attribute_value_id,
                        &mut controlling_funcs_for_component,
                    )
                    .await?;

                    work_queue
                        .push_back(WorkQueueValue::Discovered(controlling_attribute_value_id));
                    self.inner.id_depends_on(
                        controlling_attribute_value_id,
                        current_attribute_value_controlling_value_id,
                    );
                }
            }

            // We have to be sure to process the parent of every value we
            // discover for any dependencies might have since changes to
            // children mean changes to parents. Suppose for example an
            // output socket depends on the domain prop, we have to be sure
            // we add that output socket to the graph if a child of domain
            // changes, because if a child of a value has changed, then the
            // view of that value to the leaves will also have changed.
            if let Some(parent_attribute_value_id) =
                AttributeValue::parent_id(ctx, current_attribute_value_controlling_value_id).await?
            {
                work_queue.push_back(WorkQueueValue::Discovered(parent_attribute_value_id));
            }
        }

        // Parent props always depend on their children.
        // Walking to the root for every value discovered
        // in the loop above ensures we construct a single
        // DVU connected graph for every tree of dependencies.
        for id in self.inner.all_ids() {
            let mut cursor = id;
            while let Some(parent_av_id) = AttributeValue::parent_id(ctx, cursor).await? {
                self.value_depends_on(parent_av_id, cursor);
                cursor = parent_av_id;
            }
        }

        Ok(())
    }

    async fn component_id_for_av(
        &mut self,
        ctx: &DalContext,
        value_id: AttributeValueId,
    ) -> AttributeValueResult<ComponentId> {
        Ok(match self.component_ids_for_av.entry(value_id) {
            btree_map::Entry::Vacant(vacant_entry) => {
                let component_id = AttributeValue::component_id(ctx, value_id).await?;
                vacant_entry.insert(component_id);
                component_id
            }
            btree_map::Entry::Occupied(occupied_entry) => *occupied_entry.get(),
        })
    }

    pub async fn debug_dot(&self, ctx: &DalContext, suffix: Option<&str>) {
        let mut is_for_map = HashMap::new();
        let mut component_name_map = HashMap::new();

        for attribute_value_id in self.inner.id_to_index_map().keys() {
            let component_id = AttributeValue::component_id(ctx, *attribute_value_id)
                .await
                .expect("get component id for av");
            let component = Component::get_by_id(ctx, component_id)
                .await
                .expect("get component");

            component_name_map.insert(
                *attribute_value_id,
                component
                    .name(ctx)
                    .await
                    .expect("able to get component name"),
            );
            let is_for = AttributeValue::is_for(ctx, *attribute_value_id)
                .await
                .expect("able to get value is for")
                .debug_info(ctx)
                .await
                .expect("able to get info for value is for");
            is_for_map.insert(*attribute_value_id, is_for);
        }

        let label_value_fn =
            move |_: &StableDiGraph<AttributeValueId, ()>,
                  (_, attribute_value_id): (NodeIndex, &AttributeValueId)| {
                let attribute_value_id = *attribute_value_id;
                let is_for = is_for_map.clone();

                let is_for_string = is_for
                    .clone()
                    .get(&attribute_value_id)
                    .map(ToOwned::to_owned)
                    .expect("is for exists for every value");

                let component_name = component_name_map
                    .get(&attribute_value_id)
                    .map(ToOwned::to_owned)
                    .expect("component name exists for every value");

                format!("label = \"{component_name}\n{attribute_value_id}\n{is_for_string}\"")
            };

        let dot = petgraph::dot::Dot::with_attr_getters(
            self.inner.graph(),
            &[
                petgraph::dot::Config::NodeNoLabel,
                petgraph::dot::Config::EdgeNoLabel,
            ],
            &|_, _| "label = \"\"".to_string(),
            &label_value_fn,
        );

        let filename_no_extension = format!("{}-{}", Ulid::new(), suffix.unwrap_or("depgraph"));
        let mut file = File::create(format!("/home/zacharyhamm/{filename_no_extension}.txt"))
            .expect("could not create file");

        file.write_all(format!("{dot:?}").as_bytes())
            .expect("could not write file");
        println!("dot output stored in file (filename without extension: {filename_no_extension})");
    }

    async fn get_controlling_attribute_value_id(
        ctx: &DalContext,
        current_component_id: ComponentId,
        current_attribute_value_id: AttributeValueId,
        controlling_funcs_for_component: &mut HashMap<
            ComponentId,
            HashMap<AttributeValueId, ControllingFuncData>,
        >,
    ) -> AttributeValueResult<AttributeValueId> {
        Ok(
            match controlling_funcs_for_component.entry(current_component_id) {
                Entry::Vacant(entry) => {
                    let controlling_func_data =
                        Component::list_av_controlling_func_ids_for_id(ctx, current_component_id)
                            .await
                            .map_err(Box::new)?;
                    let data = controlling_func_data
                        .get(&current_attribute_value_id)
                        .copied();

                    entry.insert(controlling_func_data);

                    data
                }
                Entry::Occupied(entry) => entry.get().get(&current_attribute_value_id).copied(),
            }
            .map(|func_data| func_data.av_id)
            // If nothing controls us, we control ourselves
            .unwrap_or(current_attribute_value_id),
        )
    }

    pub fn value_depends_on(
        &mut self,
        value_id: AttributeValueId,
        depends_on_id: AttributeValueId,
    ) {
        self.inner.id_depends_on(value_id, depends_on_id);
    }

    pub fn contains_value(&self, value_id: AttributeValueId) -> bool {
        self.inner.contains_id(value_id)
    }

    pub fn direct_dependencies_of(&self, value_id: AttributeValueId) -> Vec<AttributeValueId> {
        self.inner.direct_dependencies_of(value_id)
    }

    pub fn remove_value(&mut self, value_id: AttributeValueId) {
        self.inner.remove_id(value_id);
    }

    pub fn cycle_on_self(&mut self, value_id: AttributeValueId) {
        self.inner.cycle_on_self(value_id);
    }

    pub fn independent_values(&self) -> Vec<AttributeValueId> {
        self.inner.independent_ids()
    }

    pub fn all_value_ids(&self) -> Vec<AttributeValueId> {
        self.inner.all_ids()
    }

    /// Indicates whether the value needs to be processed. This is useful for determining when to
    /// filter or de-duplicate values when executing from their prototype functions. If the value is
    /// marked as needing to be processed, it likely needs to execute from its prototype function.
    pub fn values_needs_to_execute_from_prototype_function(
        &self,
        value_id: AttributeValueId,
    ) -> bool {
        self.values_that_need_to_execute_from_prototype_function
            .contains(&value_id)
    }
}
