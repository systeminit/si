use std::{
    io::{BufRead, Write},
    str::FromStr,
};

use object_tree::{
    read_key_value_line, write_key_value_line, GraphError, NameStr, NodeChild, NodeKind,
    NodeWithChildren, ReadBytes, WriteBytes,
};

use crate::{ActionSpec, ActionSpecKind};

use super::PkgNode;

const KEY_NAME_STR: &str = "name";
const KEY_KIND_STR: &str = "kind";

#[derive(Clone, Debug)]
pub struct ActionNode {
    pub name: String,
    pub kind: ActionSpecKind,
}

impl ActionNode {
    fn kind_str(&self) -> &str {
        self.kind.as_ref()
    }
}

impl NameStr for ActionNode {
    fn name(&self) -> &str {
        &self.name
    }
}

impl WriteBytes for ActionNode {
    fn write_bytes<W: Write>(&self, writer: &mut W) -> Result<(), GraphError> {
        write_key_value_line(writer, KEY_KIND_STR, self.kind_str())?;
        write_key_value_line(writer, KEY_NAME_STR, self.name())?;

        Ok(())
    }
}

impl ReadBytes for ActionNode {
    fn read_bytes<R: BufRead>(reader: &mut R) -> Result<Self, GraphError>
    where
        Self: std::marker::Sized,
    {
        let kind = read_key_value_line(reader, KEY_KIND_STR)?;
        let name = read_key_value_line(reader, KEY_NAME_STR)?;

        Ok(Self {
            kind: ActionSpecKind::from_str(&kind).map_err(GraphError::parse)?,
            name,
        })
    }
}

impl NodeChild for ActionSpec {
    type NodeType = PkgNode;

    fn as_node_with_children(&self) -> NodeWithChildren<Self::NodeType> {
        NodeWithChildren::new(
            NodeKind::Leaf,
            Self::NodeType::Action(ActionNode {
                name: self.name.clone(),
                kind: self.kind,
            }),
            vec![],
        )
    }
}
