use std::{
    io::{BufRead, Write},
    str::FromStr,
};

use object_tree::{
    read_key_value_line, write_key_value_line, GraphError, NodeChild, NodeKind, NodeWithChildren,
    ReadBytes, WriteBytes,
};

use crate::{FuncDescriptionSpec, FuncUniqueId};

use super::PkgNode;

const KEY_FUNC_UNIQUE_ID_STR: &str = "func_unique_id";
const KEY_CONTENTS_STR: &str = "contents";

#[derive(Clone, Debug)]
pub struct FuncDescriptionNode {
    pub func_unique_id: FuncUniqueId,
    pub contents: serde_json::Value,
}

impl WriteBytes for FuncDescriptionNode {
    fn write_bytes<W: Write>(&self, writer: &mut W) -> Result<(), GraphError> {
        write_key_value_line(
            writer,
            KEY_FUNC_UNIQUE_ID_STR,
            self.func_unique_id.to_string(),
        )?;
        write_key_value_line(
            writer,
            KEY_CONTENTS_STR,
            serde_json::to_string(&self.contents).map_err(GraphError::parse)?,
        )?;

        Ok(())
    }
}

impl ReadBytes for FuncDescriptionNode {
    fn read_bytes<R: BufRead>(reader: &mut R) -> Result<Self, GraphError>
    where
        Self: std::marker::Sized,
    {
        let func_unique_id_str = read_key_value_line(reader, KEY_FUNC_UNIQUE_ID_STR)?;
        let func_unique_id =
            FuncUniqueId::from_str(&func_unique_id_str).map_err(GraphError::parse)?;

        let contents_str = read_key_value_line(reader, KEY_CONTENTS_STR)?;
        let contents: serde_json::Value =
            serde_json::from_str(&contents_str).map_err(GraphError::parse)?;

        Ok(Self {
            func_unique_id,
            contents,
        })
    }
}

impl NodeChild for FuncDescriptionSpec {
    type NodeType = PkgNode;

    fn as_node_with_children(&self) -> NodeWithChildren<Self::NodeType> {
        NodeWithChildren::new(
            NodeKind::Leaf,
            Self::NodeType::FuncDescription(FuncDescriptionNode {
                func_unique_id: self.func_unique_id,
                contents: self.contents.to_owned(),
            }),
            vec![],
        )
    }
}
