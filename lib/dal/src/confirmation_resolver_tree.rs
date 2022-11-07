use crate::{
    ComponentError, ComponentId, ConfirmationResolver, ConfirmationResolverId, DalContext, Edge,
    EdgeError, StandardModel, StandardModelError,
};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
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
    #[error("resolver {0} not found in tree")]
    ResolverIdNotFoundInTree(ConfirmationResolverId),
    #[error("resolver {0} not found in resolvers list")]
    ResolverIdNotFoundInResolvers(ConfirmationResolverId),
    #[error("missing parent confirmation of {0} (parent_id = {1})")]
    MissingParentConfirmation(ConfirmationResolverId, ConfirmationResolverId),
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
    resolvers: HashMap<ConfirmationResolverId, ConfirmationResolver>,
    tree: HashMap<ConfirmationResolverId, ConfirmationResolverTreeElement>,
}

impl ConfirmationResolverTree {
    pub async fn build(
        ctx: &DalContext,
        resolvers: Vec<ConfirmationResolver>,
    ) -> ConfirmationResolverTreeResult<Self> {
        let edges = Edge::list(ctx).await?;
        let mut tree = ConfirmationResolverTree {
            resolvers: resolvers.into_iter().map(|r| (*r.id(), r)).collect(),
            tree: Default::default(),
        };

        // O(N² * M) sucks, but the graph should be way too small to matter
        // We should move this to the database anyway
        for resolver in tree.resolvers.values() {
            let component_id = resolver.context().component_id;
            if component_id.is_none() {
                continue;
            }

            let mut parent_ids = Vec::new();
            let mut children_ids = Vec::new();

            for resolver in tree.resolvers.values() {
                let other_component_id = resolver.context().component_id;

                for edge in &edges {
                    let head_id: ComponentId = edge.head_object_id().into();
                    let tail_id: ComponentId = edge.tail_object_id().into();

                    if head_id == component_id && tail_id == other_component_id {
                        parent_ids.push(*resolver.id());
                    } else if tail_id == component_id && head_id == other_component_id {
                        children_ids.push(*resolver.id());
                    }
                }
            }

            tree.tree.insert(
                *resolver.id(),
                ConfirmationResolverTreeElement {
                    id: *resolver.id(),
                    parent_ids,
                    children_ids,
                },
            );
        }

        Ok(tree)
    }

    pub fn into_vec(mut self) -> Result<Vec<ConfirmationResolver>, ConfirmationResolverTreeError> {
        let mut processed_ids = HashSet::new();
        let mut sorted_resolver_ids = Vec::with_capacity(self.tree.len());
        let mut work_queue = VecDeque::new();
        let mut stuck_elements = HashSet::new();

        // List root components

        for el in self.tree.values().filter(|el| el.parent_ids.is_empty()) {
            work_queue.push_back(el);
        }

        'outer: while let Some(element) = work_queue.pop_front() {
            // Element might be child of multiple components so we should only process it once
            if processed_ids.contains(&element.id) {
                continue;
            }

            // Ensures component can be processed (all parents have been processed)
            for parent_id in &element.parent_ids {
                if !processed_ids.contains(parent_id) {
                    work_queue.push_back(element);

                    // Avoids infinite loop
                    stuck_elements.insert(element.id);
                    if stuck_elements.len() == work_queue.len() {
                        return Err(ConfirmationResolverTreeError::MissingParentConfirmation(
                            element.id, *parent_id,
                        ));
                    }

                    continue 'outer;
                }
            }

            let _ = stuck_elements.remove(&element.id);

            processed_ids.insert(element.id);
            sorted_resolver_ids.push(element.id);

            for child_id in &element.children_ids {
                if let Some(child) = self.tree.get(child_id) {
                    work_queue.push_back(child);
                } else {
                    return Err(ConfirmationResolverTreeError::ResolverIdNotFoundInTree(
                        *child_id,
                    ));
                }
            }
        }

        let mut resolvers = Vec::with_capacity(sorted_resolver_ids.len());
        for id in sorted_resolver_ids {
            if let Some(resolver) = self.resolvers.remove(&id) {
                resolvers.push(resolver);
            } else {
                return Err(ConfirmationResolverTreeError::ResolverIdNotFoundInResolvers(id));
            }
        }
        Ok(resolvers)
    }
}
