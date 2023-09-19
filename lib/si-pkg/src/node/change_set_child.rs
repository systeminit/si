use serde::{Deserialize, Serialize};
use std::io::{BufRead, Write};

use object_tree::{
    read_key_value_line, write_key_value_line, GraphError, NameStr, NodeChild, NodeKind,
    NodeWithChildren, ReadBytes, WriteBytes,
};

use super::PkgNode;
use crate::{FuncSpec, SchemaSpec};

const CHANGE_SET_CHILD_TYPE_FUNCS: &str = "funcs";
const CHANGE_SET_CHILD_TYPE_SCHEMAS: &str = "schemas";

const KEY_KIND_STR: &str = "kind";

#[remain::sorted]
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ChangeSetChild {
    Funcs(Vec<FuncSpec>),
    Schemas(Vec<SchemaSpec>),
}

#[remain::sorted]
#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
pub enum ChangeSetChildNode {
    Funcs,
    Schemas,
}

impl ChangeSetChildNode {
    pub fn kind_str(&self) -> &'static str {
        match self {
            Self::Funcs => CHANGE_SET_CHILD_TYPE_FUNCS,
            Self::Schemas => CHANGE_SET_CHILD_TYPE_SCHEMAS,
        }
    }
}

impl NameStr for ChangeSetChildNode {
    fn name(&self) -> &str {
        match self {
            Self::Funcs => CHANGE_SET_CHILD_TYPE_FUNCS,
            Self::Schemas => CHANGE_SET_CHILD_TYPE_SCHEMAS,
        }
    }
}

impl WriteBytes for ChangeSetChildNode {
    fn write_bytes<W: Write>(&self, writer: &mut W) -> Result<(), GraphError> {
        write_key_value_line(writer, KEY_KIND_STR, self.kind_str())?;
        Ok(())
    }
}

impl ReadBytes for ChangeSetChildNode {
    fn read_bytes<R: BufRead>(reader: &mut R) -> Result<Option<Self>, GraphError>
    where
        Self: std::marker::Sized,
    {
        let kind_str = read_key_value_line(reader, KEY_KIND_STR)?;

        let node = match kind_str.as_str() {
            CHANGE_SET_CHILD_TYPE_FUNCS => Self::Funcs,
            CHANGE_SET_CHILD_TYPE_SCHEMAS => Self::Schemas,
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

impl NodeChild for ChangeSetChild {
    type NodeType = PkgNode;

    fn as_node_with_children(&self) -> NodeWithChildren<Self::NodeType> {
        match self {
            Self::Funcs(entries) => NodeWithChildren::new(
                NodeKind::Tree,
                Self::NodeType::ChangeSetChild(ChangeSetChildNode::Funcs),
                entries
                    .iter()
                    .map(|func| {
                        Box::new(func.clone()) as Box<dyn NodeChild<NodeType = Self::NodeType>>
                    })
                    .collect(),
            ),
            Self::Schemas(entries) => NodeWithChildren::new(
                NodeKind::Tree,
                Self::NodeType::ChangeSetChild(ChangeSetChildNode::Schemas),
                entries
                    .iter()
                    .map(|schema| {
                        Box::new(schema.clone()) as Box<dyn NodeChild<NodeType = Self::NodeType>>
                    })
                    .collect(),
            ),
        }
    }
}
