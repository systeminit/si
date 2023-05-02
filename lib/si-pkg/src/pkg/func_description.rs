use object_tree::{Hash, HashedNode};
use petgraph::prelude::*;

use super::{PkgResult, SiPkgError, Source};

use crate::{
    FuncDescriptionSpec,
    {node::PkgNode, spec::FuncUniqueId},
};

#[derive(Clone, Debug)]
pub struct SiPkgFuncDescription<'a> {
    func_unique_id: FuncUniqueId,
    contents: serde_json::Value,
    hash: Hash,
    source: Source<'a>,
}

impl<'a> SiPkgFuncDescription<'a> {
    pub fn from_graph(
        graph: &'a Graph<HashedNode<PkgNode>, ()>,
        node_idx: NodeIndex,
    ) -> PkgResult<Self> {
        let hashed_node = &graph[node_idx];
        let node = match hashed_node.inner() {
            PkgNode::FuncDescription(node) => node.clone(),
            unexpected => {
                return Err(SiPkgError::UnexpectedPkgNodeType(
                    PkgNode::FUNC_DESCRIPTION_KIND_STR,
                    unexpected.node_kind_str(),
                ))
            }
        };

        Ok(Self {
            func_unique_id: node.func_unique_id,
            contents: node.contents,
            hash: hashed_node.hash(),
            source: Source::new(graph, node_idx),
        })
    }

    pub fn func_unique_id(&self) -> FuncUniqueId {
        self.func_unique_id
    }

    pub fn contents(&self) -> &serde_json::Value {
        &self.contents
    }

    pub fn hash(&self) -> Hash {
        self.hash
    }

    pub fn source(&self) -> &Source<'a> {
        &self.source
    }
}

impl<'a> TryFrom<SiPkgFuncDescription<'a>> for FuncDescriptionSpec {
    type Error = SiPkgError;

    fn try_from(value: SiPkgFuncDescription<'a>) -> Result<Self, Self::Error> {
        Ok(FuncDescriptionSpec::builder()
            .func_unique_id(value.func_unique_id)
            .contents(value.contents)
            .build()?)
    }
}
