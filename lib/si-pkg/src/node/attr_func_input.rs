use std::{
    io::{BufRead, Write},
    str::FromStr,
};

use object_tree::{
    read_key_value_line, write_key_value_line, GraphError, NameStr, NodeChild, NodeKind,
    NodeWithChildren, ReadBytes, WriteBytes,
};

use crate::{AttrFuncInputSpec, AttrFuncInputSpecKind};

use super::{read_common_fields, write_common_fields, PkgNode};

const KEY_KIND_STR: &str = "kind";
const KEY_NAME_STR: &str = "name";
const KEY_PROP_PATH_STR: &str = "prop_path";
const KEY_SOCKET_NAME_STR: &str = "socket_name";

#[remain::sorted]
#[derive(Clone, Debug)]
pub enum AttrFuncInputNode {
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

        match self {
            Self::InputSocket {
                unique_id, deleted, ..
            }
            | Self::OutputSocket {
                unique_id, deleted, ..
            }
            | Self::Prop {
                unique_id, deleted, ..
            } => {
                write_common_fields(writer, unique_id.as_deref(), *deleted)?;
            }
        }

        Ok(())
    }
}

impl ReadBytes for AttrFuncInputNode {
    fn read_bytes<R: BufRead>(reader: &mut R) -> Result<Option<Self>, GraphError>
    where
        Self: std::marker::Sized,
    {
        let name = read_key_value_line(reader, KEY_NAME_STR)?;
        let kind_str = read_key_value_line(reader, KEY_KIND_STR)?;

        let kind = AttrFuncInputSpecKind::from_str(&kind_str).map_err(GraphError::parse)?;

        Ok(Some(match kind {
            AttrFuncInputSpecKind::Prop => {
                let prop_path = read_key_value_line(reader, KEY_PROP_PATH_STR)?;
                let (unique_id, deleted) = read_common_fields(reader)?;
                Self::Prop {
                    name,
                    prop_path,
                    unique_id,
                    deleted,
                }
            }
            AttrFuncInputSpecKind::InputSocket => {
                let socket_name = read_key_value_line(reader, KEY_SOCKET_NAME_STR)?;
                let (unique_id, deleted) = read_common_fields(reader)?;
                Self::InputSocket {
                    name,
                    socket_name,
                    unique_id,
                    deleted,
                }
            }
            AttrFuncInputSpecKind::OutputSocket => {
                let socket_name = read_key_value_line(reader, KEY_SOCKET_NAME_STR)?;
                let (unique_id, deleted) = read_common_fields(reader)?;
                Self::OutputSocket {
                    name,
                    socket_name,
                    unique_id,
                    deleted,
                }
            }
        }))
    }
}

impl NodeChild for AttrFuncInputSpec {
    type NodeType = PkgNode;

    fn as_node_with_children(&self) -> NodeWithChildren<Self::NodeType> {
        NodeWithChildren::new(
            NodeKind::Leaf,
            Self::NodeType::AttrFuncInput(match self.to_owned() {
                AttrFuncInputSpec::Prop {
                    name,
                    prop_path,
                    unique_id,
                    deleted,
                } => AttrFuncInputNode::Prop {
                    name,
                    prop_path,
                    unique_id,
                    deleted,
                },
                AttrFuncInputSpec::InputSocket {
                    name,
                    socket_name,
                    unique_id,
                    deleted,
                } => AttrFuncInputNode::InputSocket {
                    name,
                    socket_name,
                    unique_id,
                    deleted,
                },
                AttrFuncInputSpec::OutputSocket {
                    name,
                    socket_name,
                    unique_id,
                    deleted,
                } => AttrFuncInputNode::OutputSocket {
                    name,
                    socket_name,
                    unique_id,
                    deleted,
                },
            }),
            vec![],
        )
    }
}
