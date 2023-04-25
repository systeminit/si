use object_tree::{Hash, HashedNode};
use petgraph::prelude::*;
use std::future::Future;

use url::Url;

use super::{PkgResult, SiPkgError, SiPkgLeafFunction, SiPkgProp, Source};

use crate::node::{PkgNode, PropChildNode, SchemaVariantChildNode};

#[derive(Clone, Debug)]
pub struct SiPkgSchemaVariant<'a> {
    name: String,
    link: Option<Url>,
    color: Option<String>,

    hash: Hash,

    source: Source<'a>,
}

impl<'a> SiPkgSchemaVariant<'a> {
    pub fn from_graph(
        graph: &'a Graph<HashedNode<PkgNode>, ()>,
        node_idx: NodeIndex,
    ) -> PkgResult<Self> {
        let schema_variant_hashed_node = &graph[node_idx];
        let schema_variant_node = match schema_variant_hashed_node.inner() {
            PkgNode::SchemaVariant(node) => node.clone(),
            unexpected => {
                return Err(SiPkgError::UnexpectedPkgNodeType(
                    PkgNode::SCHEMA_VARIANT_KIND_STR,
                    unexpected.node_kind_str(),
                ))
            }
        };

        let schema_variant = Self {
            name: schema_variant_node.name,
            link: schema_variant_node.link,
            color: schema_variant_node.color,
            hash: schema_variant_hashed_node.hash(),
            source: Source::new(graph, node_idx),
        };

        Ok(schema_variant)
    }

    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    pub fn link(&self) -> Option<&Url> {
        self.link.as_ref()
    }

    pub fn color(&self) -> Option<&str> {
        self.color.as_deref()
    }

    pub fn leaf_functions(&self) -> PkgResult<Vec<SiPkgLeafFunction>> {
        let child_idxs = self
            .source
            .graph
            .neighbors_directed(self.source.node_idx, Outgoing)
            .find(|node_idx| {
                matches!(
                    &self.source.graph[*node_idx].inner(),
                    PkgNode::SchemaVariantChild(SchemaVariantChildNode::LeafFunctions)
                )
            })
            .ok_or(SiPkgError::CategoryNotFound(
                SchemaVariantChildNode::LeafFunctions.kind_str(),
            ))?;

        let child_node_idxs: Vec<_> = self
            .source
            .graph
            .neighbors_directed(child_idxs, Outgoing)
            .collect();

        let mut leaf_functions = vec![];
        for child_idx in child_node_idxs {
            leaf_functions.push(SiPkgLeafFunction::from_graph(self.source.graph, child_idx)?);
        }

        Ok(leaf_functions)
    }

    fn prop_stack_from_source<I>(
        source: Source<'a>,
        node_idx: NodeIndex,
        parent_id: Option<I>,
    ) -> PkgResult<Vec<(SiPkgProp, Option<I>)>>
    where
        I: Copy,
    {
        Ok(
            match source
                .graph
                .neighbors_directed(node_idx, Outgoing)
                .find(|node_idx| {
                    matches!(
                        &source.graph[*node_idx].inner(),
                        PkgNode::PropChild(PropChildNode::Props)
                    )
                }) {
                Some(prop_child_idxs) => {
                    let child_node_idxs: Vec<_> = source
                        .graph
                        .neighbors_directed(prop_child_idxs, Outgoing)
                        .collect();

                    let mut entries = vec![];
                    for child_idx in child_node_idxs {
                        entries.push((SiPkgProp::from_graph(source.graph, child_idx)?, parent_id));
                    }

                    entries
                }
                None => vec![],
            },
        )
    }

    pub async fn visit_prop_tree<F, Fut, I, C, E>(
        &'a self,
        process_prop_fn: F,
        parent_id: Option<I>,
        context: &'a C,
    ) -> Result<(), E>
    where
        F: Fn(SiPkgProp<'a>, Option<I>, &'a C) -> Fut,
        Fut: Future<Output = Result<Option<I>, E>>,
        E: std::convert::From<SiPkgError>,
        I: Copy,
    {
        let domain_idxs = self
            .source
            .graph
            .neighbors_directed(self.source.node_idx, Outgoing)
            .find(|node_idx| {
                matches!(
                    &self.source.graph[*node_idx].inner(),
                    PkgNode::SchemaVariantChild(SchemaVariantChildNode::Domain)
                )
            })
            .ok_or(SiPkgError::CategoryNotFound(
                SchemaVariantChildNode::Domain.kind_str(),
            ))?;

        let mut child_node_idxs: Vec<_> = self
            .source
            .graph
            .neighbors_directed(domain_idxs, Outgoing)
            .collect();
        let domain_node_idx = match child_node_idxs.pop() {
            Some(idx) => idx,
            None => Err(SiPkgError::DomainPropNotFound(self.hash()))?,
        };
        if !child_node_idxs.is_empty() {
            Err(SiPkgError::DomainPropMultipleFound(self.hash()))?;
        }

        // Skip processing the domain prop as a `dal::SchemaVariant` already guarantees such a prop
        // has already been created. Rather, we will push all immediate children of the domain prop
        // to be ready for processing.
        let mut stack =
            Self::prop_stack_from_source(self.source.clone(), domain_node_idx, parent_id)?;

        while let Some((prop, parent_id)) = stack.pop() {
            let node_idx = prop.source().node_idx;
            let new_id = process_prop_fn(prop, parent_id, context).await?;

            stack.extend(Self::prop_stack_from_source(
                self.source.clone(),
                node_idx,
                new_id,
            )?);
        }

        Ok(())
    }

    pub fn hash(&self) -> Hash {
        self.hash
    }
}
