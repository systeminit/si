use std::{
    io::{BufRead, Write},
    str::FromStr,
};

use object_tree::{
    read_key_value_line, write_key_value_line, GraphError, NodeChild, NodeKind, NodeWithChildren,
    ReadBytes, WriteBytes,
};

use crate::{FuncUniqueId, WorkflowSpec};

use super::PkgNode;

const KEY_FUNC_UNIQUE_ID_STR: &str = "func_unique_id";
const KEY_TITLE_STR: &str = "title";

#[derive(Clone, Debug)]
pub struct WorkflowNode {
    pub func_unique_id: FuncUniqueId,
    pub title: String,
}

impl WriteBytes for WorkflowNode {
    fn write_bytes<W: Write>(&self, writer: &mut W) -> Result<(), GraphError> {
        write_key_value_line(
            writer,
            KEY_FUNC_UNIQUE_ID_STR,
            self.func_unique_id.to_string(),
        )?;
        write_key_value_line(writer, KEY_TITLE_STR, &self.title)?;

        Ok(())
    }
}

impl ReadBytes for WorkflowNode {
    fn read_bytes<R: BufRead>(reader: &mut R) -> Result<Self, GraphError>
    where
        Self: std::marker::Sized,
    {
        let func_unique_id_str = read_key_value_line(reader, KEY_FUNC_UNIQUE_ID_STR)?;
        let title = read_key_value_line(reader, KEY_TITLE_STR)?;

        Ok(Self {
            func_unique_id: FuncUniqueId::from_str(&func_unique_id_str)
                .map_err(GraphError::parse)?,
            title,
        })
    }
}

impl NodeChild for WorkflowSpec {
    type NodeType = PkgNode;

    fn as_node_with_children(&self) -> NodeWithChildren<Self::NodeType> {
        NodeWithChildren::new(
            NodeKind::Tree,
            Self::NodeType::Workflow(WorkflowNode {
                func_unique_id: self.func_unique_id,
                title: self.title.clone(),
            }),
            self.actions
                .iter()
                .map(|action| {
                    Box::new(action.clone()) as Box<dyn NodeChild<NodeType = Self::NodeType>>
                })
                .collect(),
        )
    }
}
