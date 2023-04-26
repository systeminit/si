use object_tree::{Hash, HashedNode};
use petgraph::prelude::*;

use super::{PkgResult, SiPkgError, SiPkgValidation, Source};

use crate::node::{PkgNode, PropChildNode, PropNode};

#[derive(Clone, Debug)]
pub enum SiPkgProp<'a> {
    String {
        name: String,
        hash: Hash,
        source: Source<'a>,
    },
    Number {
        name: String,
        hash: Hash,
        source: Source<'a>,
    },
    Boolean {
        name: String,
        hash: Hash,
        source: Source<'a>,
    },
    Map {
        name: String,
        hash: Hash,
        source: Source<'a>,
    },
    Array {
        name: String,
        hash: Hash,
        source: Source<'a>,
    },
    Object {
        name: String,
        hash: Hash,
        source: Source<'a>,
    },
}

impl<'a> SiPkgProp<'a> {
    pub fn validations(&self) -> PkgResult<Vec<SiPkgValidation>> {
        Ok(match self {
            SiPkgProp::Map { source, .. }
            | SiPkgProp::Array { source, .. }
            | SiPkgProp::String { source, .. }
            | SiPkgProp::Number { source, .. }
            | SiPkgProp::Object { source, .. }
            | SiPkgProp::Boolean { source, .. } => {
                let mut validations = vec![];

                if let Some(validation_child_idxs) = source
                    .graph
                    .neighbors_directed(source.node_idx, Outgoing)
                    .find(|node_idx| {
                        matches!(
                            &source.graph[*node_idx].inner(),
                            PkgNode::PropChild(PropChildNode::Validations)
                        )
                    })
                {
                    let child_node_idxs: Vec<_> = source
                        .graph
                        .neighbors_directed(validation_child_idxs, Outgoing)
                        .collect();

                    for child_idx in child_node_idxs {
                        validations.push(SiPkgValidation::from_graph(source.graph, child_idx)?);
                    }
                }

                validations
            }
        })
    }

    pub fn from_graph(
        graph: &'a Graph<HashedNode<PkgNode>, ()>,
        node_idx: NodeIndex,
    ) -> PkgResult<Self> {
        let prop_hashed_node = &graph[node_idx];
        let prop_node = match prop_hashed_node.inner() {
            PkgNode::Prop(node) => node.clone(),
            unexpected => {
                return Err(SiPkgError::UnexpectedPkgNodeType(
                    PkgNode::PROP_KIND_STR,
                    unexpected.node_kind_str(),
                ))
            }
        };

        let hash = prop_hashed_node.hash();
        let source = Source::new(graph, node_idx);

        Ok(match prop_node {
            PropNode::String { name } => Self::String { name, hash, source },
            PropNode::Integer { name } => Self::Number { name, hash, source },
            PropNode::Boolean { name } => Self::Boolean { name, hash, source },
            PropNode::Map { name } => Self::Map { name, hash, source },
            PropNode::Array { name } => Self::Array { name, hash, source },
            PropNode::Object { name } => Self::Object { name, hash, source },
        })
    }

    pub fn name(&self) -> &str {
        match self {
            Self::String { name, .. }
            | Self::Number { name, .. }
            | Self::Boolean { name, .. }
            | Self::Map { name, .. }
            | Self::Array { name, .. }
            | Self::Object { name, .. } => name,
        }
    }

    pub fn hash(&self) -> Hash {
        match self {
            Self::String { hash, .. }
            | Self::Number { hash, .. }
            | Self::Boolean { hash, .. }
            | Self::Map { hash, .. }
            | Self::Array { hash, .. }
            | Self::Object { hash, .. } => *hash,
        }
    }

    pub fn source(&self) -> &Source<'a> {
        match self {
            Self::String { source, .. }
            | Self::Number { source, .. }
            | Self::Boolean { source, .. }
            | Self::Map { source, .. }
            | Self::Array { source, .. }
            | Self::Object { source, .. } => source,
        }
    }
}
