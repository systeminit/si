use std::io::{BufRead, Write};

use object_tree::{
    read_key_value_line, write_key_value_line, GraphError, NameStr, NodeChild, NodeKind,
    NodeWithChildren, ReadBytes, WriteBytes,
};
use serde::{Deserialize, Serialize};

use crate::AttrFuncInputSpec;

use super::PkgNode;

const ATTRIBUTE_VALUE_CHILD_TYPE_ATTR_FUNC_INPUTS: &str = "attr_func_inputs";

const KEY_KIND_STR: &str = "kind";

#[remain::sorted]
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum AttributeValueChild {
    AttrFuncInputs(Vec<AttrFuncInputSpec>),
}

#[remain::sorted]
#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
pub enum AttributeValueChildNode {
    AttrFuncInputs,
}

impl AttributeValueChildNode {
    pub fn kind_str(&self) -> &'static str {
        match self {
            Self::AttrFuncInputs => ATTRIBUTE_VALUE_CHILD_TYPE_ATTR_FUNC_INPUTS,
        }
    }
}

impl NameStr for AttributeValueChildNode {
    fn name(&self) -> &str {
        match self {
            Self::AttrFuncInputs => ATTRIBUTE_VALUE_CHILD_TYPE_ATTR_FUNC_INPUTS,
        }
    }
}

impl WriteBytes for AttributeValueChildNode {
    fn write_bytes<W: Write>(&self, writer: &mut W) -> Result<(), GraphError> {
        write_key_value_line(writer, KEY_KIND_STR, self.kind_str())?;
        Ok(())
    }
}

impl ReadBytes for AttributeValueChildNode {
    fn read_bytes<R: BufRead>(reader: &mut R) -> Result<Option<Self>, GraphError>
    where
        Self: std::marker::Sized,
    {
        let kind_str = read_key_value_line(reader, KEY_KIND_STR)?;

        let node = match kind_str.as_str() {
            ATTRIBUTE_VALUE_CHILD_TYPE_ATTR_FUNC_INPUTS => Self::AttrFuncInputs,
            invalid_kind => {
                dbg!(format!("invalid schema variant child kind: {invalid_kind}"));
                return Ok(None);
            }
        };

        Ok(Some(node))
    }
}

impl NodeChild for AttributeValueChild {
    type NodeType = PkgNode;

    fn as_node_with_children(&self) -> NodeWithChildren<Self::NodeType> {
        match self {
            Self::AttrFuncInputs(inputs) => NodeWithChildren::new(
                NodeKind::Tree,
                Self::NodeType::AttributeValueChild(AttributeValueChildNode::AttrFuncInputs),
                inputs
                    .iter()
                    .map(|input| {
                        Box::new(input.clone()) as Box<dyn NodeChild<NodeType = Self::NodeType>>
                    })
                    .collect(),
            ),
        }
    }
}
