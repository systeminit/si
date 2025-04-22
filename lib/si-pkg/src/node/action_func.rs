use std::{
    io::{BufRead, Write},
    str::FromStr,
};

use object_tree::{
    GraphError, NodeChild, NodeKind, NodeWithChildren, ReadBytes, WriteBytes, read_key_value_line,
    read_key_value_line_opt, write_key_value_line, write_key_value_line_opt,
};

use crate::{ActionFuncSpec, ActionFuncSpecKind};

use super::{PkgNode, read_common_fields, write_common_fields};

const KEY_KIND_STR: &str = "kind";
const KEY_FUNC_UNIQUE_ID_STR: &str = "func_unique_id";
const KEY_NAME_STR: &str = "name";

#[derive(Clone, Debug)]
pub struct ActionFuncNode {
    pub name: Option<String>,
    pub func_unique_id: String,
    pub kind: ActionFuncSpecKind,
    pub unique_id: Option<String>,
    pub deleted: bool,
}

impl WriteBytes for ActionFuncNode {
    fn write_bytes<W: Write>(&self, writer: &mut W) -> Result<(), GraphError> {
        write_key_value_line(writer, KEY_KIND_STR, self.kind)?;

        write_key_value_line(
            writer,
            KEY_FUNC_UNIQUE_ID_STR,
            self.func_unique_id.to_string(),
        )?;

        write_key_value_line_opt(writer, KEY_NAME_STR, self.name.as_deref())?;

        write_common_fields(writer, self.unique_id.as_deref(), self.deleted)?;

        Ok(())
    }
}

impl ReadBytes for ActionFuncNode {
    fn read_bytes<R: BufRead>(reader: &mut R) -> Result<Option<Self>, GraphError>
    where
        Self: std::marker::Sized,
    {
        let kind_str = read_key_value_line(reader, KEY_KIND_STR)?;
        let kind = ActionFuncSpecKind::from_str(&kind_str).map_err(GraphError::parse)?;

        let func_unique_id = read_key_value_line(reader, KEY_FUNC_UNIQUE_ID_STR)?;

        let name = read_key_value_line_opt(reader, KEY_NAME_STR)?;

        let (unique_id, deleted) = read_common_fields(reader)?;

        Ok(Some(Self {
            name,
            kind,
            func_unique_id,
            unique_id,
            deleted,
        }))
    }
}

impl NodeChild for ActionFuncSpec {
    type NodeType = PkgNode;

    fn as_node_with_children(&self) -> NodeWithChildren<Self::NodeType> {
        NodeWithChildren::new(
            NodeKind::Leaf,
            Self::NodeType::ActionFunc(ActionFuncNode {
                name: self.name.to_owned(),
                func_unique_id: self.func_unique_id.to_owned(),
                kind: self.kind,
                unique_id: self.unique_id.to_owned(),
                deleted: self.deleted,
            }),
            vec![],
        )
    }
}
