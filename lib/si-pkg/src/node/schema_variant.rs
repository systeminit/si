use std::{
    io::{BufRead, Write},
    str::FromStr,
};

use object_tree::{
    read_key_value_line, write_key_value_line, GraphError, NameStr, NodeChild, NodeKind,
    NodeWithChildren, ReadBytes, WriteBytes,
};
use url::Url;

use crate::{node::SchemaVariantChild, SchemaVariantSpec, SchemaVariantSpecComponentType};

use super::PkgNode;

const KEY_COLOR_STR: &str = "color";
const KEY_LINK_STR: &str = "link";
const KEY_NAME_STR: &str = "name";
const KEY_COMPONENT_TYPE_STR: &str = "component_type";

#[derive(Clone, Debug)]
pub struct SchemaVariantNode {
    pub name: String,
    pub link: Option<Url>,
    pub color: Option<String>,
    pub component_type: SchemaVariantSpecComponentType,
}

impl NameStr for SchemaVariantNode {
    fn name(&self) -> &str {
        &self.name
    }
}

impl WriteBytes for SchemaVariantNode {
    fn write_bytes<W: Write>(&self, writer: &mut W) -> Result<(), GraphError> {
        write_key_value_line(writer, KEY_NAME_STR, self.name())?;
        write_key_value_line(
            writer,
            KEY_LINK_STR,
            self.link.as_ref().map(|l| l.as_str()).unwrap_or(""),
        )?;
        write_key_value_line(writer, KEY_COLOR_STR, self.color.as_deref().unwrap_or(""))?;
        write_key_value_line(writer, KEY_COMPONENT_TYPE_STR, self.component_type)?;

        Ok(())
    }
}

impl ReadBytes for SchemaVariantNode {
    fn read_bytes<R: BufRead>(reader: &mut R) -> Result<Self, GraphError>
    where
        Self: std::marker::Sized,
    {
        let name = read_key_value_line(reader, KEY_NAME_STR)?;
        let link_str = read_key_value_line(reader, KEY_LINK_STR)?;
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

        Ok(Self {
            name,
            link,
            color,
            component_type,
        })
    }
}

impl NodeChild for SchemaVariantSpec {
    type NodeType = PkgNode;

    fn as_node_with_children(&self) -> NodeWithChildren<Self::NodeType> {
        NodeWithChildren::new(
            NodeKind::Tree,
            Self::NodeType::SchemaVariant(SchemaVariantNode {
                name: self.name.to_string(),
                link: self.link.as_ref().cloned(),
                color: self.color.as_ref().cloned(),
                component_type: self.component_type,
            }),
            vec![
                Box::new(SchemaVariantChild::CommandFuncs(self.command_funcs.clone()))
                    as Box<dyn NodeChild<NodeType = Self::NodeType>>,
                Box::new(SchemaVariantChild::Domain(self.domain.clone()))
                    as Box<dyn NodeChild<NodeType = Self::NodeType>>,
                Box::new(SchemaVariantChild::LeafFunctions(
                    self.leaf_functions.clone(),
                )) as Box<dyn NodeChild<NodeType = Self::NodeType>>,
                Box::new(SchemaVariantChild::FuncDescriptions(
                    self.func_descriptions.clone(),
                )) as Box<dyn NodeChild<NodeType = Self::NodeType>>,
                Box::new(SchemaVariantChild::Workflows(self.workflows.clone()))
                    as Box<dyn NodeChild<NodeType = Self::NodeType>>,
                Box::new(SchemaVariantChild::Sockets(self.sockets.clone()))
                    as Box<dyn NodeChild<NodeType = Self::NodeType>>,
            ],
        )
    }
}
