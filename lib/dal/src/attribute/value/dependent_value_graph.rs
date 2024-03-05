use petgraph::prelude::*;
use std::collections::{hash_map::Entry, HashMap, VecDeque};
use tokio::{fs::File, io::AsyncWriteExt};
use ulid::Ulid;

use crate::{
    attribute::{
        prototype::{argument::AttributePrototypeArgument, AttributePrototype},
        value::ValueIsFor,
    },
    workspace_snapshot::edge_weight::EdgeWeightKindDiscriminants,
    DalContext, Prop,
};

use super::{AttributeValue, AttributeValueId, AttributeValueResult};

#[derive(Debug, Clone)]
pub struct DependentValueGraph {
    graph: StableDiGraph<AttributeValueId, ()>,
    id_to_index_map: HashMap<AttributeValueId, NodeIndex>,
}

impl Default for DependentValueGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl DependentValueGraph {
    pub fn new() -> Self {
        Self {
            id_to_index_map: HashMap::new(),
            graph: StableGraph::new(),
        }
    }

    /// Construct a [`DependentValueGraph`] of all the [`AttributeValueId`] whose values depend on
    /// the value of the values in [`values`]. This includes the entire parent tree of each value
    /// discovered, up to the root for every value's component, as well as any dependencies of
    /// values discovered while walking the graph (e.g., if a value's prototype takes one of the
    /// passed values as an input, we also need to find the values for the other inputs to the
    /// prototype, etc.).
    pub async fn for_values(
        ctx: &DalContext,
        values: Vec<AttributeValueId>,
    ) -> AttributeValueResult<Self> {
        let mut dependent_value_graph = Self::new();

        let mut work_queue = VecDeque::from_iter(values);
        while let Some(current_attribute_value_id) = work_queue.pop_front() {
            let current_component_id =
                AttributeValue::component_id(ctx, current_attribute_value_id).await?;
            let data_source_id: Ulid = AttributeValue::is_for(ctx, current_attribute_value_id)
                .await?
                .into();

            // Gather the Attribute Prototype Arguments that take the thing the
            // current value is for (prop, or socket) as an input
            let relevant_apas = {
                let workspace_snapshot = ctx.workspace_snapshot()?.read().await;

                let attribute_prototype_argument_idxs = workspace_snapshot
                    .incoming_sources_for_edge_weight_kind(
                        data_source_id,
                        EdgeWeightKindDiscriminants::PrototypeArgumentValue,
                    )?;

                let mut relevant_apas = vec![];
                for apa_idx in attribute_prototype_argument_idxs {
                    let apa = workspace_snapshot
                        .get_node_weight(apa_idx)?
                        .get_attribute_prototype_argument_node_weight()?;

                    match apa.targets() {
                        // If there are no targets, this is a schema-level attribute prototype argument
                        None => relevant_apas.push(apa),
                        Some(targets) => {
                            if targets.source_component_id == current_component_id {
                                relevant_apas.push(apa)
                            }
                        }
                    }
                }
                relevant_apas
            };

            // Find the values that are set by the prototype for the relevant
            // AttributePrototypeArguments, and declare that these values depend
            // on the value of the current value
            for apa in relevant_apas {
                let prototype_id =
                    AttributePrototypeArgument::prototype_id_for_argument_id(ctx, apa.id().into())
                        .await?;

                let attribute_value_ids =
                    AttributePrototype::attribute_value_ids(ctx, prototype_id).await?;

                for attribute_value_id in attribute_value_ids {
                    let filter_component_id = match apa.targets() {
                        None => current_component_id,
                        Some(targets) => targets.destination_component_id,
                    };
                    let component_id =
                        AttributeValue::component_id(ctx, attribute_value_id).await?;
                    if component_id == filter_component_id {
                        work_queue.push_back(attribute_value_id);
                        dependent_value_graph
                            .value_depends_on(attribute_value_id, current_attribute_value_id);
                    }
                }
            }

            // Also walk up to the root and ensure each parent value of the
            // current value depends on its child (down to the current value)
            // This ensures that we update the materialized views of the parent
            // tree
            if let Some(parent_attribute_value_id) =
                AttributeValue::parent_attribute_value_id(ctx, current_attribute_value_id).await?
            {
                work_queue.push_back(parent_attribute_value_id);
                dependent_value_graph
                    .value_depends_on(parent_attribute_value_id, current_attribute_value_id);
            }
        }

        Ok(dependent_value_graph)
    }

    pub async fn debug_dot(&self, ctx: &DalContext, suffix: Option<&str>) {
        let mut is_for_map = HashMap::new();

        for attribute_value_id in self.id_to_index_map.keys() {
            let is_for: String = match AttributeValue::is_for(ctx, *attribute_value_id)
                .await
                .expect("able to get value is for")
            {
                ValueIsFor::Prop(prop_id) => format!(
                    "prop = {}",
                    Prop::path_by_id(ctx, prop_id)
                        .await
                        .expect("able to get prop path")
                        .with_replaced_sep("/"),
                ),
                ValueIsFor::OutputSocket(_) => "output socket".into(),
                ValueIsFor::InputSocket(_) => "input socket".into(),
            };
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

                format!("label = \"{}\n{}\"", attribute_value_id, is_for_string)
            };

        let dot = petgraph::dot::Dot::with_attr_getters(
            &self.graph,
            &[
                petgraph::dot::Config::NodeNoLabel,
                petgraph::dot::Config::EdgeNoLabel,
            ],
            &|_, _| "label = \"\"".to_string(),
            &label_value_fn,
        );

        let filename_no_extension = format!(
            "{}-{}",
            Ulid::new().to_string(),
            suffix.unwrap_or("depgraph")
        );
        let mut file = File::create(format!("/home/zacharyhamm/{filename_no_extension}.txt"))
            .await
            .expect("could not create file");

        file.write_all(format!("{dot:?}").as_bytes())
            .await
            .expect("could not write file");
        println!("dot output stored in file (filename without extension: {filename_no_extension})");
    }

    pub fn add_value(&mut self, value_id: AttributeValueId) -> NodeIndex {
        match self.id_to_index_map.entry(value_id) {
            Entry::Vacant(entry) => {
                let node_idx = self.graph.add_node(value_id);
                entry.insert(node_idx);

                node_idx
            }
            Entry::Occupied(entry) => *entry.get(),
        }
    }

    pub fn value_depends_on(
        &mut self,
        value_id: AttributeValueId,
        depends_on_id: AttributeValueId,
    ) {
        let value_idx = self.add_value(value_id);
        let depends_on_idx = self.add_value(depends_on_id);

        self.graph.add_edge(value_idx, depends_on_idx, ());
    }

    pub fn contains_value(&self, value_id: AttributeValueId) -> bool {
        self.id_to_index_map.get(&value_id).is_some()
    }

    pub fn direct_dependencies_of(&self, value_id: AttributeValueId) -> Vec<AttributeValueId> {
        match self.id_to_index_map.get(&value_id) {
            None => vec![],
            Some(value_idx) => self
                .graph
                .edges_directed(*value_idx, Outgoing)
                .filter_map(|edge_ref| self.graph.node_weight(edge_ref.target()).copied())
                .collect(),
        }
    }

    pub fn remove_value(&mut self, value_id: AttributeValueId) {
        if let Some(node_idx) = self.id_to_index_map.remove(&value_id) {
            self.graph.remove_node(node_idx);
        }
    }

    pub fn cycle_on_self(&mut self, value_id: AttributeValueId) {
        if let Some(node_idx) = self.id_to_index_map.get(&value_id) {
            self.graph.add_edge(*node_idx, *node_idx, ());
        }
    }

    pub fn independent_values(&self) -> Vec<AttributeValueId> {
        self.graph
            .externals(Outgoing)
            .filter_map(|node_idx| self.graph.node_weight(node_idx).copied())
            .collect()
    }

    pub fn into_graph(self) -> StableDiGraph<AttributeValueId, ()> {
        self.graph
    }
}
