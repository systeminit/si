use std::io::{BufRead, Write};

use object_tree::{
    read_key_value_line, write_key_value_line, GraphError, NodeChild, NodeKind, NodeWithChildren,
    ReadBytes, WriteBytes,
};

use super::PkgNode;
use crate::spec::PositionSpec;

const KEY_X_STR: &str = "x";
const KEY_Y_STR: &str = "y";
const KEY_WIDTH_STR: &str = "width";
const KEY_HEIGHT_STR: &str = "height";

#[derive(Clone, Debug)]
pub struct PositionNode {
    pub x: String,
    pub y: String,
    pub width: String,
    pub height: String,
}

impl WriteBytes for PositionNode {
    fn write_bytes<W: Write>(&self, writer: &mut W) -> Result<(), GraphError> {
        write_key_value_line(writer, KEY_X_STR, &self.x)?;
        write_key_value_line(writer, KEY_Y_STR, &self.y)?;
        write_key_value_line(writer, KEY_WIDTH_STR, &self.width)?;
        write_key_value_line(writer, KEY_HEIGHT_STR, &self.height)?;

        Ok(())
    }
}

impl ReadBytes for PositionNode {
    fn read_bytes<R: BufRead>(reader: &mut R) -> Result<Option<Self>, GraphError>
    where
        Self: std::marker::Sized,
    {
        let x = read_key_value_line(reader, KEY_X_STR)?;
        let y = read_key_value_line(reader, KEY_Y_STR)?;
        let width = read_key_value_line(reader, KEY_WIDTH_STR)?;
        let height = read_key_value_line(reader, KEY_HEIGHT_STR)?;

        Ok(Some(Self {
            x,
            y,
            width,
            height,
        }))
    }
}

impl NodeChild for PositionSpec {
    type NodeType = PkgNode;

    fn as_node_with_children(&self) -> NodeWithChildren<Self::NodeType> {
        NodeWithChildren::new(
            NodeKind::Leaf,
            Self::NodeType::Position(PositionNode {
                x: self.x.to_owned(),
                y: self.y.to_owned(),
                width: self.width.to_owned(),
                height: self.height.to_owned(),
            }),
            vec![],
        )
    }
}
