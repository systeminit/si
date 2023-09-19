use std::{
    io::{BufRead, Write},
    str::FromStr,
};

use object_tree::{
    read_key_value_line, write_key_value_line, GraphError, NodeChild, NodeKind, NodeWithChildren,
    ReadBytes, WriteBytes,
};

use crate::{SiPropFuncSpec, SiPropFuncSpecKind};

use super::{read_common_fields, write_common_fields, PkgNode};

const KEY_KIND_STR: &str = "kind";
const KEY_FUNC_UNIQUE_ID_STR: &str = "func_unique_id";

#[derive(Clone, Debug)]
pub struct SiPropFuncNode {
    pub kind: SiPropFuncSpecKind,
    pub func_unique_id: String,
    pub unique_id: Option<String>,
    pub deleted: bool,
}

impl WriteBytes for SiPropFuncNode {
    fn write_bytes<W: Write>(&self, writer: &mut W) -> Result<(), GraphError> {
        write_key_value_line(writer, KEY_KIND_STR, self.kind)?;
        write_key_value_line(
            writer,
            KEY_FUNC_UNIQUE_ID_STR,
            self.func_unique_id.to_string(),
        )?;
        write_common_fields(writer, self.unique_id.as_deref(), self.deleted)?;

        Ok(())
    }
}

impl ReadBytes for SiPropFuncNode {
    fn read_bytes<R: BufRead>(reader: &mut R) -> Result<Option<Self>, GraphError>
    where
        Self: std::marker::Sized,
    {
        let kind_str = read_key_value_line(reader, KEY_KIND_STR)?;
        let kind = SiPropFuncSpecKind::from_str(&kind_str).map_err(GraphError::parse)?;

        let func_unique_id = read_key_value_line(reader, KEY_FUNC_UNIQUE_ID_STR)?;

        let (unique_id, deleted) = read_common_fields(reader)?;

        Ok(Some(Self {
            kind,
            func_unique_id,
            unique_id,
            deleted,
        }))
    }
}

impl NodeChild for SiPropFuncSpec {
    type NodeType = PkgNode;

    fn as_node_with_children(&self) -> NodeWithChildren<Self::NodeType> {
        NodeWithChildren::new(
            NodeKind::Tree,
            Self::NodeType::SiPropFunc(SiPropFuncNode {
                kind: self.kind,
                func_unique_id: self.func_unique_id.to_owned(),
                unique_id: self.unique_id.to_owned(),
                deleted: self.deleted,
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
