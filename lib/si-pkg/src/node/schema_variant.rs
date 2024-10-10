use std::{
    io::{BufRead, Write},
    str::FromStr,
};

use object_tree::{
    read_key_value_line, read_key_value_line_opt, write_key_value_line, write_key_value_line_opt,
    GraphError, NameStr, NodeChild, NodeKind, NodeWithChildren, ReadBytes, WriteBytes,
};
use url::Url;

use crate::{node::SchemaVariantChild, SchemaVariantSpec, SchemaVariantSpecComponentType};

use super::{read_common_fields, write_common_fields, PkgNode};

const KEY_COLOR_STR: &str = "color";
const KEY_LINK_STR: &str = "link";
const KEY_VERSION_STR: &str = "name"; // This got renamed to Version, but we kept the key for backwards compatibility
const KEY_COMPONENT_TYPE_STR: &str = "component_type";
const KEY_FUNC_UNIQUE_ID_STR: &str = "func_unique_id";
const KEY_IS_BUILTIN_STR: &str = "is_builtin";
const KEY_DESCRIPTION_STR: &str = "description";

#[derive(Clone, Debug)]
pub struct SchemaVariantData {
    pub version: String,
    pub link: Option<Url>,
    pub color: Option<String>,
    pub component_type: SchemaVariantSpecComponentType,
    pub func_unique_id: String,
    pub description: Option<String>,
}

#[derive(Clone, Debug)]
pub struct SchemaVariantNode {
    pub version: String,
    pub data: Option<SchemaVariantData>,
    pub unique_id: Option<String>,
    pub deleted: bool,
    pub is_builtin: bool,
}

impl NameStr for SchemaVariantNode {
    fn name(&self) -> &str {
        &self.version
    }
}

impl WriteBytes for SchemaVariantNode {
    fn write_bytes<W: Write>(&self, writer: &mut W) -> Result<(), GraphError> {
        write_key_value_line(writer, KEY_VERSION_STR, self.name())?;
        if let Some(data) = &self.data {
            write_key_value_line(
                writer,
                KEY_LINK_STR,
                data.link.as_ref().map(|l| l.as_str()).unwrap_or(""),
            )?;
            write_key_value_line(writer, KEY_COLOR_STR, data.color.as_deref().unwrap_or(""))?;
            write_key_value_line(writer, KEY_COMPONENT_TYPE_STR, data.component_type)?;
            write_key_value_line(
                writer,
                KEY_FUNC_UNIQUE_ID_STR,
                data.func_unique_id.to_string(),
            )?;
            write_key_value_line_opt(writer, KEY_DESCRIPTION_STR, data.description.as_deref())?;
        }

        write_common_fields(writer, self.unique_id.as_deref(), self.deleted)?;

        write_key_value_line(writer, KEY_IS_BUILTIN_STR, self.is_builtin)?;

        Ok(())
    }
}

impl ReadBytes for SchemaVariantNode {
    fn read_bytes<R: BufRead>(reader: &mut R) -> Result<Option<Self>, GraphError>
    where
        Self: std::marker::Sized,
    {
        let version = read_key_value_line(reader, KEY_VERSION_STR)?;
        let data = match read_key_value_line_opt(reader, KEY_LINK_STR)? {
            Some(link_str) => {
                let link = if link_str.is_empty() {
                    None
                } else {
                    Some(Url::parse(&link_str).map_err(GraphError::parse)?)
                };
                let color_str = read_key_value_line(reader, KEY_COLOR_STR)?;
                let color = if color_str.is_empty() {
                    None
                } else {
                    Some(color_str)
                };
                let component_type_str = read_key_value_line(reader, KEY_COMPONENT_TYPE_STR)?;
                let component_type = SchemaVariantSpecComponentType::from_str(&component_type_str)
                    .map_err(GraphError::parse)?;

                let func_unique_id = read_key_value_line(reader, KEY_FUNC_UNIQUE_ID_STR)?;
                let description = read_key_value_line_opt(reader, KEY_DESCRIPTION_STR)?;

                Some(SchemaVariantData {
                    version: version.to_owned(),
                    link,
                    color,
                    component_type,
                    func_unique_id,
                    description,
                })
            }
            None => None,
        };

        let (unique_id, deleted) = read_common_fields(reader)?;

        let is_builtin = match read_key_value_line_opt(reader, KEY_IS_BUILTIN_STR)? {
            None => false,
            Some(is_builtin_str) => bool::from_str(&is_builtin_str).map_err(GraphError::parse)?,
        };

        Ok(Some(Self {
            version,
            data,
            unique_id,
            deleted,
            is_builtin,
        }))
    }
}

impl NodeChild for SchemaVariantSpec {
    type NodeType = PkgNode;

    fn as_node_with_children(&self) -> NodeWithChildren<Self::NodeType> {
        let mut children = vec![
            Box::new(SchemaVariantChild::ActionFuncs(self.action_funcs.clone()))
                as Box<dyn NodeChild<NodeType = Self::NodeType>>,
            Box::new(SchemaVariantChild::AuthFuncs(self.auth_funcs.clone()))
                as Box<dyn NodeChild<NodeType = Self::NodeType>>,
            Box::new(SchemaVariantChild::Domain(self.domain.clone()))
                as Box<dyn NodeChild<NodeType = Self::NodeType>>,
            Box::new(SchemaVariantChild::ResourceValue(
                self.resource_value.clone(),
            )) as Box<dyn NodeChild<NodeType = Self::NodeType>>,
            Box::new(SchemaVariantChild::LeafFunctions(
                self.leaf_functions.clone(),
            )) as Box<dyn NodeChild<NodeType = Self::NodeType>>,
            Box::new(SchemaVariantChild::Sockets(self.sockets.clone()))
                as Box<dyn NodeChild<NodeType = Self::NodeType>>,
            Box::new(SchemaVariantChild::SiPropFuncs(self.si_prop_funcs.clone()))
                as Box<dyn NodeChild<NodeType = Self::NodeType>>,
            Box::new(SchemaVariantChild::Secrets(self.secrets.clone()))
                as Box<dyn NodeChild<NodeType = Self::NodeType>>,
            Box::new(SchemaVariantChild::RootPropFuncs(
                self.root_prop_funcs.clone(),
            )) as Box<dyn NodeChild<NodeType = Self::NodeType>>,
            Box::new(SchemaVariantChild::ManagementFuncs(
                self.management_funcs.clone(),
            )) as Box<dyn NodeChild<NodeType = Self::NodeType>>,
        ];

        if let Some(secret_definition) = self.secret_definition.clone() {
            children.push(
                Box::new(SchemaVariantChild::SecretDefinition(secret_definition))
                    as Box<dyn NodeChild<NodeType = Self::NodeType>>,
            )
        }

        NodeWithChildren::new(
            NodeKind::Tree,
            Self::NodeType::SchemaVariant(SchemaVariantNode {
                version: self.version.to_owned(),
                data: self.data.as_ref().map(|data| SchemaVariantData {
                    version: self.version.to_owned(),
                    link: data.link.as_ref().cloned(),
                    color: data.color.as_ref().cloned(),
                    component_type: data.component_type,
                    func_unique_id: data.func_unique_id.to_owned(),
                    description: data.description.to_owned(),
                }),
                unique_id: self.unique_id.to_owned(),
                deleted: self.deleted,
                is_builtin: self.is_builtin,
            }),
            children,
        )
    }
}
