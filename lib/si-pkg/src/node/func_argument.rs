use super::{read_common_fields, write_common_fields, PkgNode};
use crate::spec::{FuncArgumentKind, FuncArgumentSpec};
use object_tree::{
    read_key_value_line, write_key_value_line, GraphError, NameStr, NodeChild, NodeKind,
    NodeWithChildren, ReadBytes, WriteBytes,
};
use std::io::{BufRead, Write};
use std::str::FromStr;

const KEY_NAME_STR: &str = "name";
const KEY_KIND_STR: &str = "kind";
const KEY_ELEMENT_KIND_STR: &str = "element_kind";

#[derive(Clone, Debug)]
pub struct FuncArgumentNode {
    pub name: String,
    pub kind: FuncArgumentKind,
    pub element_kind: Option<FuncArgumentKind>,
    pub unique_id: Option<String>,
    pub deleted: bool,
}

impl NameStr for FuncArgumentNode {
    fn name(&self) -> &str {
        &self.name
    }
}

impl WriteBytes for FuncArgumentNode {
    fn write_bytes<W: Write>(&self, writer: &mut W) -> Result<(), GraphError> {
        write_key_value_line(writer, KEY_NAME_STR, &self.name)?;
        write_key_value_line(writer, KEY_KIND_STR, self.kind)?;
        write_key_value_line(
            writer,
            KEY_ELEMENT_KIND_STR,
            self.element_kind
                .as_ref()
                .map(|kind| kind.to_string())
                .unwrap_or("".to_string()),
        )?;

        write_common_fields(writer, self.unique_id.as_deref(), self.deleted)?;

        Ok(())
    }
}

impl ReadBytes for FuncArgumentNode {
    fn read_bytes<R: BufRead>(reader: &mut R) -> Result<Option<Self>, GraphError>
    where
        Self: std::marker::Sized,
    {
        let name = read_key_value_line(reader, KEY_NAME_STR)?;
        let kind_str = read_key_value_line(reader, KEY_KIND_STR)?;
        let kind = FuncArgumentKind::from_str(&kind_str).map_err(GraphError::parse)?;

        let element_kind_str = read_key_value_line(reader, KEY_ELEMENT_KIND_STR)?;
        let element_kind = if element_kind_str.is_empty() {
            None
        } else {
            Some(FuncArgumentKind::from_str(&element_kind_str).map_err(GraphError::parse)?)
        };

        let (unique_id, deleted) = read_common_fields(reader)?;

        Ok(Some(Self {
            name,
            kind,
            element_kind,
            unique_id,
            deleted,
        }))
    }
}

impl NodeChild for FuncArgumentSpec {
    type NodeType = PkgNode;

    fn as_node_with_children(&self) -> NodeWithChildren<Self::NodeType> {
        NodeWithChildren::new(
            NodeKind::Leaf,
            Self::NodeType::FuncArgument(FuncArgumentNode {
                name: self.name.to_string(),
                kind: self.kind,
                element_kind: self.element_kind.to_owned(),
                unique_id: self.unique_id.to_owned(),
                deleted: self.deleted,
            }),
            vec![],
        )
    }
}
