use std::io::{BufRead, Write};

use object_tree::{
    read_key_value_line, write_key_value_line, GraphError, NameStr, NodeChild, NodeKind,
    NodeWithChildren, ReadBytes, WriteBytes,
};
use serde::{Deserialize, Serialize};

use crate::{PropSpec, ValidationSpec};

use super::PkgNode;

const PROP_CHILD_TYPE_PROPS: &str = "props";
const PROP_CHILD_TYPE_VALIDATIONS: &str = "validations";

const KEY_KIND_STR: &str = "kind";

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum PropChild {
    Props(Vec<PropSpec>),
    Validations(Vec<ValidationSpec>),
}

#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
pub enum PropChildNode {
    Props,
    Validations,
}

impl PropChildNode {
    pub fn kind_str(&self) -> &'static str {
        match self {
            Self::Props => PROP_CHILD_TYPE_PROPS,
            Self::Validations => PROP_CHILD_TYPE_VALIDATIONS,
        }
    }
}

impl NameStr for PropChildNode {
    fn name(&self) -> &str {
        match self {
            Self::Props => PROP_CHILD_TYPE_PROPS,
            Self::Validations => PROP_CHILD_TYPE_VALIDATIONS,
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
            Self::Props(props) => {
                let props = props
                    .iter()
                    .map(|prop| {
                        Box::new(prop.clone()) as Box<dyn NodeChild<NodeType = Self::NodeType>>
                    })
                    .collect();

                NodeWithChildren::new(
                    NodeKind::Tree,
                    Self::NodeType::PropChild(PropChildNode::Props),
                    props,
                )
            }
            Self::Validations(validations) => {
                let validations = validations
                    .iter()
                    .map(|validation| {
                        Box::new(validation.clone())
                            as Box<dyn NodeChild<NodeType = Self::NodeType>>
                    })
                    .collect();

                NodeWithChildren::new(
                    NodeKind::Tree,
                    Self::NodeType::PropChild(PropChildNode::Validations),
                    validations,
                )
            }
        }
    }
}
