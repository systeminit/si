use std::io::{BufRead, Write};

use object_tree::{
    read_key_value_line, write_key_value_line, GraphError, NameStr, NodeChild, NodeKind,
    NodeWithChildren, ReadBytes, WriteBytes,
};

use crate::PropSpec;

use super::{prop_child::PropChild, PkgNode};

const KEY_KIND_STR: &str = "kind";
const KEY_NAME_STR: &str = "name";

const PROP_TY_STRING: &str = "string";
const PROP_TY_INTEGER: &str = "integer";
const PROP_TY_BOOLEAN: &str = "boolean";
const PROP_TY_MAP: &str = "map";
const PROP_TY_ARRAY: &str = "array";
const PROP_TY_OBJECT: &str = "object";

#[derive(Clone, Debug)]
pub enum PropNode {
    String { name: String },
    Integer { name: String },
    Boolean { name: String },
    Map { name: String },
    Array { name: String },
    Object { name: String },
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

        let node = match kind_str.as_str() {
            PROP_TY_STRING => Self::String { name },
            PROP_TY_INTEGER => Self::Integer { name },
            PROP_TY_BOOLEAN => Self::Boolean { name },
            PROP_TY_MAP => Self::Map { name },
            PROP_TY_ARRAY => Self::Array { name },
            PROP_TY_OBJECT => Self::Object { name },
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
            Self::String { name, validations } => NodeWithChildren::new(
                NodeKind::Tree,
                Self::NodeType::Prop(PropNode::String {
                    name: name.to_string(),
                }),
                vec![Box::new(PropChild::Validations(validations.clone()))
                    as Box<dyn NodeChild<NodeType = Self::NodeType>>],
            ),
            Self::Number { name, validations } => NodeWithChildren::new(
                NodeKind::Tree,
                Self::NodeType::Prop(PropNode::Integer {
                    name: name.to_string(),
                }),
                vec![Box::new(PropChild::Validations(validations.clone()))
                    as Box<dyn NodeChild<NodeType = Self::NodeType>>],
            ),
            Self::Boolean { name, validations } => NodeWithChildren::new(
                NodeKind::Tree,
                Self::NodeType::Prop(PropNode::Boolean {
                    name: name.to_string(),
                }),
                vec![Box::new(PropChild::Validations(validations.clone()))
                    as Box<dyn NodeChild<NodeType = Self::NodeType>>],
            ),
            Self::Map {
                name,
                type_prop,
                validations,
            } => NodeWithChildren::new(
                NodeKind::Tree,
                Self::NodeType::Prop(PropNode::Map {
                    name: name.to_string(),
                }),
                vec![
                    Box::new(PropChild::Props(vec![*type_prop.clone()]))
                        as Box<dyn NodeChild<NodeType = Self::NodeType>>,
                    Box::new(PropChild::Validations(validations.clone()))
                        as Box<dyn NodeChild<NodeType = Self::NodeType>>,
                ],
            ),
            Self::Array {
                name,
                type_prop,
                validations,
            } => NodeWithChildren::new(
                NodeKind::Tree,
                Self::NodeType::Prop(PropNode::Array {
                    name: name.to_string(),
                }),
                vec![
                    Box::new(PropChild::Props(vec![*type_prop.clone()]))
                        as Box<dyn NodeChild<NodeType = Self::NodeType>>,
                    Box::new(PropChild::Validations(validations.clone()))
                        as Box<dyn NodeChild<NodeType = Self::NodeType>>,
                ],
            ),
            Self::Object {
                name,
                entries,
                validations,
            } => NodeWithChildren::new(
                NodeKind::Tree,
                Self::NodeType::Prop(PropNode::Object {
                    name: name.to_string(),
                }),
                vec![
                    Box::new(PropChild::Props(entries.clone()))
                        as Box<dyn NodeChild<NodeType = Self::NodeType>>,
                    Box::new(PropChild::Validations(validations.clone()))
                        as Box<dyn NodeChild<NodeType = Self::NodeType>>,
                ],
            ),
        }
    }
}
