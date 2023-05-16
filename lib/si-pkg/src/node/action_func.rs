use std::{
    io::{BufRead, Write},
    str::FromStr,
};

use object_tree::{
    read_key_value_line, write_key_value_line, GraphError, NodeChild, NodeKind, NodeWithChildren,
    ReadBytes, WriteBytes,
};

use crate::{ActionFuncSpec, ActionFuncSpecKind, FuncUniqueId};

use super::PkgNode;

const KEY_KIND_STR: &str = "kind";
const KEY_FUNC_UNIQUE_ID_STR: &str = "func_unique_id";

#[derive(Clone, Debug)]
pub struct ActionFuncNode {
    pub func_unique_id: FuncUniqueId,
    pub kind: ActionFuncSpecKind,
}

impl WriteBytes for ActionFuncNode {
    fn write_bytes<W: Write>(&self, writer: &mut W) -> Result<(), GraphError> {
        write_key_value_line(writer, KEY_KIND_STR, self.kind)?;

        write_key_value_line(
            writer,
            KEY_FUNC_UNIQUE_ID_STR,
            self.func_unique_id.to_string(),
        )?;

        Ok(())
    }
}

impl ReadBytes for ActionFuncNode {
    fn read_bytes<R: BufRead>(reader: &mut R) -> Result<Self, GraphError>
    where
        Self: std::marker::Sized,
    {
        let kind_str = read_key_value_line(reader, KEY_KIND_STR)?;
        let kind = ActionFuncSpecKind::from_str(&kind_str).map_err(GraphError::parse)?;

        let func_unique_id_str = read_key_value_line(reader, KEY_FUNC_UNIQUE_ID_STR)?;
        let func_unique_id =
            FuncUniqueId::from_str(&func_unique_id_str).map_err(GraphError::parse)?;

        Ok(Self {
            kind,
            func_unique_id,
        })
    }
}

impl NodeChild for ActionFuncSpec {
    type NodeType = PkgNode;

    fn as_node_with_children(&self) -> NodeWithChildren<Self::NodeType> {
        NodeWithChildren::new(
            NodeKind::Leaf,
            Self::NodeType::ActionFunc(ActionFuncNode {
                func_unique_id: self.func_unique_id,
                kind: self.kind,
            }),
            vec![],
        )
    }
}
