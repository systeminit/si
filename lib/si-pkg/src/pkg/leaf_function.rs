use object_tree::{
    Hash,
    HashedNode,
};
use petgraph::prelude::*;

use super::{
    PkgResult,
    SiPkgError,
    Source,
};
use crate::{
    HasUniqueId,
    LeafFunctionSpec,
    node::PkgNode,
    spec::{
        LeafInputLocation,
        LeafKind,
    },
};

#[derive(Clone, Debug)]
pub struct SiPkgLeafFunction<'a> {
    func_unique_id: String,
    leaf_kind: LeafKind,
    unique_id: Option<String>,
    deleted: bool,
    inputs: Vec<LeafInputLocation>,

    hash: Hash,
    source: Source<'a>,
}

impl<'a> SiPkgLeafFunction<'a> {
    pub fn from_graph(
        graph: &'a Graph<HashedNode<PkgNode>, ()>,
        node_idx: NodeIndex,
    ) -> PkgResult<Self> {
        let hashed_node = &graph[node_idx];
        let node = match hashed_node.inner() {
            PkgNode::LeafFunction(node) => node.clone(),
            unexpected => {
                return Err(SiPkgError::UnexpectedPkgNodeType(
                    PkgNode::LEAF_FUNCTION_KIND_STR,
                    unexpected.node_kind_str(),
                ));
            }
        };

        let mut inputs = vec![];
        if node.input_domain {
            inputs.push(LeafInputLocation::Domain);
        }
        if node.input_resource {
            inputs.push(LeafInputLocation::Resource);
        }
        if node.input_code {
            inputs.push(LeafInputLocation::Code);
        }
        if node.input_deleted_at {
            inputs.push(LeafInputLocation::DeletedAt);
        }
        if node.input_secret {
            inputs.push(LeafInputLocation::Secrets);
        }

        Ok(Self {
            func_unique_id: node.func_unique_id,
            leaf_kind: node.leaf_kind,
            unique_id: node.unique_id,
            deleted: node.deleted,
            inputs,

            hash: hashed_node.hash(),
            source: Source::new(graph, node_idx),
        })
    }

    pub fn func_unique_id(&self) -> &str {
        self.func_unique_id.as_str()
    }

    pub fn leaf_kind(&self) -> LeafKind {
        self.leaf_kind
    }

    pub fn unique_id(&self) -> Option<&str> {
        self.unique_id.as_deref()
    }

    pub fn deleted(&self) -> bool {
        self.deleted
    }

    pub fn inputs(&self) -> &[LeafInputLocation] {
        &self.inputs
    }

    pub fn hash(&self) -> Hash {
        self.hash
    }

    pub fn source(&self) -> &Source<'a> {
        &self.source
    }
}

impl<'a> HasUniqueId for SiPkgLeafFunction<'a> {
    fn unique_id(&self) -> Option<&str> {
        self.unique_id()
    }
}

impl<'a> TryFrom<SiPkgLeafFunction<'a>> for LeafFunctionSpec {
    type Error = SiPkgError;

    fn try_from(value: SiPkgLeafFunction<'a>) -> Result<Self, Self::Error> {
        Ok(LeafFunctionSpec::builder()
            .func_unique_id(value.func_unique_id)
            .leaf_kind(value.leaf_kind)
            .unique_id(value.unique_id)
            .deleted(value.deleted)
            .inputs(value.inputs)
            .build()?)
    }
}
