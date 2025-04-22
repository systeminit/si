use std::{
    io::{BufRead, Write},
    str::FromStr,
};

use object_tree::{
    GraphError, NodeChild, NodeKind, NodeWithChildren, ReadBytes, WriteBytes, read_key_value_line,
    read_key_value_line_opt, write_key_value_line,
};

use crate::{LeafFunctionSpec, LeafInputLocation, LeafKind};

use super::{PkgNode, read_common_fields, write_common_fields};

const FUNC_UNIQUE_ID_STR: &str = "func_unique_id";
const LEAF_KIND_STR: &str = "leaf_kind";
const INPUT_DOMAIN_STR: &str = "input_domain";
const INPUT_SECRET_STR: &str = "input_secret";
const INPUT_DELETED_AT_STR: &str = "input_deleted_at";
const INPUT_CODE_STR: &str = "input_code";
const INPUT_RESOURCE_STR: &str = "input_resource";

#[derive(Clone, Debug)]
pub struct LeafFunctionNode {
    pub func_unique_id: String,
    pub leaf_kind: LeafKind,
    pub input_code: bool,
    pub input_deleted_at: bool,
    pub input_secret: bool,
    pub input_domain: bool,
    pub input_resource: bool,
    pub unique_id: Option<String>,
    pub deleted: bool,
}

impl WriteBytes for LeafFunctionNode {
    fn write_bytes<W: Write>(&self, writer: &mut W) -> Result<(), GraphError> {
        write_key_value_line(writer, LEAF_KIND_STR, self.leaf_kind)?;
        write_key_value_line(writer, FUNC_UNIQUE_ID_STR, self.func_unique_id.to_string())?;
        write_key_value_line(writer, INPUT_CODE_STR, self.input_code)?;
        write_key_value_line(writer, INPUT_DOMAIN_STR, self.input_domain)?;
        write_key_value_line(writer, INPUT_DELETED_AT_STR, self.input_deleted_at)?;
        write_key_value_line(writer, INPUT_RESOURCE_STR, self.input_resource)?;
        write_key_value_line(writer, INPUT_SECRET_STR, self.input_secret)?;

        write_common_fields(writer, self.unique_id.as_deref(), self.deleted)?;

        Ok(())
    }
}

impl ReadBytes for LeafFunctionNode {
    fn read_bytes<R: BufRead>(reader: &mut R) -> Result<Option<Self>, GraphError>
    where
        Self: std::marker::Sized,
    {
        let leaf_kind_str = read_key_value_line(reader, LEAF_KIND_STR)?;
        let leaf_kind = LeafKind::from_str(&leaf_kind_str).map_err(GraphError::parse)?;
        let func_unique_id = read_key_value_line(reader, FUNC_UNIQUE_ID_STR)?;
        let input_code = bool::from_str(&read_key_value_line(reader, INPUT_CODE_STR)?)
            .map_err(GraphError::parse)?;
        let input_domain = bool::from_str(&read_key_value_line(reader, INPUT_DOMAIN_STR)?)
            .map_err(GraphError::parse)?;
        let input_deleted_at = bool::from_str(&read_key_value_line(reader, INPUT_DELETED_AT_STR)?)
            .map_err(GraphError::parse)?;
        let input_resource = bool::from_str(&read_key_value_line(reader, INPUT_RESOURCE_STR)?)
            .map_err(GraphError::parse)?;

        let (unique_id, deleted) = read_common_fields(reader)?;

        let input_secret;
        let maybe_secret = read_key_value_line_opt(reader, INPUT_SECRET_STR)?;
        if let Some(secret) = maybe_secret {
            input_secret = bool::from_str(secret.as_str()).map_err(GraphError::parse)?;
        } else {
            input_secret = false
        }

        Ok(Some(Self {
            func_unique_id,
            leaf_kind,
            input_code,
            input_domain,
            input_secret,
            input_deleted_at,
            input_resource,
            unique_id,
            deleted,
        }))
    }
}

impl NodeChild for LeafFunctionSpec {
    type NodeType = PkgNode;

    fn as_node_with_children(&self) -> NodeWithChildren<Self::NodeType> {
        NodeWithChildren::new(
            NodeKind::Leaf,
            Self::NodeType::LeafFunction(LeafFunctionNode {
                func_unique_id: self.func_unique_id.to_owned(),
                leaf_kind: self.leaf_kind,
                input_code: self.inputs.contains(&LeafInputLocation::Code),
                input_deleted_at: self.inputs.contains(&LeafInputLocation::DeletedAt),
                input_domain: self.inputs.contains(&LeafInputLocation::Domain),
                input_resource: self.inputs.contains(&LeafInputLocation::Resource),
                unique_id: self.unique_id.to_owned(),
                deleted: self.deleted,
                input_secret: self.inputs.contains(&LeafInputLocation::Secrets),
            }),
            vec![],
        )
    }
}
