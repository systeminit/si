use std::{
    io::{BufRead, Write},
    str::FromStr,
};

use object_tree::{
    read_key_value_line, write_key_value_line, GraphError, Hash, NodeChild, NodeKind,
    NodeWithChildren, ReadBytes, WriteBytes,
};

use crate::QualificationSpec;

use super::PkgNode;

const FUNC_UNIQUE_ID_STR: &str = "func_unique_id";

#[derive(Clone, Debug)]
pub struct QualificationNode {
    pub func_unique_id: Hash,
}

impl WriteBytes for QualificationNode {
    fn write_bytes<W: Write>(&self, writer: &mut W) -> Result<(), GraphError> {
        write_key_value_line(writer, FUNC_UNIQUE_ID_STR, self.func_unique_id.to_string())?;

        Ok(())
    }
}

impl ReadBytes for QualificationNode {
    fn read_bytes<R: BufRead>(reader: &mut R) -> Result<Self, GraphError>
    where
        Self: std::marker::Sized,
    {
        let func_unique_id_str = read_key_value_line(reader, FUNC_UNIQUE_ID_STR)?;
        let func_unique_id = Hash::from_str(&func_unique_id_str)?;

        Ok(Self { func_unique_id })
    }
}

impl NodeChild for QualificationSpec {
    type NodeType = PkgNode;

    fn as_node_with_children(&self) -> NodeWithChildren<Self::NodeType> {
        NodeWithChildren::new(
            NodeKind::Leaf,
            Self::NodeType::Qualification(QualificationNode {
                func_unique_id: self.func_unique_id,
            }),
            vec![],
        )
    }
}
