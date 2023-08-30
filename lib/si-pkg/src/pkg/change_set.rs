use object_tree::{Hash, HashedNode};
use petgraph::prelude::*;

use super::{PkgResult, SiPkgError, SiPkgSchema, Source};
use crate::SiPkgFunc;
use crate::{
    node::{ChangeSetChildNode, PkgNode},
    ChangeSetSpecStatus,
};

#[derive(Clone, Debug)]
pub struct SiPkgChangeSet<'a> {
    name: String,
    based_on_change_set: Option<String>,
    status: ChangeSetSpecStatus,

    hash: Hash,

    source: Source<'a>,
}

macro_rules! impl_change_set_children_from_graph {
    ($fn_name:ident, ChangeSetChildNode::$child_node:ident, $pkg_type:ident) => {
        pub fn $fn_name(&self) -> PkgResult<Vec<$pkg_type>> {
            let mut entries = vec![];
            if let Some(child_idxs) = self
                .source
                .graph
                .neighbors_directed(self.source.node_idx, Outgoing)
                .find(|node_idx| {
                    matches!(
                        &self.source.graph[*node_idx].inner(),
                        PkgNode::ChangeSetChild(ChangeSetChildNode::$child_node)
                    )
                })
            {
                let child_node_idxs: Vec<_> = self
                    .source
                    .graph
                    .neighbors_directed(child_idxs, Outgoing)
                    .collect();

                for child_idx in child_node_idxs {
                    entries.push($pkg_type::from_graph(self.source.graph, child_idx)?);
                }
            }

            Ok(entries)
        }
    };
}

impl<'a> SiPkgChangeSet<'a> {
    pub fn from_graph(
        graph: &'a Graph<HashedNode<PkgNode>, ()>,
        node_idx: NodeIndex,
    ) -> PkgResult<Self> {
        let change_set_hashed_node = &graph[node_idx];
        let change_set_node = match change_set_hashed_node.inner() {
            PkgNode::ChangeSet(node) => node.clone(),
            unexpected => {
                return Err(SiPkgError::UnexpectedPkgNodeType(
                    PkgNode::CHANGE_SET_KIND_STR,
                    unexpected.node_kind_str(),
                ));
            }
        };

        let change_set = Self {
            name: change_set_node.name,
            status: change_set_node.status,
            based_on_change_set: change_set_node.based_on_change_set,
            hash: change_set_hashed_node.hash(),
            source: Source::new(graph, node_idx),
        };

        Ok(change_set)
    }

    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    pub fn status(&self) -> ChangeSetSpecStatus {
        self.status
    }

    pub fn based_on_change_set(&self) -> Option<&str> {
        self.based_on_change_set.as_deref()
    }

    pub fn hash(&self) -> Hash {
        self.hash
    }

    impl_change_set_children_from_graph!(funcs, ChangeSetChildNode::Funcs, SiPkgFunc);
    impl_change_set_children_from_graph!(schemas, ChangeSetChildNode::Schemas, SiPkgSchema);
}
