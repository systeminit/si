use std::{
    io::{BufRead, Write},
    str::FromStr,
};

use object_tree::{
    read_key_value_line, read_key_value_line_opt, write_key_value_line, GraphError, NameStr,
    NodeChild, NodeKind, NodeWithChildren, ReadBytes, WriteBytes,
};
use url::Url;

use crate::{node::SchemaVariantChild, SchemaVariantSpec, SchemaVariantSpecComponentType};

use super::{read_common_fields, write_common_fields, PkgNode};

const KEY_COLOR_STR: &str = "color";
const KEY_LINK_STR: &str = "link";
const KEY_NAME_STR: &str = "name";
const KEY_COMPONENT_TYPE_STR: &str = "component_type";
const KEY_FUNC_UNIQUE_ID_STR: &str = "func_unique_id";

#[derive(Clone, Debug)]
pub struct SchemaVariantData {
    pub name: String,
    pub link: Option<Url>,
    pub color: Option<String>,
    pub component_type: SchemaVariantSpecComponentType,
    pub func_unique_id: String,
}

#[derive(Clone, Debug)]
pub struct SchemaVariantNode {
    pub name: String,
    pub data: Option<SchemaVariantData>,
    pub unique_id: Option<String>,
    pub deleted: bool,
}

impl NameStr for SchemaVariantNode {
    fn name(&self) -> &str {
        &self.name
    }
}

impl WriteBytes for SchemaVariantNode {
    fn write_bytes<W: Write>(&self, writer: &mut W) -> Result<(), GraphError> {
        write_key_value_line(writer, KEY_NAME_STR, self.name())?;
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
        }

        write_common_fields(writer, self.unique_id.as_deref(), self.deleted)?;

        Ok(())
    }
}

impl ReadBytes for SchemaVariantNode {
    fn read_bytes<R: BufRead>(reader: &mut R) -> Result<Option<Self>, GraphError>
    where
        Self: std::marker::Sized,
    {
        let name = read_key_value_line(reader, KEY_NAME_STR)?;
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

                Some(SchemaVariantData {
                    name: name.to_owned(),
                    link,
                    color,
                    component_type,
                    func_unique_id,
                })
            }
            None => None,
        };

        let (unique_id, deleted) = read_common_fields(reader)?;

        Ok(Some(Self {
            name,
            data,
            unique_id,
            deleted,
        }))
    }
}

impl NodeChild for SchemaVariantSpec {
    type NodeType = PkgNode;

    fn as_node_with_children(&self) -> NodeWithChildren<Self::NodeType> {
        NodeWithChildren::new(
            NodeKind::Tree,
            Self::NodeType::SchemaVariant(SchemaVariantNode {
                name: self.name.to_owned(),
                data: self.data.as_ref().map(|data| SchemaVariantData {
                    name: self.name.to_owned(),
                    link: data.link.as_ref().cloned(),
                    color: data.color.as_ref().cloned(),
                    component_type: data.component_type,
                    func_unique_id: data.func_unique_id.to_owned(),
                }),
                unique_id: self.unique_id.to_owned(),
                deleted: self.deleted,
            }),
            vec![
                Box::new(SchemaVariantChild::ActionFuncs(self.action_funcs.clone()))
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
            ],
        )
    }
}
