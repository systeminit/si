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
    ActionFuncSpec,
    ActionFuncSpecKind,
    HasUniqueId,
    node::PkgNode,
};

#[derive(Clone, Debug)]
pub struct SiPkgActionFunc<'a> {
    func_unique_id: String,
    name: Option<String>,
    kind: ActionFuncSpecKind,
    unique_id: Option<String>,
    deleted: bool,

    hash: Hash,
    source: Source<'a>,
}

impl<'a> SiPkgActionFunc<'a> {
    pub fn from_graph(
        graph: &'a Graph<HashedNode<PkgNode>, ()>,
        node_idx: NodeIndex,
    ) -> PkgResult<Self> {
        let hashed_node = &graph[node_idx];
        let node = match hashed_node.inner() {
            PkgNode::ActionFunc(node) => node.clone(),
            unexpected => {
                return Err(SiPkgError::UnexpectedPkgNodeType(
                    PkgNode::ACTION_FUNC_KIND_STR,
                    unexpected.node_kind_str(),
                ));
            }
        };

        Ok(Self {
            name: node.name,
            func_unique_id: node.func_unique_id,
            kind: node.kind,
            unique_id: node.unique_id,
            deleted: node.deleted,

            hash: hashed_node.hash(),
            source: Source::new(graph, node_idx),
        })
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub fn func_unique_id(&self) -> &str {
        self.func_unique_id.as_str()
    }

    pub fn kind(&self) -> ActionFuncSpecKind {
        self.kind
    }

    pub fn unique_id(&self) -> Option<&str> {
        self.unique_id.as_deref()
    }

    pub fn deleted(&self) -> bool {
        self.deleted
    }

    pub fn hash(&self) -> Hash {
        self.hash
    }

    pub fn source(&self) -> &Source<'a> {
        &self.source
    }
}

impl<'a> HasUniqueId for SiPkgActionFunc<'a> {
    fn unique_id(&self) -> Option<&str> {
        self.unique_id()
    }
}

impl<'a> TryFrom<SiPkgActionFunc<'a>> for ActionFuncSpec {
    type Error = SiPkgError;

    fn try_from(value: SiPkgActionFunc<'a>) -> Result<Self, Self::Error> {
        Ok(ActionFuncSpec::builder()
            .func_unique_id(value.func_unique_id())
            .name(value.name().map(ToOwned::to_owned))
            .kind(value.kind())
            .unique_id(value.unique_id().map(ToOwned::to_owned))
            .deleted(value.deleted())
            .build()?)
    }
}
