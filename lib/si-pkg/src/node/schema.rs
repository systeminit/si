use std::{
    io::{BufRead, Write},
    str::FromStr,
};

use object_tree::{
    read_key_value_line, read_key_value_line_opt, write_key_value_line, GraphError, NameStr,
    NodeChild, NodeKind, NodeWithChildren, ReadBytes, WriteBytes,
};

use crate::SchemaSpec;

use super::{read_common_fields, write_common_fields, PkgNode};

const KEY_CATEGORY_STR: &str = "category";
const KEY_CATEGORY_NAME_STR: &str = "category_name";
const KEY_NAME_STR: &str = "name";
const KEY_UI_HIDDEN_STR: &str = "ui_hidden";

#[derive(Clone, Debug)]
pub struct SchemaData {
    pub name: String,
    pub category: String,
    pub category_name: Option<String>,
    pub ui_hidden: bool,
}

#[derive(Clone, Debug)]
pub struct SchemaNode {
    pub name: String,
    pub data: Option<SchemaData>,
    pub unique_id: Option<String>,
    pub deleted: bool,
}

impl NameStr for SchemaNode {
    fn name(&self) -> &str {
        &self.name
    }
}

impl WriteBytes for SchemaNode {
    fn write_bytes<W: Write>(&self, writer: &mut W) -> Result<(), GraphError> {
        write_key_value_line(writer, KEY_NAME_STR, self.name())?;

        if let Some(data) = &self.data {
            write_key_value_line(writer, KEY_CATEGORY_STR, &data.category)?;
            write_key_value_line(
                writer,
                KEY_CATEGORY_NAME_STR,
                data.category_name.as_deref().unwrap_or(""),
            )?;
            write_key_value_line(writer, KEY_UI_HIDDEN_STR, data.ui_hidden)?;
        }

        write_common_fields(writer, self.unique_id.as_deref(), self.deleted)?;

        Ok(())
    }
}

impl ReadBytes for SchemaNode {
    fn read_bytes<R: BufRead>(reader: &mut R) -> Result<Option<Self>, GraphError>
    where
        Self: std::marker::Sized,
    {
        let name = read_key_value_line(reader, KEY_NAME_STR)?;
        let data = match read_key_value_line_opt(reader, KEY_CATEGORY_STR)? {
            None => None,
            Some(category) => {
                let category_name_str = read_key_value_line(reader, KEY_CATEGORY_NAME_STR)?;
                let category_name = if category_name_str.is_empty() {
                    None
                } else {
                    Some(category_name_str)
                };
                let ui_hidden = bool::from_str(&read_key_value_line(reader, KEY_UI_HIDDEN_STR)?)
                    .map_err(GraphError::parse)?;

                Some(SchemaData {
                    name: name.to_owned(),
                    category,
                    category_name,
                    ui_hidden,
                })
            }
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
                unique_id: self.unique_id.as_ref().cloned(),
                deleted: self.deleted,
                data: self.data.as_ref().map(|data| SchemaData {
                    name: data.name.to_owned(),
                    category: data.category.to_owned(),
                    category_name: data.category_name.as_ref().cloned(),
                    ui_hidden: data.ui_hidden,
                }),
            }),
            children,
        )
    }
}
