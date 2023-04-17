use std::io::{BufRead, Write};

use object_tree::{
    read_key_value_line, write_key_value_line, GraphError, NameStr, NodeChild, NodeKind,
    NodeWithChildren, ReadBytes, WriteBytes,
};
use serde::{Deserialize, Serialize};

use crate::{PropSpec, QualificationSpec};

use super::PkgNode;

const VARIANT_CHILD_TYPE_DOMAIN: &str = "domain";
const VARIANT_CHILD_TYPE_QUALIFICATIONS: &str = "qualifications";

const KEY_KIND_STR: &str = "kind";

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SchemaVariantChild {
    Domain(PropSpec),
    Qualifications(Vec<QualificationSpec>),
}

#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
pub enum SchemaVariantChildNode {
    Domain,
    Qualifications,
}

impl SchemaVariantChildNode {
    pub fn kind_str(&self) -> &'static str {
        match self {
            Self::Domain => VARIANT_CHILD_TYPE_DOMAIN,
            Self::Qualifications => VARIANT_CHILD_TYPE_QUALIFICATIONS,
        }
    }
}

impl NameStr for SchemaVariantChildNode {
    fn name(&self) -> &str {
        match self {
            Self::Domain => VARIANT_CHILD_TYPE_DOMAIN,
            Self::Qualifications => VARIANT_CHILD_TYPE_QUALIFICATIONS,
        }
    }
}

impl WriteBytes for SchemaVariantChildNode {
    fn write_bytes<W: Write>(&self, writer: &mut W) -> Result<(), GraphError> {
        write_key_value_line(writer, KEY_KIND_STR, self.kind_str())?;
        Ok(())
    }
}

impl ReadBytes for SchemaVariantChildNode {
    fn read_bytes<R: BufRead>(reader: &mut R) -> Result<Self, GraphError>
    where
        Self: std::marker::Sized,
    {
        let kind_str = read_key_value_line(reader, KEY_KIND_STR)?;

        let node = match kind_str.as_str() {
            VARIANT_CHILD_TYPE_DOMAIN => Self::Domain,
            VARIANT_CHILD_TYPE_QUALIFICATIONS => Self::Qualifications,
            invalid_kind => {
                return Err(GraphError::parse_custom(format!(
                    "invalid schema variant child kind: {invalid_kind}"
                )))
            }
        };

        Ok(node)
    }
}

impl NodeChild for SchemaVariantChild {
    type NodeType = PkgNode;

    fn as_node_with_children(&self) -> NodeWithChildren<Self::NodeType> {
        match self {
            Self::Domain(domain) => {
                let domain =
                    Box::new(domain.clone()) as Box<dyn NodeChild<NodeType = Self::NodeType>>;

                NodeWithChildren::new(
                    NodeKind::Tree,
                    Self::NodeType::SchemaVariantChild(SchemaVariantChildNode::Domain),
                    vec![domain],
                )
            }
            Self::Qualifications(entries) => {
                let mut children = Vec::new();
                for entry in entries {
                    children
                        .push(Box::new(entry.clone())
                            as Box<dyn NodeChild<NodeType = Self::NodeType>>);
                }

                NodeWithChildren::new(
                    NodeKind::Tree,
                    Self::NodeType::SchemaVariantChild(SchemaVariantChildNode::Qualifications),
                    children,
                )
            }
        }
    }
}
