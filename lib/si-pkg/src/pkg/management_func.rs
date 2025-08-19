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
    ManagementFuncSpec,
    node::PkgNode,
};

#[derive(Clone, Debug)]
pub struct SiPkgManagementFunc<'a> {
    func_unique_id: String,
    name: String,
    description: Option<String>,

    hash: Hash,
    source: Source<'a>,
}

impl<'a> SiPkgManagementFunc<'a> {
    pub fn from_graph(
        graph: &'a Graph<HashedNode<PkgNode>, ()>,
        node_idx: NodeIndex,
    ) -> PkgResult<Self> {
        let hashed_node = &graph[node_idx];
        let node = match hashed_node.inner() {
            PkgNode::ManagementFunc(node) => node.clone(),
            unexpected => {
                return Err(SiPkgError::UnexpectedPkgNodeType(
                    PkgNode::MANAGEMENT_FUNC_KIND_STR,
                    unexpected.node_kind_str(),
                ));
            }
        };

        Ok(Self {
            func_unique_id: node.func_unique_id,
            name: node.name,
            description: node.description,

            hash: hashed_node.hash(),
            source: Source::new(graph, node_idx),
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn func_unique_id(&self) -> &str {
        self.func_unique_id.as_str()
    }

    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    pub fn hash(&self) -> Hash {
        self.hash
    }

    pub fn source(&self) -> &Source<'a> {
        &self.source
    }
}

impl<'a> HasUniqueId for SiPkgManagementFunc<'a> {
    fn unique_id(&self) -> Option<&str> {
        Some(&self.func_unique_id)
    }
}

impl<'a> TryFrom<SiPkgManagementFunc<'a>> for ManagementFuncSpec {
    type Error = SiPkgError;

    fn try_from(value: SiPkgManagementFunc<'a>) -> Result<Self, Self::Error> {
        Ok(ManagementFuncSpec::builder()
            .func_unique_id(value.func_unique_id())
            .name(value.name().to_owned())
            .description(value.description().map(ToOwned::to_owned))
            .build()?)
    }
}
