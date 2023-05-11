use std::io::{BufRead, Write};

use object_tree::{
    read_key_value_line, write_key_value_line, GraphError, NameStr, NodeChild, NodeKind,
    NodeWithChildren, ReadBytes, WriteBytes,
};
use serde::{Deserialize, Serialize};

use crate::{AttrFuncInputSpec, PropSpec, ValidationSpec};

use super::PkgNode;

const PROP_CHILD_TYPE_PROPS: &str = "props";
const PROP_CHILD_TYPE_VALIDATIONS: &str = "validations";
const PROP_CHILD_TYPE_ATTR_FUNC_INPUTS: &str = "attr_func_inputs";

const KEY_KIND_STR: &str = "kind";

#[remain::sorted]
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum PropChild {
    AttrFuncInputs(Vec<AttrFuncInputSpec>),
    Props(Vec<PropSpec>),
    Validations(Vec<ValidationSpec>),
}

#[remain::sorted]
#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
pub enum PropChildNode {
    AttrFuncInputs,
    Props,
    Validations,
}

impl PropChildNode {
    pub fn kind_str(&self) -> &'static str {
        match self {
            Self::Props => PROP_CHILD_TYPE_PROPS,
            Self::Validations => PROP_CHILD_TYPE_VALIDATIONS,
            Self::AttrFuncInputs => PROP_CHILD_TYPE_ATTR_FUNC_INPUTS,
        }
    }
}

impl NameStr for PropChildNode {
    fn name(&self) -> &str {
        match self {
            Self::Props => PROP_CHILD_TYPE_PROPS,
            Self::Validations => PROP_CHILD_TYPE_VALIDATIONS,
            Self::AttrFuncInputs => PROP_CHILD_TYPE_ATTR_FUNC_INPUTS,
        }
    }
}

impl WriteBytes for PropChildNode {
    fn write_bytes<W: Write>(&self, writer: &mut W) -> Result<(), GraphError> {
        write_key_value_line(writer, KEY_KIND_STR, self.kind_str())?;
        Ok(())
    }
}

impl ReadBytes for PropChildNode {
    fn read_bytes<R: BufRead>(reader: &mut R) -> Result<Self, GraphError>
    where
        Self: std::marker::Sized,
    {
        let kind_str = read_key_value_line(reader, KEY_KIND_STR)?;

        let node = match kind_str.as_str() {
            PROP_CHILD_TYPE_PROPS => Self::Props,
            PROP_CHILD_TYPE_VALIDATIONS => Self::Validations,
            PROP_CHILD_TYPE_ATTR_FUNC_INPUTS => Self::AttrFuncInputs,
            invalid_kind => {
                return Err(GraphError::parse_custom(format!(
                    "invalid schema variant child kind: {invalid_kind}"
                )))
            }
        };

        Ok(node)
    }
}

impl NodeChild for PropChild {
    type NodeType = PkgNode;

    fn as_node_with_children(&self) -> NodeWithChildren<Self::NodeType> {
        match self {
            Self::Props(props) => NodeWithChildren::new(
                NodeKind::Tree,
                Self::NodeType::PropChild(PropChildNode::Props),
                props
                    .iter()
                    .map(|prop| {
                        Box::new(prop.clone()) as Box<dyn NodeChild<NodeType = Self::NodeType>>
                    })
                    .collect(),
            ),
            Self::Validations(validations) => NodeWithChildren::new(
                NodeKind::Tree,
                Self::NodeType::PropChild(PropChildNode::Validations),
                validations
                    .iter()
                    .map(|validation| {
                        Box::new(validation.clone())
                            as Box<dyn NodeChild<NodeType = Self::NodeType>>
                    })
                    .collect(),
            ),
            Self::AttrFuncInputs(inputs) => NodeWithChildren::new(
                NodeKind::Tree,
                Self::NodeType::PropChild(PropChildNode::AttrFuncInputs),
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
