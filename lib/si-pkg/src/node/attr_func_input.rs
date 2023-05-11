use std::{
    io::{BufRead, Write},
    str::FromStr,
};

use object_tree::{
    read_key_value_line, write_key_value_line, GraphError, NameStr, NodeChild, NodeKind,
    NodeWithChildren, ReadBytes, WriteBytes,
};

use crate::{AttrFuncInputSpec, AttrFuncInputSpecKind};

use super::PkgNode;

const KEY_KIND_STR: &str = "kind";
const KEY_NAME_STR: &str = "name";
const KEY_PROP_PATH_STR: &str = "prop_path";
const KEY_SOCKET_NAME_STR: &str = "socket_name";

#[remain::sorted]
#[derive(Clone, Debug)]
pub enum AttrFuncInputNode {
    InputSocket { name: String, socket_name: String },
    OutputSocket { name: String, socket_name: String },
    Prop { name: String, prop_path: String },
}

impl NameStr for AttrFuncInputNode {
    fn name(&self) -> &str {
        match self {
            Self::Prop { name, .. }
            | Self::InputSocket { name, .. }
            | Self::OutputSocket { name, .. } => name,
        }
    }
}

impl AttrFuncInputNode {
    fn kind_str(&self) -> &str {
        match self {
            Self::Prop { .. } => AttrFuncInputSpecKind::Prop.as_ref(),
            Self::InputSocket { .. } => AttrFuncInputSpecKind::InputSocket.as_ref(),
            Self::OutputSocket { .. } => AttrFuncInputSpecKind::OutputSocket.as_ref(),
        }
    }
}

impl WriteBytes for AttrFuncInputNode {
    fn write_bytes<W: Write>(&self, writer: &mut W) -> Result<(), GraphError> {
        write_key_value_line(writer, KEY_NAME_STR, self.name())?;
        write_key_value_line(writer, KEY_KIND_STR, self.kind_str())?;

        match self {
            Self::OutputSocket { socket_name, .. } | Self::InputSocket { socket_name, .. } => {
                write_key_value_line(writer, KEY_SOCKET_NAME_STR, socket_name)?;
            }
            Self::Prop { prop_path, .. } => {
                write_key_value_line(writer, KEY_PROP_PATH_STR, prop_path)?;
            }
        }

        Ok(())
    }
}

impl ReadBytes for AttrFuncInputNode {
    fn read_bytes<R: BufRead>(reader: &mut R) -> Result<Self, GraphError>
    where
        Self: std::marker::Sized,
    {
        let name = read_key_value_line(reader, KEY_NAME_STR)?;
        let kind_str = read_key_value_line(reader, KEY_KIND_STR)?;

        let kind = AttrFuncInputSpecKind::from_str(&kind_str).map_err(GraphError::parse)?;
        Ok(match kind {
            AttrFuncInputSpecKind::Prop => {
                let prop_path = read_key_value_line(reader, KEY_PROP_PATH_STR)?;
                Self::Prop { name, prop_path }
            }
            AttrFuncInputSpecKind::InputSocket => {
                let socket_name = read_key_value_line(reader, KEY_SOCKET_NAME_STR)?;
                Self::InputSocket { name, socket_name }
            }
            AttrFuncInputSpecKind::OutputSocket => {
                let socket_name = read_key_value_line(reader, KEY_SOCKET_NAME_STR)?;
                Self::OutputSocket { name, socket_name }
            }
        })
    }
}

impl NodeChild for AttrFuncInputSpec {
    type NodeType = PkgNode;

    fn as_node_with_children(&self) -> NodeWithChildren<Self::NodeType> {
        NodeWithChildren::new(
            NodeKind::Leaf,
            Self::NodeType::AttrFuncInput(match self {
                AttrFuncInputSpec::Prop { name, prop_path } => AttrFuncInputNode::Prop {
                    name: name.clone(),
                    prop_path: prop_path.clone(),
                },
                AttrFuncInputSpec::InputSocket { name, socket_name } => {
                    AttrFuncInputNode::InputSocket {
                        name: name.clone(),
                        socket_name: socket_name.clone(),
                    }
                }
                AttrFuncInputSpec::OutputSocket { name, socket_name } => {
                    AttrFuncInputNode::OutputSocket {
                        name: name.clone(),
                        socket_name: socket_name.clone(),
                    }
                }
            }),
            vec![],
        )
    }
}
