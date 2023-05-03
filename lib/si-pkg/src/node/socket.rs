use std::{
    io::{BufRead, Write},
    str::FromStr,
};

use object_tree::{
    read_key_value_line, write_key_value_line, GraphError, NameStr, NodeChild, NodeKind,
    NodeWithChildren, ReadBytes, WriteBytes,
};

use crate::{FuncUniqueId, SocketSpec, SocketSpecArity, SocketSpecKind};

use super::PkgNode;

const KEY_KIND_STR: &str = "kind";
const KEY_NAME_STR: &str = "name";
const KEY_ARITY_STR: &str = "arity";
const KEY_FUNC_UNIQUE_ID_STR: &str = "func_unique_id";
const KEY_UI_HIDDEN_STR: &str = "ui_hidden";

#[derive(Clone, Debug)]
pub struct SocketNode {
    pub func_unique_id: Option<FuncUniqueId>,
    pub name: String,
    pub kind: SocketSpecKind,
    pub arity: SocketSpecArity,
    pub ui_hidden: bool,
}

impl NameStr for SocketNode {
    fn name(&self) -> &str {
        &self.name
    }
}

impl WriteBytes for SocketNode {
    fn write_bytes<W: Write>(&self, writer: &mut W) -> Result<(), GraphError> {
        write_key_value_line(writer, KEY_NAME_STR, &self.name)?;
        write_key_value_line(writer, KEY_KIND_STR, self.kind)?;
        write_key_value_line(writer, KEY_ARITY_STR, self.arity)?;

        write_key_value_line(
            writer,
            KEY_FUNC_UNIQUE_ID_STR,
            self.func_unique_id
                .map(|fuid| fuid.to_string())
                .unwrap_or("".to_string()),
        )?;

        write_key_value_line(writer, KEY_UI_HIDDEN_STR, self.ui_hidden)?;

        Ok(())
    }
}

impl ReadBytes for SocketNode {
    fn read_bytes<R: BufRead>(reader: &mut R) -> Result<Self, GraphError>
    where
        Self: std::marker::Sized,
    {
        let name = read_key_value_line(reader, KEY_NAME_STR)?;
        let kind_str = read_key_value_line(reader, KEY_KIND_STR)?;
        let kind = SocketSpecKind::from_str(&kind_str).map_err(GraphError::parse)?;

        let arity_str = read_key_value_line(reader, KEY_ARITY_STR)?;
        let arity = SocketSpecArity::from_str(&arity_str).map_err(GraphError::parse)?;

        let func_unique_id_str = read_key_value_line(reader, KEY_FUNC_UNIQUE_ID_STR)?;
        let func_unique_id = if func_unique_id_str.is_empty() {
            None
        } else {
            Some(FuncUniqueId::from_str(&func_unique_id_str).map_err(GraphError::parse)?)
        };

        let ui_hidden = bool::from_str(&read_key_value_line(reader, KEY_UI_HIDDEN_STR)?)
            .map_err(GraphError::parse)?;

        Ok(Self {
            name,
            kind,
            arity,
            func_unique_id,
            ui_hidden,
        })
    }
}

impl NodeChild for SocketSpec {
    type NodeType = PkgNode;

    fn as_node_with_children(&self) -> NodeWithChildren<Self::NodeType> {
        NodeWithChildren::new(
            NodeKind::Tree,
            Self::NodeType::Socket(SocketNode {
                func_unique_id: self.func_unique_id,
                name: self.name.clone(),
                kind: self.kind,
                arity: self.arity,
                ui_hidden: self.ui_hidden,
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
