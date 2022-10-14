use crate::{
    ComponentError, ComponentId, ConfirmationResolver, ConfirmationResolverId, DalContext, Edge,
    EdgeError, StandardModel, StandardModelError,
};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use telemetry::prelude::*;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfirmationResolverTreeError {
    #[error(transparent)]
    Component(#[from] ComponentError),
    #[error(transparent)]
    Edge(#[from] EdgeError),
    #[error(transparent)]
    StandardModel(#[from] StandardModelError),
    #[error("component {0} not found")]
    ComponentNotFound(ComponentId),
    #[error("no resolvers available")]
    Empty,
}

pub type ConfirmationResolverTreeResult<T> = Result<T, ConfirmationResolverTreeError>;

// This is very inneficient and should probably be done in PG

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct ConfirmationResolverTreeElement {
    id: ConfirmationResolverId,
    parent_ids: Vec<ConfirmationResolverId>,
    children_ids: Vec<ConfirmationResolverId>,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct ConfirmationResolverTree {
    resolvers: Vec<ConfirmationResolver>,
    tree: Vec<ConfirmationResolverTreeElement>,
}

impl ConfirmationResolverTree {
    pub async fn build(
        ctx: &DalContext,
        resolvers: Vec<ConfirmationResolver>,
    ) -> ConfirmationResolverTreeResult<Self> {
        let edges = Edge::list(ctx).await?;
        let mut tree = ConfirmationResolverTree {
            resolvers,
            tree: Vec::new(),
        };

        for resolver in &tree.resolvers {
            let component_id = resolver.context().component_id;
            if component_id.is_none() {
                continue;
            }

            tree.tree.push(ConfirmationResolverTreeElement {
                id: *resolver.id(),
                parent_ids: tree
                    .resolvers
                    .iter()
                    .filter(|resolver| {
                        edges
                            .iter()
                            .filter(|c| ComponentId::from(c.head_object_id()) == component_id)
                            .any(|c| {
                                ComponentId::from(c.tail_object_id())
                                    == resolver.context().component_id
                            })
                    })
                    .map(|r| *r.id())
                    .collect(),
                children_ids: tree
                    .resolvers
                    .iter()
                    .filter(|resolver| {
                        edges
                            .iter()
                            .filter(|c| ComponentId::from(c.tail_object_id()) == component_id)
                            .any(|c| {
                                ComponentId::from(c.head_object_id())
                                    == resolver.context().component_id
                            })
                    })
                    .map(|r| *r.id())
                    .collect(),
            })
        }

        Ok(tree)
    }

    // TODO: be more efficient here with allocations (inline into_vec)
    pub fn flatten(trees: Vec<Self>) -> Vec<ConfirmationResolver> {
        let mut resolvers = Vec::new();
        for tree in trees {
            resolvers.extend(tree.into_vec());
        }
        resolvers
    }

    pub fn into_vec(self) -> Vec<ConfirmationResolver> {
        let mut sorted_resolver_ids = Vec::with_capacity(self.tree.len());
        let mut work_queue = VecDeque::new();

        for el in self.tree.iter().filter(|el| el.parent_ids.is_empty()) {
            work_queue.push_back(el);
        }

        while let Some(child) = work_queue.pop_front() {
            sorted_resolver_ids.push(child.id);
            work_queue.extend(
                self.tree
                    .iter()
                    .filter(|el| child.children_ids.contains(&el.id)),
            );
        }
        self.resolvers
            .into_iter()
            .filter(|resolver| sorted_resolver_ids.contains(resolver.id()))
            .collect()
    }
}
