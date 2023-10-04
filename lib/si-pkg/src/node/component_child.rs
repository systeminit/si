use serde::{Deserialize, Serialize};
use std::io::{BufRead, Write};

use object_tree::{
    read_key_value_line, write_key_value_line, GraphError, NameStr, NodeChild, NodeKind,
    NodeWithChildren, ReadBytes, WriteBytes,
};

use super::PkgNode;
use crate::{AttributeValueSpec, PositionSpec};

const CHANGE_SET_CHILD_TYPE_ATTRIBUTES: &str = "attributes";
const CHANGE_SET_CHILD_TYPE_INPUT_SOCKETS: &str = "input_sockets";
const CHANGE_SET_CHILD_TYPE_OUTPUT_SOCKETS: &str = "output_sockets";
const CHANGE_SET_CHILD_TYPE_POSITION: &str = "position";

const KEY_KIND_STR: &str = "kind";

#[remain::sorted]
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ComponentChild {
    Attributes(Vec<AttributeValueSpec>),
    InputSockets(Vec<AttributeValueSpec>),
    OutputSockets(Vec<AttributeValueSpec>),
    Position(PositionSpec),
}

#[remain::sorted]
#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
pub enum ComponentChildNode {
    Attributes,
    InputSockets,
    OutputSockets,
    Position,
}

impl ComponentChildNode {
    pub fn kind_str(&self) -> &'static str {
        match self {
            Self::Attributes => CHANGE_SET_CHILD_TYPE_ATTRIBUTES,
            Self::InputSockets => CHANGE_SET_CHILD_TYPE_INPUT_SOCKETS,
            Self::OutputSockets => CHANGE_SET_CHILD_TYPE_OUTPUT_SOCKETS,
            Self::Position => CHANGE_SET_CHILD_TYPE_POSITION,
        }
    }
}

impl NameStr for ComponentChildNode {
    fn name(&self) -> &str {
        match self {
            Self::Attributes => CHANGE_SET_CHILD_TYPE_ATTRIBUTES,
            Self::InputSockets => CHANGE_SET_CHILD_TYPE_INPUT_SOCKETS,
            Self::OutputSockets => CHANGE_SET_CHILD_TYPE_OUTPUT_SOCKETS,
            Self::Position => CHANGE_SET_CHILD_TYPE_POSITION,
        }
    }
}

impl WriteBytes for ComponentChildNode {
    fn write_bytes<W: Write>(&self, writer: &mut W) -> Result<(), GraphError> {
        write_key_value_line(writer, KEY_KIND_STR, self.kind_str())?;
        Ok(())
    }
}

impl ReadBytes for ComponentChildNode {
    fn read_bytes<R: BufRead>(reader: &mut R) -> Result<Option<Self>, GraphError>
    where
        Self: std::marker::Sized,
    {
        let kind_str = read_key_value_line(reader, KEY_KIND_STR)?;

        let node = match kind_str.as_str() {
            CHANGE_SET_CHILD_TYPE_ATTRIBUTES => Self::Attributes,
            CHANGE_SET_CHILD_TYPE_INPUT_SOCKETS => Self::InputSockets,
            CHANGE_SET_CHILD_TYPE_OUTPUT_SOCKETS => Self::OutputSockets,
            CHANGE_SET_CHILD_TYPE_POSITION => Self::Position,
            invalid_kind => {
                dbg!(format!(
                    "invalid change set child node kind: {invalid_kind}"
                ));
                return Ok(None);
            }
        };

        Ok(Some(node))
    }
}

impl NodeChild for ComponentChild {
    type NodeType = PkgNode;

    fn as_node_with_children(&self) -> NodeWithChildren<Self::NodeType> {
        match self {
            Self::Attributes(entries) => NodeWithChildren::new(
                NodeKind::Tree,
                Self::NodeType::ComponentChild(ComponentChildNode::Attributes),
                entries
                    .iter()
                    .map(|attr| {
                        Box::new(attr.to_owned()) as Box<dyn NodeChild<NodeType = Self::NodeType>>
                    })
                    .collect(),
            ),
            Self::InputSockets(entries) => NodeWithChildren::new(
                NodeKind::Tree,
                Self::NodeType::ComponentChild(ComponentChildNode::InputSockets),
                entries
                    .iter()
                    .map(|input| {
                        Box::new(input.to_owned()) as Box<dyn NodeChild<NodeType = Self::NodeType>>
                    })
                    .collect(),
            ),
            Self::OutputSockets(entries) => NodeWithChildren::new(
                NodeKind::Tree,
                Self::NodeType::ComponentChild(ComponentChildNode::OutputSockets),
                entries
                    .iter()
                    .map(|output| {
                        Box::new(output.to_owned()) as Box<dyn NodeChild<NodeType = Self::NodeType>>
                    })
                    .collect(),
            ),
            Self::Position(position) => NodeWithChildren::new(
                NodeKind::Tree,
                Self::NodeType::ComponentChild(ComponentChildNode::Position),
                vec![Box::new(position.to_owned()) as Box<dyn NodeChild<NodeType = Self::NodeType>>],
            ),
        }
    }
}
