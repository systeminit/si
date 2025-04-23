use std::io::{
    BufRead,
    Write,
};

use object_tree::{
    GraphError,
    NameStr,
    NodeChild,
    NodeKind,
    NodeWithChildren,
    ReadBytes,
    WriteBytes,
    read_key_value_line,
    write_key_value_line,
};
use serde::{
    Deserialize,
    Serialize,
};

use super::PkgNode;
use crate::{
    ActionFuncSpec,
    AuthenticationFuncSpec,
    LeafFunctionSpec,
    ManagementFuncSpec,
    PropSpec,
    RootPropFuncSpec,
    SiPropFuncSpec,
    SocketSpec,
};

const VARIANT_CHILD_TYPE_ACTION_FUNCS: &str = "action_funcs";
const VARIANT_CHILD_TYPE_AUTH_FUNCS: &str = "auth_funcs";
const VARIANT_CHILD_TYPE_DOMAIN: &str = "domain";
const VARIANT_CHILD_TYPE_LEAF_FUNCTIONS: &str = "leaf_functions";
const VARIANT_CHILD_TYPE_MANAGEMENT_FUNCS: &str = "management_funcs";
const VARIANT_CHILD_TYPE_RESOURCE_VALUE: &str = "resource_value";
const VARIANT_CHILD_TYPE_SI_PROP_FUNCS: &str = "si_prop_funcs";
const VARIANT_CHILD_TYPE_SOCKETS: &str = "sockets";
const VARIANT_CHILD_TYPE_SECRET_DEFINITION: &str = "secret_definition";
const VARIANT_CHILD_TYPE_SECRETS: &str = "secrets";
const VARIANT_CHILD_TYPE_ROOT_PROP_FUNCS: &str = "root_prop_funcs";

const KEY_KIND_STR: &str = "kind";

#[remain::sorted]
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SchemaVariantChild {
    ActionFuncs(Vec<ActionFuncSpec>),
    AuthFuncs(Vec<AuthenticationFuncSpec>),
    Domain(PropSpec),
    LeafFunctions(Vec<LeafFunctionSpec>),
    ManagementFuncs(Vec<ManagementFuncSpec>),
    ResourceValue(PropSpec),
    RootPropFuncs(Vec<RootPropFuncSpec>),
    SecretDefinition(PropSpec),
    Secrets(PropSpec),
    SiPropFuncs(Vec<SiPropFuncSpec>),
    Sockets(Vec<SocketSpec>),
}

#[remain::sorted]
#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
pub enum SchemaVariantChildNode {
    ActionFuncs,
    AuthFuncs,
    Domain,
    LeafFunctions,
    ManagementFuncs,
    ResourceValue,
    RootPropFuncs,
    SecretDefinition,
    Secrets,
    SiPropFuncs,
    Sockets,
}

impl SchemaVariantChildNode {
    pub fn kind_str(&self) -> &'static str {
        match self {
            Self::ActionFuncs => VARIANT_CHILD_TYPE_ACTION_FUNCS,
            Self::AuthFuncs => VARIANT_CHILD_TYPE_AUTH_FUNCS,
            Self::Domain => VARIANT_CHILD_TYPE_DOMAIN,
            Self::LeafFunctions => VARIANT_CHILD_TYPE_LEAF_FUNCTIONS,
            Self::ManagementFuncs => VARIANT_CHILD_TYPE_MANAGEMENT_FUNCS,
            Self::ResourceValue => VARIANT_CHILD_TYPE_RESOURCE_VALUE,
            Self::RootPropFuncs => VARIANT_CHILD_TYPE_ROOT_PROP_FUNCS,
            Self::SecretDefinition => VARIANT_CHILD_TYPE_SECRET_DEFINITION,
            Self::Secrets => VARIANT_CHILD_TYPE_SECRETS,
            Self::SiPropFuncs => VARIANT_CHILD_TYPE_SI_PROP_FUNCS,
            Self::Sockets => VARIANT_CHILD_TYPE_SOCKETS,
        }
    }
}

impl NameStr for SchemaVariantChildNode {
    fn name(&self) -> &str {
        match self {
            Self::ActionFuncs => VARIANT_CHILD_TYPE_ACTION_FUNCS,
            Self::AuthFuncs => VARIANT_CHILD_TYPE_AUTH_FUNCS,
            Self::Domain => VARIANT_CHILD_TYPE_DOMAIN,
            Self::LeafFunctions => VARIANT_CHILD_TYPE_LEAF_FUNCTIONS,
            Self::ManagementFuncs => VARIANT_CHILD_TYPE_MANAGEMENT_FUNCS,
            Self::ResourceValue => VARIANT_CHILD_TYPE_RESOURCE_VALUE,
            Self::RootPropFuncs => VARIANT_CHILD_TYPE_ROOT_PROP_FUNCS,
            Self::SecretDefinition => VARIANT_CHILD_TYPE_SECRET_DEFINITION,
            Self::Secrets => VARIANT_CHILD_TYPE_SECRETS,
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
            VARIANT_CHILD_TYPE_AUTH_FUNCS => Self::AuthFuncs,
            VARIANT_CHILD_TYPE_DOMAIN => Self::Domain,
            VARIANT_CHILD_TYPE_LEAF_FUNCTIONS => Self::LeafFunctions,
            VARIANT_CHILD_TYPE_MANAGEMENT_FUNCS => Self::ManagementFuncs,
            VARIANT_CHILD_TYPE_RESOURCE_VALUE => Self::ResourceValue,
            VARIANT_CHILD_TYPE_SI_PROP_FUNCS => Self::SiPropFuncs,
            VARIANT_CHILD_TYPE_ROOT_PROP_FUNCS => Self::RootPropFuncs,
            VARIANT_CHILD_TYPE_SOCKETS => Self::Sockets,
            VARIANT_CHILD_TYPE_SECRETS => Self::Secrets,
            VARIANT_CHILD_TYPE_SECRET_DEFINITION => Self::SecretDefinition,
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
            Self::AuthFuncs(funcs) => NodeWithChildren::new(
                NodeKind::Tree,
                Self::NodeType::SchemaVariantChild(SchemaVariantChildNode::AuthFuncs),
                funcs
                    .iter()
                    .map(|func| {
                        Box::new(func.clone()) as Box<dyn NodeChild<NodeType = Self::NodeType>>
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
            Self::Secrets(secrets) => {
                let secrets =
                    Box::new(secrets.clone()) as Box<dyn NodeChild<NodeType = Self::NodeType>>;

                NodeWithChildren::new(
                    NodeKind::Tree,
                    Self::NodeType::SchemaVariantChild(SchemaVariantChildNode::Secrets),
                    vec![secrets],
                )
            }
            SchemaVariantChild::SecretDefinition(secret_definition) => {
                let secret_definition = Box::new(secret_definition.clone())
                    as Box<dyn NodeChild<NodeType = Self::NodeType>>;

                NodeWithChildren::new(
                    NodeKind::Tree,
                    Self::NodeType::SchemaVariantChild(SchemaVariantChildNode::SecretDefinition),
                    vec![secret_definition],
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
            Self::RootPropFuncs(root_prop_funcs) => NodeWithChildren::new(
                NodeKind::Tree,
                Self::NodeType::SchemaVariantChild(SchemaVariantChildNode::RootPropFuncs),
                root_prop_funcs
                    .iter()
                    .map(|prop_func| {
                        Box::new(prop_func.clone()) as Box<dyn NodeChild<NodeType = Self::NodeType>>
                    })
                    .collect(),
            ),
            Self::ManagementFuncs(management_funcs) => NodeWithChildren::new(
                NodeKind::Tree,
                Self::NodeType::SchemaVariantChild(SchemaVariantChildNode::ManagementFuncs),
                management_funcs
                    .iter()
                    .map(|management_func| {
                        Box::new(management_func.clone())
                            as Box<dyn NodeChild<NodeType = Self::NodeType>>
                    })
                    .collect(),
            ),
        }
    }
}
