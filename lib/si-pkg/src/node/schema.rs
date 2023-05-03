use std::{
    io::{BufRead, Write},
    str::FromStr,
};

use object_tree::{
    read_key_value_line, write_key_value_line, GraphError, NameStr, NodeChild, NodeKind,
    NodeWithChildren, ReadBytes, WriteBytes,
};

use crate::SchemaSpec;

use super::PkgNode;

const KEY_CATEGORY_STR: &str = "category";
const KEY_CATEGORY_NAME_STR: &str = "category_name";
const KEY_NAME_STR: &str = "name";
const KEY_UI_HIDDEN_STR: &str = "ui_hidden";

#[derive(Clone, Debug)]
pub struct SchemaNode {
    pub name: String,
    pub category: String,
    pub category_name: Option<String>,
    pub ui_hidden: bool,
}

impl NameStr for SchemaNode {
    fn name(&self) -> &str {
        &self.name
    }
}

impl WriteBytes for SchemaNode {
    fn write_bytes<W: Write>(&self, writer: &mut W) -> Result<(), GraphError> {
        write_key_value_line(writer, KEY_NAME_STR, self.name())?;
        write_key_value_line(writer, KEY_CATEGORY_STR, &self.category)?;
        write_key_value_line(
            writer,
            KEY_CATEGORY_NAME_STR,
            self.category_name.as_deref().unwrap_or(""),
        )?;
        write_key_value_line(writer, KEY_UI_HIDDEN_STR, self.ui_hidden)?;

        Ok(())
    }
}

impl ReadBytes for SchemaNode {
    fn read_bytes<R: BufRead>(reader: &mut R) -> Result<Self, GraphError>
    where
        Self: std::marker::Sized,
    {
        let name = read_key_value_line(reader, KEY_NAME_STR)?;
        let category = read_key_value_line(reader, KEY_CATEGORY_STR)?;
        let category_name_str = read_key_value_line(reader, KEY_CATEGORY_NAME_STR)?;
        let category_name = if category_name_str.is_empty() {
            None
        } else {
            Some(category_name_str)
        };
        let ui_hidden = bool::from_str(&read_key_value_line(reader, KEY_UI_HIDDEN_STR)?)
            .map_err(GraphError::parse)?;

        Ok(Self {
            name,
            category,
            category_name,
            ui_hidden,
        })
    }
}

impl NodeChild for SchemaSpec {
    type NodeType = PkgNode;

    fn as_node_with_children(&self) -> NodeWithChildren<Self::NodeType> {
        let mut children = Vec::new();
        for entry in &self.variants {
            children.push(Box::new(entry.clone()) as Box<dyn NodeChild<NodeType = Self::NodeType>>);
        }

        NodeWithChildren::new(
            NodeKind::Tree,
            Self::NodeType::Schema(SchemaNode {
                name: self.name.to_string(),
                category: self.category.to_string(),
                category_name: self.category_name.clone(),
                ui_hidden: self.ui_hidden,
            }),
            children,
        )
    }
}
