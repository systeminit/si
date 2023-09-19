use std::io::{BufRead, Write};

use object_tree::{
    read_key_value_line, write_key_value_line, GraphError, NameStr, NodeChild, NodeKind,
    NodeWithChildren, ReadBytes, WriteBytes,
};
use serde::{Deserialize, Serialize};

use crate::{ChangeSetSpec, FuncSpec, SchemaSpec};

use super::PkgNode;

const CATEGORY_TYPE_CHANGE_SETS: &str = "change_sets";
const CATEGORY_TYPE_SCHEMAS: &str = "schemas";
const CATEGORY_TYPE_FUNCS: &str = "funcs";

const KEY_KIND_STR: &str = "kind";

#[remain::sorted]
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum PackageCategory {
    ChangeSets(Vec<ChangeSetSpec>),
    Funcs(Vec<FuncSpec>),
    Schemas(Vec<SchemaSpec>),
}

#[remain::sorted]
#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
pub enum CategoryNode {
    ChangeSets,
    Funcs,
    Schemas,
}

impl CategoryNode {
    pub fn kind_str(&self) -> &'static str {
        match self {
            Self::ChangeSets => CATEGORY_TYPE_CHANGE_SETS,
            Self::Funcs => CATEGORY_TYPE_FUNCS,
            Self::Schemas => CATEGORY_TYPE_SCHEMAS,
        }
    }
}

impl NameStr for CategoryNode {
    fn name(&self) -> &str {
        match self {
            Self::ChangeSets => CATEGORY_TYPE_CHANGE_SETS,
            Self::Schemas => CATEGORY_TYPE_SCHEMAS,
            Self::Funcs => CATEGORY_TYPE_FUNCS,
        }
    }
}

impl WriteBytes for CategoryNode {
    fn write_bytes<W: Write>(&self, writer: &mut W) -> Result<(), GraphError> {
        write_key_value_line(writer, KEY_KIND_STR, self.kind_str())?;
        Ok(())
    }
}

impl ReadBytes for CategoryNode {
    fn read_bytes<R: BufRead>(reader: &mut R) -> Result<Option<Self>, GraphError>
    where
        Self: std::marker::Sized,
    {
        let kind_str = read_key_value_line(reader, KEY_KIND_STR)?;

        let node = match kind_str.as_str() {
            CATEGORY_TYPE_CHANGE_SETS => Self::ChangeSets,
            CATEGORY_TYPE_FUNCS => Self::Funcs,
            CATEGORY_TYPE_SCHEMAS => Self::Schemas,
            invalid_kind => {
                dbg!(format!(
                    "invalid package category node kind: {invalid_kind}"
                ));
                return Ok(None);
            }
        };

        Ok(Some(node))
    }
}

impl NodeChild for PackageCategory {
    type NodeType = PkgNode;

    fn as_node_with_children(&self) -> NodeWithChildren<Self::NodeType> {
        match self {
            Self::ChangeSets(entries) => NodeWithChildren::new(
                NodeKind::Tree,
                Self::NodeType::Category(CategoryNode::ChangeSets),
                entries
                    .iter()
                    .map(|cs| Box::new(cs.clone()) as Box<dyn NodeChild<NodeType = Self::NodeType>>)
                    .collect(),
            ),
            Self::Funcs(entries) => NodeWithChildren::new(
                NodeKind::Tree,
                Self::NodeType::Category(CategoryNode::Funcs),
                entries
                    .iter()
                    .map(|func| {
                        Box::new(func.clone()) as Box<dyn NodeChild<NodeType = Self::NodeType>>
                    })
                    .collect(),
            ),
            Self::Schemas(entries) => NodeWithChildren::new(
                NodeKind::Tree,
                Self::NodeType::Category(CategoryNode::Schemas),
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
