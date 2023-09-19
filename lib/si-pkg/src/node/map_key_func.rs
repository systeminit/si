use std::io::{BufRead, Write};

use object_tree::{
    read_key_value_line, write_key_value_line, GraphError, NodeChild, NodeKind, NodeWithChildren,
    ReadBytes, WriteBytes,
};

use crate::MapKeyFuncSpec;

use super::PkgNode;

const KEY_KEY_STR: &str = "key";
const KEY_FUNC_UNIQUE_ID_STR: &str = "func_unique_id";

#[derive(Clone, Debug)]
pub struct MapKeyFuncNode {
    pub key: String,
    pub func_unique_id: String,
}

impl WriteBytes for MapKeyFuncNode {
    fn write_bytes<W: Write>(&self, writer: &mut W) -> Result<(), GraphError> {
        write_key_value_line(writer, KEY_KEY_STR, &self.key)?;
        write_key_value_line(
            writer,
            KEY_FUNC_UNIQUE_ID_STR,
            self.func_unique_id.to_string(),
        )?;

        Ok(())
    }
}

impl ReadBytes for MapKeyFuncNode {
    fn read_bytes<R: BufRead>(reader: &mut R) -> Result<Option<Self>, GraphError>
    where
        Self: std::marker::Sized,
    {
        let key = read_key_value_line(reader, KEY_KEY_STR)?;
        let func_unique_id = read_key_value_line(reader, KEY_FUNC_UNIQUE_ID_STR)?;

        Ok(Some(Self {
            key,
            func_unique_id,
        }))
    }
}

impl NodeChild for MapKeyFuncSpec {
    type NodeType = PkgNode;

    fn as_node_with_children(&self) -> NodeWithChildren<Self::NodeType> {
        NodeWithChildren::new(
            NodeKind::Tree,
            Self::NodeType::MapKeyFunc(MapKeyFuncNode {
                key: self.key.to_owned(),
                func_unique_id: self.func_unique_id.to_owned(),
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
