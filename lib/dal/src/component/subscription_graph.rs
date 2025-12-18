//! Component subscription graph.
//!
//! Creates a dependency graph by walking value subscription edges, so that we
//! can detect if the subscriptions would create a cycle
//!
use si_id::ComponentId;

use super::{
    Component,
    ComponentResult,
};
use crate::{
    AttributePrototype,
    AttributeValue,
    DalContext,
    attribute::prototype::argument::AttributePrototypeArgument,
    dependency_graph::DependencyGraph,
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

    pub fn is_cyclic(&self) -> bool {
        self.inner.is_cyclic()
    }
}
