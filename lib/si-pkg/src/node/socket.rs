use std::{
    io::{BufRead, Write},
    str::FromStr,
};

use object_tree::{
    GraphError, NameStr, NodeChild, NodeKind, NodeWithChildren, ReadBytes, WriteBytes,
    read_key_value_line, read_key_value_line_opt, write_key_value_line,
};

use crate::{SocketSpec, SocketSpecArity, SocketSpecKind};

use super::{PkgNode, read_unique_id, write_unique_id};

const KEY_KIND_STR: &str = "kind";
const KEY_NAME_STR: &str = "name";
const KEY_CONNECTION_ANNOTATIONS_STR: &str = "type";
const KEY_ARITY_STR: &str = "arity";
const KEY_FUNC_UNIQUE_ID_STR: &str = "func_unique_id";
const KEY_UI_HIDDEN_STR: &str = "ui_hidden";

#[derive(Clone, Debug)]
pub struct SocketData {
    pub name: String,
    pub connection_annotations: String,
    pub kind: SocketSpecKind,
    pub arity: SocketSpecArity,
    pub func_unique_id: Option<String>,
    pub ui_hidden: bool,
}

#[derive(Clone, Debug)]
pub struct SocketNode {
    pub name: String,
    pub data: Option<SocketData>,
    pub unique_id: Option<String>,
}

impl NameStr for SocketNode {
    fn name(&self) -> &str {
        &self.name
    }
}

impl WriteBytes for SocketNode {
    fn write_bytes<W: Write>(&self, writer: &mut W) -> Result<(), GraphError> {
        write_key_value_line(writer, KEY_NAME_STR, &self.name)?;

        if let Some(data) = &self.data {
            // KEY_KIND_STR string should be the first data field to be serialized,
            // since we use it to detect if the payload has a data field in read_bytes below
            write_key_value_line(writer, KEY_KIND_STR, data.kind)?;
            write_key_value_line(writer, KEY_ARITY_STR, data.arity)?;
            write_key_value_line(
                writer,
                KEY_CONNECTION_ANNOTATIONS_STR,
                data.connection_annotations.clone(),
            )?;

            write_key_value_line(
                writer,
                KEY_FUNC_UNIQUE_ID_STR,
                data.func_unique_id.as_deref().unwrap_or(""),
            )?;
            write_key_value_line(writer, KEY_UI_HIDDEN_STR, data.ui_hidden)?;
        }

        write_unique_id(writer, self.unique_id.as_deref())?;

        Ok(())
    }
}

impl ReadBytes for SocketNode {
    fn read_bytes<R: BufRead>(reader: &mut R) -> Result<Option<Self>, GraphError>
    where
        Self: std::marker::Sized,
    {
        let name = read_key_value_line(reader, KEY_NAME_STR)?;

        let data = match read_key_value_line_opt(reader, KEY_KIND_STR)? {
            None => None,
            Some(kind_str) => {
                let kind = SocketSpecKind::from_str(&kind_str).map_err(GraphError::parse)?;

                let arity_str = read_key_value_line(reader, KEY_ARITY_STR)?;
                let arity = SocketSpecArity::from_str(&arity_str).map_err(GraphError::parse)?;

                // We have to check if the type_string has been set for backwards compatibility
                let connection_annotations =
                    match read_key_value_line_opt(reader, KEY_CONNECTION_ANNOTATIONS_STR)? {
                        None => serde_json::to_string(&vec![name.to_owned()])?,

                        Some(s) => s,
                    };

                let func_unique_id_str = read_key_value_line(reader, KEY_FUNC_UNIQUE_ID_STR)?;
                let func_unique_id = if func_unique_id_str.is_empty() {
                    None
                } else {
                    Some(func_unique_id_str)
                };

                let ui_hidden = bool::from_str(&read_key_value_line(reader, KEY_UI_HIDDEN_STR)?)
                    .map_err(GraphError::parse)?;

                Some(SocketData {
                    name: name.to_owned(),
                    connection_annotations,
                    kind,
                    arity,
                    func_unique_id,
                    ui_hidden,
                })
            }
        };

        let unique_id = read_unique_id(reader)?;

        Ok(Some(Self {
            name,
            data,
            unique_id,
        }))
    }
}

impl NodeChild for SocketSpec {
    type NodeType = PkgNode;

    fn as_node_with_children(&self) -> NodeWithChildren<Self::NodeType> {
        NodeWithChildren::new(
            NodeKind::Tree,
            Self::NodeType::Socket(SocketNode {
                name: self.name.to_owned(),
                data: self.data.as_ref().map(|data| SocketData {
                    name: self.name.to_owned(),
                    connection_annotations: data.connection_annotations.to_owned(),
                    kind: data.kind,
                    arity: data.arity,
                    func_unique_id: data.func_unique_id.to_owned(),
                    ui_hidden: data.ui_hidden,
                }),
                unique_id: self.unique_id.to_owned(),
            }),
            self.inputs
                .iter()
                .map(|input| {
                    Box::new(input.clone()) as Box<dyn NodeChild<NodeType = Self::NodeType>>
                })
                .collect(),
        )
    }
}
