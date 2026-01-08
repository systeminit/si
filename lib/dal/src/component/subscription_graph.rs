//! Component subscription graph.
//!
//! Creates a dependency graph by walking value subscription edges, so that we
//! can detect if the subscriptions would create a cycle
//!
use std::collections::BTreeSet;

use petgraph::visit::EdgeRef;
use si_id::ComponentId;

use super::{
    Component,
    ComponentResult,
};
use crate::{
    AttributePrototype,
    AttributeValue,
    DalContext,
    WorkspaceSnapshotGraphVCurrent,
    attribute::prototype::argument::AttributePrototypeArgument,
    dependency_graph::DependencyGraph,
    workspace_snapshot::{
        edge_weight::EdgeWeightKindDiscriminants,
        graph::{
            WorkspaceSnapshotGraphError,
            WorkspaceSnapshotGraphResult,
        },
        node_weight::{
            NodeWeight,
            category_node_weight::CategoryNodeKind,
        },
    },
};

pub struct SubscriptionGraph {
    inner: DependencyGraph<ComponentId>,
}

impl SubscriptionGraph {
    pub async fn new(ctx: &DalContext) -> ComponentResult<Self> {
        let components = Component::list_ids(ctx).await?;
        let mut inner = DependencyGraph::new();

        for component_id in components {
            for (_, apa_id) in Component::subscribers(ctx, component_id).await? {
                let prototype_id = AttributePrototypeArgument::prototype_id(ctx, apa_id).await?;
                for av_id in AttributePrototype::attribute_value_ids(ctx, prototype_id).await? {
                    let subscriber_component_id = AttributeValue::component_id(ctx, av_id).await?;
                    // Don't add self-subscriptions
                    if subscriber_component_id == component_id {
                        continue;
                    }
                    inner.id_depends_on(subscriber_component_id, component_id);
                }
            }
        }

        Ok(Self { inner })
    }

    /// Constructs a SubscriptionGraph directly from a
    /// WorkspaceSnapshotGraphVCurrent by examining the graph structure without
    /// using the DalContext (for the Validator)
    pub fn new_from_snapshot(
        snapshot: &WorkspaceSnapshotGraphVCurrent,
    ) -> WorkspaceSnapshotGraphResult<Self> {
        let mut inner = DependencyGraph::new();

        let Some((_, component_category_idx)) =
            snapshot.get_category_node(CategoryNodeKind::Component)?
        else {
            return Ok(Self { inner });
        };

        let component_idxs: Vec<_> = snapshot
            .targets(component_category_idx, EdgeWeightKindDiscriminants::Use)
            .filter(|idx| matches!(snapshot.get_node_weight(*idx), Ok(NodeWeight::Component(_))))
            .collect();

        for component_idx in component_idxs {
            let component_id: ComponentId = snapshot.get_node_weight(component_idx)?.id().into();

            let Some(root_av_idx) =
                snapshot.target_opt(component_idx, EdgeWeightKindDiscriminants::Root)?
            else {
                continue;
            };

            for edge_ref in
                snapshot.incoming_edges(root_av_idx, EdgeWeightKindDiscriminants::ValueSubscription)
            {
                let apa_idx = edge_ref.source();

                if !matches!(
                    snapshot.get_node_weight(apa_idx),
                    Ok(NodeWeight::AttributePrototypeArgument(_))
                ) {
                    continue;
                }

                let Some(prototype_idx) =
                    snapshot.source_opt(apa_idx, EdgeWeightKindDiscriminants::PrototypeArgument)?
                else {
                    continue;
                };

                for av_idx in
                    snapshot.sources(prototype_idx, EdgeWeightKindDiscriminants::Prototype)
                {
                    if let Some(subscriber_component_id) =
                        Self::component_id_for_av(snapshot, av_idx)?
                    {
                        // Don't add self-subscriptions
                        if subscriber_component_id == component_id {
                            continue;
                        }
                        inner.id_depends_on(subscriber_component_id, component_id);
                    }
                }
            }
        }

        Ok(Self { inner })
    }

    fn component_id_for_av(
        snapshot: &WorkspaceSnapshotGraphVCurrent,
        av_idx: petgraph::graph::NodeIndex,
    ) -> WorkspaceSnapshotGraphResult<Option<ComponentId>> {
        // First verify this is an AttributeValue
        let node_weight = snapshot.get_node_weight(av_idx)?;
        if !matches!(node_weight, NodeWeight::AttributeValue(_)) {
            return Ok(None);
        }

        let mut seen_list = BTreeSet::from([av_idx]);
        let mut current_idx = av_idx;
        loop {
            let parent_idx = snapshot
                .source_opt(current_idx, EdgeWeightKindDiscriminants::Contain)
                .ok()
                .flatten();

            match parent_idx {
                Some(parent_idx) => {
                    if seen_list.contains(&parent_idx) {
                        return Err(WorkspaceSnapshotGraphError::CycleInAttributeValueTree);
                    }
                    seen_list.insert(parent_idx);
                    current_idx = parent_idx;
                }
                None => break, // root!
            }
        }

        let Some(component_idx) = (match snapshot
            .source_opt(current_idx, EdgeWeightKindDiscriminants::Root)?
        {
            Some(component_idx) => Some(component_idx),
            None => snapshot.source_opt(current_idx, EdgeWeightKindDiscriminants::SocketValue)?,
        }) else {
            return Ok(None);
        };

        let component_weight = snapshot.get_node_weight(component_idx)?;
        if !matches!(component_weight, NodeWeight::Component(_)) {
            return Ok(None);
        }

        Ok(Some(component_weight.id().into()))
    }

    pub fn is_cyclic(&self) -> bool {
        self.inner.is_cyclic()
    }

    /// Returns a reference to the inner dependency graph.
    /// This is primarily intended for testing purposes.
    pub fn inner(&self) -> &DependencyGraph<ComponentId> {
        &self.inner
    }
}

const WHITELISTED_WORKSPACES: [&str; 8] = [
    "01J8K1NADNB211F3PGW0WVFNGZ",
    "01K6BVSETG8RWWFA9XAJ466DJH",
    "01K6ZNY9ZBKR7KZ2BGZYF6HNJS",
    "01K8KS1SMGNZ1YHEVN4YCPWRSM",
    "01K9D7ZN09EBNPBSSMVFG3NVTZ",
    "01KAE9XXM3FCFFN44E8C8XTMN1",
    "01KAEX523JDZ2H335FR853X684",
    "01KC00WFPBNZNMMYCE367JR98C",
];

/// Check if the current workspace snapshot has a subscription cycle,
/// but ignore whitelisted workspaces
pub async fn sub_cycle_check(ctx: &DalContext) -> ComponentResult<bool> {
    let workspace_id = ctx.workspace_pk()?;
    let workspace_id_str = workspace_id.to_string();

    Ok(
        if WHITELISTED_WORKSPACES.contains(&workspace_id_str.as_str()) {
            false
        } else {
            SubscriptionGraph::new(ctx).await?.is_cyclic()
        },
    )
}
