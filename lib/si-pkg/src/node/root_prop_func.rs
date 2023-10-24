use std::io::{BufRead, Write};
use std::str::FromStr;

use object_tree::{
    read_key_value_line, write_key_value_line, GraphError, NodeChild, NodeKind, NodeWithChildren,
    ReadBytes, WriteBytes,
};

use crate::{RootPropFuncSpec, SchemaVariantSpecPropRoot};

use super::{read_common_fields, write_common_fields, PkgNode};

const KEY_PROP_STR: &str = "PROP";
const KEY_FUNC_UNIQUE_ID_STR: &str = "func_unique_id";

#[derive(Clone, Debug)]
pub struct RootPropFuncNode {
    pub prop: SchemaVariantSpecPropRoot,
    pub func_unique_id: String,
    pub unique_id: Option<String>,
    pub deleted: bool,
}

impl WriteBytes for RootPropFuncNode {
    fn write_bytes<W: Write>(&self, writer: &mut W) -> Result<(), GraphError> {
        write_key_value_line(writer, KEY_PROP_STR, self.prop)?;
        write_key_value_line(
            writer,
            KEY_FUNC_UNIQUE_ID_STR,
            self.func_unique_id.to_string(),
        )?;
        write_common_fields(writer, self.unique_id.as_deref(), self.deleted)?;

        Ok(())
    }
}

impl ReadBytes for RootPropFuncNode {
    fn read_bytes<R: BufRead>(reader: &mut R) -> Result<Option<Self>, GraphError>
    where
        Self: std::marker::Sized,
    {
        let prop_str = read_key_value_line(reader, KEY_PROP_STR)?;
        let prop = SchemaVariantSpecPropRoot::from_str(&prop_str).map_err(GraphError::parse)?;
        let func_unique_id = read_key_value_line(reader, KEY_FUNC_UNIQUE_ID_STR)?;
        let (unique_id, deleted) = read_common_fields(reader)?;

        Ok(Some(Self {
            prop,
            func_unique_id,
            unique_id,
            deleted,
        }))
    }
}

impl NodeChild for RootPropFuncSpec {
    type NodeType = PkgNode;

    fn as_node_with_children(&self) -> NodeWithChildren<Self::NodeType> {
        NodeWithChildren::new(
            NodeKind::Tree,
            Self::NodeType::RootPropFunc(RootPropFuncNode {
                prop: self.prop.to_owned(),
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
