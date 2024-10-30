use std::{
    collections::HashSet,
    io::{BufRead, Write},
};

use object_tree::{
    read_key_value_line, read_key_value_line_opt, write_key_value_line, write_key_value_line_opt,
    GraphError, NodeChild, NodeKind, NodeWithChildren, ReadBytes, WriteBytes,
};

use crate::ManagementFuncSpec;

use super::PkgNode;

const KEY_FUNC_UNIQUE_ID_STR: &str = "func_unique_id";
const KEY_NAME_STR: &str = "name";
const KEY_DESCRIPTION_STR: &str = "description";
const KEY_MANAGED_SCHEMAS_STR: &str = "managed_schemas";

#[derive(Clone, Debug)]
pub struct ManagementFuncNode {
    pub func_unique_id: String,
    pub name: String,
    pub description: Option<String>,
    pub managed_schemas: Option<HashSet<String>>,
}

impl WriteBytes for ManagementFuncNode {
    fn write_bytes<W: Write>(&self, writer: &mut W) -> Result<(), GraphError> {
        write_key_value_line(
            writer,
            KEY_FUNC_UNIQUE_ID_STR,
            self.func_unique_id.to_string(),
        )?;

        write_key_value_line(writer, KEY_NAME_STR, &self.name)?;
        write_key_value_line_opt(writer, KEY_DESCRIPTION_STR, self.description.as_deref())?;
        let managed_schemas_str = match self.managed_schemas.as_ref() {
            None => None,
            Some(managed_schemas) => Some(serde_json::to_string(managed_schemas)?),
        };
        write_key_value_line_opt(writer, KEY_MANAGED_SCHEMAS_STR, managed_schemas_str)?;

        Ok(())
    }
}

impl ReadBytes for ManagementFuncNode {
    fn read_bytes<R: BufRead>(reader: &mut R) -> Result<Option<Self>, GraphError>
    where
        Self: std::marker::Sized,
    {
        let func_unique_id = read_key_value_line(reader, KEY_FUNC_UNIQUE_ID_STR)?;
        let name = read_key_value_line(reader, KEY_NAME_STR)?;
        let description = read_key_value_line_opt(reader, KEY_DESCRIPTION_STR)?;
        let managed_schemas_str = read_key_value_line_opt(reader, KEY_MANAGED_SCHEMAS_STR)?;
        let managed_schemas = match managed_schemas_str {
            None => None,
            Some(managed_schemas_str) => Some(serde_json::from_str(&managed_schemas_str)?),
        };

        Ok(Some(Self {
            func_unique_id,
            name,
            description,
            managed_schemas,
        }))
    }
}

impl NodeChild for ManagementFuncSpec {
    type NodeType = PkgNode;

    fn as_node_with_children(&self) -> NodeWithChildren<Self::NodeType> {
        NodeWithChildren::new(
            NodeKind::Tree,
            Self::NodeType::ManagementFunc(ManagementFuncNode {
                name: self.name.to_owned(),
                func_unique_id: self.func_unique_id.to_owned(),
                description: self.description.to_owned(),
                managed_schemas: self.managed_schemas.to_owned(),
            }),
            vec![],
        )
    }
}
