use std::io::{BufRead, Write};

use object_tree::{
    read_key_value_line, write_key_value_line, GraphError, NameStr, NodeChild, NodeKind,
    NodeWithChildren, ReadBytes, WriteBytes,
};
use serde::{Deserialize, Serialize};

use crate::{ActionFuncSpec, LeafFunctionSpec, PropSpec, SiPropFuncSpec, SocketSpec};

use super::PkgNode;

const VARIANT_CHILD_TYPE_ACTION_FUNCS: &str = "action_funcs";
const VARIANT_CHILD_TYPE_DOMAIN: &str = "domain";
const VARIANT_CHILD_TYPE_LEAF_FUNCTIONS: &str = "leaf_functions";
const VARIANT_CHILD_TYPE_RESOURCE_VALUE: &str = "resource_value";
const VARIANT_CHILD_TYPE_SI_PROP_FUNCS: &str = "si_prop_funcs";
const VARIANT_CHILD_TYPE_SOCKETS: &str = "sockets";

const KEY_KIND_STR: &str = "kind";

#[remain::sorted]
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SchemaVariantChild {
    ActionFuncs(Vec<ActionFuncSpec>),
    Domain(PropSpec),
    LeafFunctions(Vec<LeafFunctionSpec>),
    ResourceValue(PropSpec),
    SiPropFuncs(Vec<SiPropFuncSpec>),
    Sockets(Vec<SocketSpec>),
}

#[remain::sorted]
#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
pub enum SchemaVariantChildNode {
    ActionFuncs,
    Domain,
    LeafFunctions,
    ResourceValue,
    SiPropFuncs,
    Sockets,
}

impl SchemaVariantChildNode {
    pub fn kind_str(&self) -> &'static str {
        match self {
            Self::ActionFuncs => VARIANT_CHILD_TYPE_ACTION_FUNCS,
            Self::Domain => VARIANT_CHILD_TYPE_DOMAIN,
            Self::LeafFunctions => VARIANT_CHILD_TYPE_LEAF_FUNCTIONS,
            Self::ResourceValue => VARIANT_CHILD_TYPE_RESOURCE_VALUE,
            Self::SiPropFuncs => VARIANT_CHILD_TYPE_SI_PROP_FUNCS,
            Self::Sockets => VARIANT_CHILD_TYPE_SOCKETS,
        }
    }
}

impl NameStr for SchemaVariantChildNode {
    fn name(&self) -> &str {
        match self {
            Self::ActionFuncs => VARIANT_CHILD_TYPE_ACTION_FUNCS,
            Self::Domain => VARIANT_CHILD_TYPE_DOMAIN,
            Self::LeafFunctions => VARIANT_CHILD_TYPE_LEAF_FUNCTIONS,
            Self::ResourceValue => VARIANT_CHILD_TYPE_RESOURCE_VALUE,
            Self::SiPropFuncs => VARIANT_CHILD_TYPE_SI_PROP_FUNCS,
            Self::Sockets => VARIANT_CHILD_TYPE_SOCKETS,
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
    fn read_bytes<R: BufRead>(reader: &mut R) -> Result<Option<Self>, GraphError>
    where
        Self: std::marker::Sized,
    {
        let kind_str = read_key_value_line(reader, KEY_KIND_STR)?;

        let node = match kind_str.as_str() {
            VARIANT_CHILD_TYPE_ACTION_FUNCS => Self::ActionFuncs,
            VARIANT_CHILD_TYPE_DOMAIN => Self::Domain,
            VARIANT_CHILD_TYPE_LEAF_FUNCTIONS => Self::LeafFunctions,
            VARIANT_CHILD_TYPE_RESOURCE_VALUE => Self::ResourceValue,
            VARIANT_CHILD_TYPE_SI_PROP_FUNCS => Self::SiPropFuncs,
            VARIANT_CHILD_TYPE_SOCKETS => Self::Sockets,
            invalid_kind => {
                dbg!(format!("invalid schema variant child kind: {invalid_kind}"));
                return Ok(None);
            }
        };

        Ok(Some(node))
    }
}

impl NodeChild for SchemaVariantChild {
    type NodeType = PkgNode;

    fn as_node_with_children(&self) -> NodeWithChildren<Self::NodeType> {
        match self {
            Self::ActionFuncs(action_funcs) => NodeWithChildren::new(
                NodeKind::Tree,
                Self::NodeType::SchemaVariantChild(SchemaVariantChildNode::ActionFuncs),
                action_funcs
                    .iter()
                    .map(|action_func| {
                        Box::new(action_func.clone())
                            as Box<dyn NodeChild<NodeType = Self::NodeType>>
                    })
                    .collect(),
            ),
            Self::Domain(domain) => {
                let domain =
                    Box::new(domain.clone()) as Box<dyn NodeChild<NodeType = Self::NodeType>>;

                NodeWithChildren::new(
                    NodeKind::Tree,
                    Self::NodeType::SchemaVariantChild(SchemaVariantChildNode::Domain),
                    vec![domain],
                )
            }
            Self::ResourceValue(resource_value) => {
                let resource_value = Box::new(resource_value.clone())
                    as Box<dyn NodeChild<NodeType = Self::NodeType>>;

                NodeWithChildren::new(
                    NodeKind::Tree,
                    Self::NodeType::SchemaVariantChild(SchemaVariantChildNode::ResourceValue),
                    vec![resource_value],
                )
            }
            Self::LeafFunctions(entries) => NodeWithChildren::new(
                NodeKind::Tree,
                Self::NodeType::SchemaVariantChild(SchemaVariantChildNode::LeafFunctions),
                entries
                    .iter()
                    .map(|entry| {
                        Box::new(entry.clone()) as Box<dyn NodeChild<NodeType = Self::NodeType>>
                    })
                    .collect(),
            ),
            Self::Sockets(sockets) => NodeWithChildren::new(
                NodeKind::Tree,
                Self::NodeType::SchemaVariantChild(SchemaVariantChildNode::Sockets),
                sockets
                    .iter()
                    .map(|socket| {
                        Box::new(socket.clone()) as Box<dyn NodeChild<NodeType = Self::NodeType>>
                    })
                    .collect(),
            ),
            Self::SiPropFuncs(si_prop_funcs) => NodeWithChildren::new(
                NodeKind::Tree,
                Self::NodeType::SchemaVariantChild(SchemaVariantChildNode::SiPropFuncs),
                si_prop_funcs
                    .iter()
                    .map(|si_prop_func| {
                        Box::new(si_prop_func.clone())
                            as Box<dyn NodeChild<NodeType = Self::NodeType>>
                    })
                    .collect(),
            ),
        }
    }
}
