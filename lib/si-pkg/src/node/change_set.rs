use std::{
    io::{BufRead, Write},
    str::FromStr,
};

use object_tree::{
    read_key_value_line, write_key_value_line, GraphError, NameStr, NodeChild, NodeKind,
    NodeWithChildren, ReadBytes, WriteBytes,
};

use super::PkgNode;
use crate::{
    node::ChangeSetChild,
    spec::{ChangeSetSpec, ChangeSetSpecStatus},
};

const KEY_NAME_STR: &str = "name";
const KEY_BASED_ON_CHANGE_SET: &str = "based_on_change_set";
const KEY_STATUS: &str = "status";

#[derive(Clone, Debug)]
pub struct ChangeSetNode {
    pub name: String,
    pub based_on_change_set: Option<String>,
    pub status: ChangeSetSpecStatus,
}

impl NameStr for ChangeSetNode {
    fn name(&self) -> &str {
        &self.name
    }
}

impl WriteBytes for ChangeSetNode {
    fn write_bytes<W: Write>(&self, writer: &mut W) -> Result<(), GraphError> {
        write_key_value_line(writer, KEY_NAME_STR, self.name())?;
        write_key_value_line(
            writer,
            KEY_BASED_ON_CHANGE_SET,
            self.based_on_change_set.as_deref().unwrap_or(""),
        )?;
        write_key_value_line(writer, KEY_STATUS, self.status)?;

        Ok(())
    }
}

impl ReadBytes for ChangeSetNode {
    fn read_bytes<R: BufRead>(reader: &mut R) -> Result<Option<Self>, GraphError>
    where
        Self: std::marker::Sized,
    {
        let name = read_key_value_line(reader, KEY_NAME_STR)?;
        let based_on_change_set_str = read_key_value_line(reader, KEY_BASED_ON_CHANGE_SET)?;
        let based_on_change_set = if based_on_change_set_str.is_empty() {
            None
        } else {
            Some(based_on_change_set_str.to_owned())
        };

        let status_str = read_key_value_line(reader, KEY_STATUS)?;
        let status = ChangeSetSpecStatus::from_str(&status_str).map_err(GraphError::parse)?;

        Ok(Some(Self {
            name,
            based_on_change_set,
            status,
        }))
    }
}

impl NodeChild for ChangeSetSpec {
    type NodeType = PkgNode;

    fn as_node_with_children(&self) -> NodeWithChildren<Self::NodeType> {
        NodeWithChildren::new(
            NodeKind::Tree,
            Self::NodeType::ChangeSet(ChangeSetNode {
                name: self.name.to_owned(),
                status: self.status,
                based_on_change_set: self.based_on_change_set.to_owned(),
            }),
            vec![
                Box::new(ChangeSetChild::Schemas(self.schemas.clone()))
                    as Box<dyn NodeChild<NodeType = Self::NodeType>>,
                Box::new(ChangeSetChild::Funcs(self.funcs.clone()))
                    as Box<dyn NodeChild<NodeType = Self::NodeType>>,
            ],
        )
    }
}
