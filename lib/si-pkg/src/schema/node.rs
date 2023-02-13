use std::io::{BufRead, Write};

use object_tree::{
    read_key_value_line, write_key_value_line, GraphError, NameStr, NodeKind, NodeWithChildren,
    ObjectKindStr, ReadBytes, WriteBytes,
};
use serde::Serialize;

use super::Prop;

const TY_STRING: &str = "string";
const TY_INTEGER: &str = "integer";
const TY_BOOLEAN: &str = "boolean";
const TY_MAP: &str = "map";
const TY_ARRAY: &str = "array";
const TY_OBJECT: &str = "object";

const KEY_KIND_STR: &str = "kind";
const KEY_NAME_STR: &str = "name";

#[derive(Clone, Debug, Serialize)]
pub enum PropNode {
    String { name: String },
    Integer { name: String },
    Boolean { name: String },
    Map { name: String },
    Array { name: String },
    Object { name: String },
}

impl PropNode {
    fn prop_kind_str(&self) -> &'static str {
        match self {
            Self::String { .. } => TY_STRING,
            Self::Integer { .. } => TY_INTEGER,
            Self::Boolean { .. } => TY_BOOLEAN,
            Self::Map { .. } => TY_MAP,
            Self::Array { .. } => TY_ARRAY,
            Self::Object { .. } => TY_OBJECT,
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

impl ObjectKindStr for PropNode {
    fn object_kind(&self) -> &str {
        "prop"
    }
}

impl WriteBytes for PropNode {
    fn write_bytes<W: Write>(&self, writer: &mut W) -> Result<(), GraphError> {
        write_key_value_line(writer, KEY_KIND_STR, self.prop_kind_str())?;
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
            TY_STRING => Self::String { name },
            TY_INTEGER => Self::Integer { name },
            TY_BOOLEAN => Self::Boolean { name },
            TY_MAP => Self::Map { name },
            TY_ARRAY => Self::Array { name },
            TY_OBJECT => Self::Object { name },
            invalid_kind => {
                return Err(GraphError::parse_custom(format!(
                    "invalid prop node kind: {invalid_kind}"
                )))
            }
        };

        Ok(node)
    }
}

impl From<Prop> for NodeWithChildren<PropNode, Prop> {
    fn from(value: Prop) -> Self {
        match value {
            Prop::String { name } => Self::new(NodeKind::Leaf, PropNode::String { name }, vec![]),
            Prop::Number { name } => Self::new(NodeKind::Leaf, PropNode::Integer { name }, vec![]),
            Prop::Boolean { name } => Self::new(NodeKind::Leaf, PropNode::Boolean { name }, vec![]),
            Prop::Map { name, type_prop } => {
                Self::new(NodeKind::Tree, PropNode::Map { name }, vec![*type_prop])
            }
            Prop::Array { name, type_prop } => {
                Self::new(NodeKind::Tree, PropNode::Array { name }, vec![*type_prop])
            }
            Prop::Object { name, entries } => {
                Self::new(NodeKind::Tree, PropNode::Object { name }, entries)
            }
        }
    }
}
