use object_tree::{Hash, HashedNode};
use petgraph::prelude::*;

use super::{PkgResult, SiPkgError, Source};

use crate::{
    node::{AttrFuncInputNode, PkgNode},
    AttrFuncInputSpec, AttrFuncInputSpecKind,
};

#[remain::sorted]
#[derive(Clone, Debug)]
pub enum SiPkgAttrFuncInput<'a> {
    InputSocket {
        name: String,
        socket_name: String,
        unique_id: Option<String>,
        deleted: bool,

        hash: Hash,
        source: Source<'a>,
    },
    OutputSocket {
        name: String,
        socket_name: String,
        unique_id: Option<String>,
        deleted: bool,

        hash: Hash,
        source: Source<'a>,
    },
    Prop {
        name: String,
        prop_path: String,
        unique_id: Option<String>,
        deleted: bool,

        hash: Hash,
        source: Source<'a>,
    },
}

// We want a version of this free of the restrictive lifetime on Source for the prop visitor
#[remain::sorted]
#[derive(Clone, Debug)]
pub enum SiPkgAttrFuncInputView {
    InputSocket {
        name: String,
        socket_name: String,
        unique_id: Option<String>,
        deleted: bool,
    },
    OutputSocket {
        name: String,
        socket_name: String,
        unique_id: Option<String>,
        deleted: bool,
    },
    Prop {
        name: String,
        prop_path: String,
        unique_id: Option<String>,
        deleted: bool,
    },
}

impl<'a> From<SiPkgAttrFuncInput<'a>> for SiPkgAttrFuncInputView {
    fn from(value: SiPkgAttrFuncInput<'a>) -> Self {
        match value {
            SiPkgAttrFuncInput::Prop {
                name,
                prop_path,
                unique_id,
                deleted,
                ..
            } => Self::Prop {
                name,
                prop_path,
                unique_id,
                deleted,
            },
            SiPkgAttrFuncInput::InputSocket {
                name,
                socket_name,
                unique_id,
                deleted,
                ..
            } => Self::InputSocket {
                name,
                socket_name,
                unique_id,
                deleted,
            },
            SiPkgAttrFuncInput::OutputSocket {
                name,
                socket_name,
                unique_id,
                deleted,
                ..
            } => Self::OutputSocket {
                name,
                socket_name,
                unique_id,
                deleted,
            },
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
            AttrFuncInputNode::Prop {
                name,
                prop_path,
                unique_id,
                deleted,
            } => Self::Prop {
                name,
                prop_path,
                unique_id,
                deleted,

                hash,
                source,
            },
            AttrFuncInputNode::InputSocket {
                name,
                socket_name,
                unique_id,
                deleted,
            } => Self::InputSocket {
                name,
                socket_name,
                unique_id,
                deleted,

                hash,
                source,
            },
            AttrFuncInputNode::OutputSocket {
                name,
                socket_name,
                unique_id,
                deleted,
            } => Self::OutputSocket {
                name,
                socket_name,
                unique_id,
                deleted,

                hash,
                source,
            },
        })
    }

    pub fn name(&self) -> &str {
        match self {
            SiPkgAttrFuncInput::Prop { name, .. }
            | SiPkgAttrFuncInput::InputSocket { name, .. }
            | SiPkgAttrFuncInput::OutputSocket { name, .. } => name.as_str(),
        }
    }
}

impl<'a> TryFrom<SiPkgAttrFuncInput<'a>> for AttrFuncInputSpec {
    type Error = SiPkgError;

    fn try_from(value: SiPkgAttrFuncInput<'a>) -> Result<Self, Self::Error> {
        let mut builder = AttrFuncInputSpec::builder();
        let (unique_id, deleted) = match &value {
            SiPkgAttrFuncInput::InputSocket {
                unique_id, deleted, ..
            }
            | SiPkgAttrFuncInput::OutputSocket {
                unique_id, deleted, ..
            }
            | SiPkgAttrFuncInput::Prop {
                unique_id, deleted, ..
            } => (unique_id.as_deref(), *deleted),
        };

        if let Some(unique_id) = unique_id {
            builder.unique_id(unique_id);
        }
        builder.deleted(deleted);

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
