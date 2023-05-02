use object_tree::{Hash, HashedNode};
use petgraph::prelude::*;

use super::{PkgResult, SiPkgError, Source};

use crate::{
    node::{AttrFuncInputNode, PkgNode},
    AttrFuncInputSpec, AttrFuncInputSpecKind,
};

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

// We want a version of this free of the restrictive lifetime on Source for the prop visitor
#[derive(Clone, Debug)]
pub enum SiPkgAttrFuncInputView {
    Prop { name: String, prop_path: String },
    InputSocket { name: String, socket_name: String },
    OutputSocket { name: String, socket_name: String },
}

impl<'a> From<SiPkgAttrFuncInput<'a>> for SiPkgAttrFuncInputView {
    fn from(value: SiPkgAttrFuncInput<'a>) -> Self {
        match value {
            SiPkgAttrFuncInput::Prop {
                name, prop_path, ..
            } => Self::Prop { name, prop_path },
            SiPkgAttrFuncInput::InputSocket {
                name, socket_name, ..
            } => Self::InputSocket { name, socket_name },
            SiPkgAttrFuncInput::OutputSocket {
                name, socket_name, ..
            } => Self::OutputSocket { name, socket_name },
        }
    }
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

impl<'a> TryFrom<SiPkgAttrFuncInput<'a>> for AttrFuncInputSpec {
    type Error = SiPkgError;

    fn try_from(value: SiPkgAttrFuncInput<'a>) -> Result<Self, Self::Error> {
        let mut builder = AttrFuncInputSpec::builder();
        match value {
            SiPkgAttrFuncInput::Prop {
                name, prop_path, ..
            } => {
                builder.kind(AttrFuncInputSpecKind::Prop);
                builder.name(name);
                builder.prop_path(prop_path);
            }
            SiPkgAttrFuncInput::InputSocket {
                name, socket_name, ..
            } => {
                builder.kind(AttrFuncInputSpecKind::InputSocket);
                builder.name(name);
                builder.socket_name(socket_name);
            }
            SiPkgAttrFuncInput::OutputSocket {
                name, socket_name, ..
            } => {
                builder.kind(AttrFuncInputSpecKind::OutputSocket);
                builder.name(name);
                builder.socket_name(socket_name);
            }
        }

        Ok(builder.build()?)
    }
}
