use object_tree::{Hash, HashedNode};
use petgraph::prelude::*;

use super::{PkgResult, SiPkgError, Source};

use crate::node::{AttrFuncInputNode, PkgNode};

#[derive(Clone, Debug)]
pub enum SiPkgAttrFuncInput<'a> {
    Prop {
        name: String,
        prop_path: String,
        hash: Hash,
        source: Source<'a>,
    },
    InputSocket {
        name: String,
        socket_name: String,
        hash: Hash,
        source: Source<'a>,
    },
    OutputSocket {
        name: String,
        socket_name: String,
        hash: Hash,
        source: Source<'a>,
    },
}

impl<'a> SiPkgAttrFuncInput<'a> {
    pub fn from_graph(
        graph: &'a Graph<HashedNode<PkgNode>, ()>,
        node_idx: NodeIndex,
    ) -> PkgResult<Self> {
        let hashed_node = &graph[node_idx];
        let node = match hashed_node.inner() {
            PkgNode::AttrFuncInput(node) => node.clone(),
            unexpected => {
                return Err(SiPkgError::UnexpectedPkgNodeType(
                    PkgNode::ATTR_FUNC_INPUT_KIND_STR,
                    unexpected.node_kind_str(),
                ))
            }
        };
        let hash = hashed_node.hash();
        let source = Source::new(graph, node_idx);

        Ok(match node {
            AttrFuncInputNode::Prop { name, prop_path } => Self::Prop {
                name,
                prop_path,
                hash,
                source,
            },
            AttrFuncInputNode::InputSocket { name, socket_name } => Self::InputSocket {
                name,
                socket_name,
                hash,
                source,
            },
            AttrFuncInputNode::OutputSocket { name, socket_name } => Self::OutputSocket {
                name,
                socket_name,
                hash,
                source,
            },
        })
    }
}
