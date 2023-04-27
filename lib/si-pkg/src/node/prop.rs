use std::{
    io::{BufRead, Write},
    str::FromStr,
};

use object_tree::{
    read_key_value_line, write_key_value_line, GraphError, NameStr, NodeChild, NodeKind,
    NodeWithChildren, ReadBytes, WriteBytes,
};

use crate::{FuncUniqueId, PropSpec};

use super::{prop_child::PropChild, PkgNode};

const KEY_KIND_STR: &str = "kind";
const KEY_NAME_STR: &str = "name";
const KEY_FUNC_UNIQUE_ID_STR: &str = "func_unique_id";

const PROP_TY_STRING: &str = "string";
const PROP_TY_INTEGER: &str = "integer";
const PROP_TY_BOOLEAN: &str = "boolean";
const PROP_TY_MAP: &str = "map";
const PROP_TY_ARRAY: &str = "array";
const PROP_TY_OBJECT: &str = "object";

#[derive(Clone, Debug)]
pub enum PropNode {
    String {
        name: String,
        func_unique_id: Option<FuncUniqueId>,
    },
    Integer {
        name: String,
        func_unique_id: Option<FuncUniqueId>,
    },
    Boolean {
        name: String,
        func_unique_id: Option<FuncUniqueId>,
    },
    Map {
        name: String,
        func_unique_id: Option<FuncUniqueId>,
    },
    Array {
        name: String,
        func_unique_id: Option<FuncUniqueId>,
    },
    Object {
        name: String,
        func_unique_id: Option<FuncUniqueId>,
    },
}

impl PropNode {
    fn kind_str(&self) -> &'static str {
        match self {
            Self::String { .. } => PROP_TY_STRING,
            Self::Integer { .. } => PROP_TY_INTEGER,
            Self::Boolean { .. } => PROP_TY_BOOLEAN,
            Self::Map { .. } => PROP_TY_MAP,
            Self::Array { .. } => PROP_TY_ARRAY,
            Self::Object { .. } => PROP_TY_OBJECT,
        }
    }
}

impl NameStr for PropNode {
    fn name(&self) -> &str {
        match self {
            Self::String { name, .. }
            | Self::Integer { name, .. }
            | Self::Boolean { name, .. }
            | Self::Map { name, .. }
            | Self::Array { name, .. }
            | Self::Object { name, .. } => name,
        }
    }
}

impl WriteBytes for PropNode {
    fn write_bytes<W: Write>(&self, writer: &mut W) -> Result<(), GraphError> {
        write_key_value_line(writer, KEY_KIND_STR, self.kind_str())?;
        write_key_value_line(writer, KEY_NAME_STR, self.name())?;
        let func_unique_id = match self {
            Self::String { func_unique_id, .. }
            | Self::Integer { func_unique_id, .. }
            | Self::Boolean { func_unique_id, .. }
            | Self::Map { func_unique_id, .. }
            | Self::Array { func_unique_id, .. }
            | Self::Object { func_unique_id, .. } => func_unique_id,
        };
        write_key_value_line(
            writer,
            KEY_FUNC_UNIQUE_ID_STR,
            func_unique_id
                .map(|fuid| fuid.to_string())
                .unwrap_or("".to_string()),
        )?;

        Ok(())
    }
}

impl ReadBytes for PropNode {
    fn read_bytes<R: BufRead>(reader: &mut R) -> Result<Self, GraphError>
    where
        Self: std::marker::Sized,
    {
        let kind_str = read_key_value_line(reader, KEY_KIND_STR)?;
        let name = read_key_value_line(reader, KEY_NAME_STR)?;
        let func_unique_id_str = read_key_value_line(reader, KEY_FUNC_UNIQUE_ID_STR)?;
        let func_unique_id = if func_unique_id_str.is_empty() {
            None
        } else {
            Some(FuncUniqueId::from_str(&func_unique_id_str).map_err(GraphError::parse)?)
        };

        let node = match kind_str.as_str() {
            PROP_TY_STRING => Self::String {
                name,
                func_unique_id,
            },
            PROP_TY_INTEGER => Self::Integer {
                name,
                func_unique_id,
            },
            PROP_TY_BOOLEAN => Self::Boolean {
                name,
                func_unique_id,
            },
            PROP_TY_MAP => Self::Map {
                name,
                func_unique_id,
            },
            PROP_TY_ARRAY => Self::Array {
                name,
                func_unique_id,
            },
            PROP_TY_OBJECT => Self::Object {
                name,
                func_unique_id,
            },
            invalid_kind => {
                return Err(GraphError::parse_custom(format!(
                    "invalid prop node kind: {invalid_kind}"
                )))
            }
        };

        Ok(node)
    }
}

impl NodeChild for PropSpec {
    type NodeType = PkgNode;

    fn as_node_with_children(&self) -> NodeWithChildren<Self::NodeType> {
        match self {
            Self::String {
                name,
                validations,
                func_unique_id,
                inputs,
            } => NodeWithChildren::new(
                NodeKind::Tree,
                Self::NodeType::Prop(PropNode::String {
                    name: name.to_string(),
                    func_unique_id: *func_unique_id,
                }),
                vec![
                    Box::new(PropChild::Validations(
                        validations.to_owned().unwrap_or(vec![]),
                    )) as Box<dyn NodeChild<NodeType = Self::NodeType>>,
                    Box::new(PropChild::AttrFuncInputs(
                        inputs.to_owned().unwrap_or(vec![]),
                    )) as Box<dyn NodeChild<NodeType = Self::NodeType>>,
                ],
            ),
            Self::Number {
                name,
                validations,
                func_unique_id,
                inputs,
            } => NodeWithChildren::new(
                NodeKind::Tree,
                Self::NodeType::Prop(PropNode::Integer {
                    name: name.to_string(),
                    func_unique_id: *func_unique_id,
                }),
                vec![
                    Box::new(PropChild::Validations(
                        validations.to_owned().unwrap_or(vec![]),
                    )) as Box<dyn NodeChild<NodeType = Self::NodeType>>,
                    Box::new(PropChild::AttrFuncInputs(
                        inputs.to_owned().unwrap_or(vec![]),
                    )) as Box<dyn NodeChild<NodeType = Self::NodeType>>,
                ],
            ),
            Self::Boolean {
                name,
                validations,
                func_unique_id,
                inputs,
            } => NodeWithChildren::new(
                NodeKind::Tree,
                Self::NodeType::Prop(PropNode::Boolean {
                    name: name.to_string(),
                    func_unique_id: *func_unique_id,
                }),
                vec![
                    Box::new(PropChild::Validations(
                        validations.to_owned().unwrap_or(vec![]),
                    )) as Box<dyn NodeChild<NodeType = Self::NodeType>>,
                    Box::new(PropChild::AttrFuncInputs(
                        inputs.to_owned().unwrap_or(vec![]),
                    )) as Box<dyn NodeChild<NodeType = Self::NodeType>>,
                ],
            ),
            Self::Map {
                name,
                type_prop,
                validations,
                func_unique_id,
                inputs,
            } => NodeWithChildren::new(
                NodeKind::Tree,
                Self::NodeType::Prop(PropNode::Map {
                    name: name.to_string(),
                    func_unique_id: *func_unique_id,
                }),
                vec![
                    Box::new(PropChild::Props(vec![*type_prop.clone()]))
                        as Box<dyn NodeChild<NodeType = Self::NodeType>>,
                    Box::new(PropChild::Validations(
                        validations.to_owned().unwrap_or(vec![]),
                    )) as Box<dyn NodeChild<NodeType = Self::NodeType>>,
                    Box::new(PropChild::AttrFuncInputs(
                        inputs.to_owned().unwrap_or(vec![]),
                    )) as Box<dyn NodeChild<NodeType = Self::NodeType>>,
                ],
            ),
            Self::Array {
                name,
                type_prop,
                validations,
                func_unique_id,
                inputs,
            } => NodeWithChildren::new(
                NodeKind::Tree,
                Self::NodeType::Prop(PropNode::Array {
                    name: name.to_string(),
                    func_unique_id: *func_unique_id,
                }),
                vec![
                    Box::new(PropChild::Props(vec![*type_prop.clone()]))
                        as Box<dyn NodeChild<NodeType = Self::NodeType>>,
                    Box::new(PropChild::Validations(
                        validations.to_owned().unwrap_or(vec![]),
                    )) as Box<dyn NodeChild<NodeType = Self::NodeType>>,
                    Box::new(PropChild::AttrFuncInputs(
                        inputs.to_owned().unwrap_or(vec![]),
                    )) as Box<dyn NodeChild<NodeType = Self::NodeType>>,
                ],
            ),
            Self::Object {
                name,
                entries,
                validations,
                func_unique_id,
                inputs,
            } => NodeWithChildren::new(
                NodeKind::Tree,
                Self::NodeType::Prop(PropNode::Object {
                    name: name.to_string(),
                    func_unique_id: *func_unique_id,
                }),
                vec![
                    Box::new(PropChild::Props(entries.clone()))
                        as Box<dyn NodeChild<NodeType = Self::NodeType>>,
                    Box::new(PropChild::Validations(
                        validations.to_owned().unwrap_or(vec![]),
                    )) as Box<dyn NodeChild<NodeType = Self::NodeType>>,
                    Box::new(PropChild::AttrFuncInputs(
                        inputs.to_owned().unwrap_or(vec![]),
                    )) as Box<dyn NodeChild<NodeType = Self::NodeType>>,
                ],
            ),
        }
    }
}
