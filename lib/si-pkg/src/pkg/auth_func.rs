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
    AuthenticationFuncSpec,
    HasUniqueId,
    node::PkgNode,
};

#[derive(Clone, Debug)]
pub struct SiPkgAuthFunc<'a> {
    func_unique_id: String,
    name: Option<String>,
    unique_id: Option<String>,
    deleted: bool,

    hash: Hash,
    source: Source<'a>,
}

impl<'a> SiPkgAuthFunc<'a> {
    pub fn from_graph(
        graph: &'a Graph<HashedNode<PkgNode>, ()>,
        node_idx: NodeIndex,
    ) -> PkgResult<Self> {
        let hashed_node = &graph[node_idx];
        let node = match hashed_node.inner() {
            PkgNode::AuthFunc(node) => node.clone(),
            unexpected => {
                return Err(SiPkgError::UnexpectedPkgNodeType(
                    PkgNode::AUTH_FUNC_KIND_STR,
                    unexpected.node_kind_str(),
                ));
            }
        };

        Ok(Self {
            name: node.name,
            func_unique_id: node.func_unique_id,
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

impl<'a> HasUniqueId for SiPkgAuthFunc<'a> {
    fn unique_id(&self) -> Option<&str> {
        self.unique_id()
    }
}

impl<'a> TryFrom<SiPkgAuthFunc<'a>> for AuthenticationFuncSpec {
    type Error = SiPkgError;

    fn try_from(value: SiPkgAuthFunc<'a>) -> Result<Self, Self::Error> {
        Ok(AuthenticationFuncSpec::builder()
            .deleted(value.deleted())
            .func_unique_id(value.func_unique_id())
            .name(value.name().map(ToOwned::to_owned))
            .unique_id(value.unique_id().map(ToOwned::to_owned))
            .deleted(value.deleted)
            .build()?)
    }
}
