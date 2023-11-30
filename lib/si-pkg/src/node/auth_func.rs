use std::io::{BufRead, Write};

use crate::AuthenticationFuncSpec;
use object_tree::{
    read_key_value_line, read_key_value_line_opt, write_key_value_line, write_key_value_line_opt,
    GraphError, NodeChild, NodeKind, NodeWithChildren, ReadBytes, WriteBytes,
};

use super::{read_common_fields, write_common_fields, PkgNode};

const KEY_FUNC_UNIQUE_ID_STR: &str = "func_unique_id";
const KEY_NAME_STR: &str = "name";

#[derive(Clone, Debug)]
pub struct AuthFuncNode {
    pub name: Option<String>,
    pub func_unique_id: String,
    pub unique_id: Option<String>,
    pub deleted: bool,
}

impl WriteBytes for AuthFuncNode {
    fn write_bytes<W: Write>(&self, writer: &mut W) -> Result<(), GraphError> {
        write_key_value_line(
            writer,
            KEY_FUNC_UNIQUE_ID_STR,
            self.func_unique_id.to_string(),
        )?;

        write_key_value_line_opt(writer, KEY_NAME_STR, self.name.as_deref())?;

        write_common_fields(writer, self.unique_id.as_deref(), self.deleted)?;

        Ok(())
    }
}

impl ReadBytes for AuthFuncNode {
    fn read_bytes<R: BufRead>(reader: &mut R) -> Result<Option<Self>, GraphError>
    where
        Self: Sized,
    {
        let func_unique_id = read_key_value_line(reader, KEY_FUNC_UNIQUE_ID_STR)?;

        let name = read_key_value_line_opt(reader, KEY_NAME_STR)?;

        let (unique_id, deleted) = read_common_fields(reader)?;

        Ok(Some(Self {
            name,
            func_unique_id,
            unique_id,
            deleted,
        }))
    }
}

impl NodeChild for AuthenticationFuncSpec {
    type NodeType = PkgNode;

    fn as_node_with_children(&self) -> NodeWithChildren<Self::NodeType> {
        NodeWithChildren::new(
            NodeKind::Leaf,
            Self::NodeType::AuthFunc(AuthFuncNode {
                name: self.name.to_owned(),
                func_unique_id: self.func_unique_id.to_owned(),
                unique_id: self.unique_id.to_owned(),
                deleted: self.deleted,
            }),
            vec![],
        )
    }
}
