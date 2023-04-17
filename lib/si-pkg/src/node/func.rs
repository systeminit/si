use super::PkgNode;
use crate::spec::{FuncSpec, FuncSpecBackendKind, FuncSpecBackendResponseType};
use object_tree::{
    read_key_value_line, write_key_value_line, GraphError, Hash, NameStr, NodeChild, NodeKind,
    NodeWithChildren, ReadBytes, WriteBytes,
};
use std::io::{BufRead, Write};
use std::str::FromStr;
use url::Url;

const KEY_NAME_STR: &str = "name";
const KEY_DISPLAY_NAME_STR: &str = "display_name";
const KEY_DESCRIPTION_STR: &str = "description";
const KEY_HANDLER_STR: &str = "handler";
const KEY_CODE_STR: &str = "code_base64";
const KEY_BACKEND_KIND_STR: &str = "backend_kind";
const KEY_RESPONSE_TYPE_STR: &str = "response_type";
const KEY_HIDDEN_STR: &str = "hidden";
const KEY_LINK_STR: &str = "link";
const KEY_UNIQUE_ID_STR: &str = "unique_id";

#[derive(Clone, Debug)]
pub struct FuncNode {
    pub name: String,
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub handler: String,
    pub code_base64: String,
    pub backend_kind: FuncSpecBackendKind,
    pub response_type: FuncSpecBackendResponseType,
    pub hidden: bool,
    pub link: Option<Url>,
    pub unique_id: Hash,
}

impl NameStr for FuncNode {
    fn name(&self) -> &str {
        &self.name
    }
}

impl WriteBytes for FuncNode {
    fn write_bytes<W: Write>(&self, writer: &mut W) -> Result<(), GraphError> {
        write_key_value_line(writer, KEY_NAME_STR, self.name())?;
        write_key_value_line(
            writer,
            KEY_DISPLAY_NAME_STR,
            self.display_name.as_deref().unwrap_or(""),
        )?;
        write_key_value_line(
            writer,
            KEY_DESCRIPTION_STR,
            self.description.as_deref().unwrap_or(""),
        )?;
        write_key_value_line(writer, KEY_HANDLER_STR, &self.handler)?;
        write_key_value_line(writer, KEY_CODE_STR, &self.code_base64)?;
        write_key_value_line(writer, KEY_BACKEND_KIND_STR, self.backend_kind)?;
        write_key_value_line(writer, KEY_RESPONSE_TYPE_STR, self.response_type)?;
        write_key_value_line(writer, KEY_HIDDEN_STR, self.hidden)?;
        write_key_value_line(
            writer,
            KEY_LINK_STR,
            self.link.as_ref().map(|l| l.as_str()).unwrap_or(""),
        )?;
        write_key_value_line(writer, KEY_UNIQUE_ID_STR, self.unique_id.to_string())?;

        Ok(())
    }
}

impl ReadBytes for FuncNode {
    fn read_bytes<R: BufRead>(reader: &mut R) -> Result<Self, GraphError>
    where
        Self: std::marker::Sized,
    {
        let name = read_key_value_line(reader, KEY_NAME_STR)?;
        let display_name_str = read_key_value_line(reader, KEY_DISPLAY_NAME_STR)?;
        let display_name = if display_name_str.is_empty() {
            None
        } else {
            Some(display_name_str)
        };
        let description_str = read_key_value_line(reader, KEY_DESCRIPTION_STR)?;
        let description = if description_str.is_empty() {
            None
        } else {
            Some(description_str)
        };
        let handler = read_key_value_line(reader, KEY_HANDLER_STR)?;
        let code_base64 = read_key_value_line(reader, KEY_CODE_STR)?;
        let backend_kind_str = read_key_value_line(reader, KEY_BACKEND_KIND_STR)?;
        let backend_kind =
            FuncSpecBackendKind::from_str(&backend_kind_str).map_err(GraphError::parse)?;
        let response_type_str = read_key_value_line(reader, KEY_RESPONSE_TYPE_STR)?;
        let response_type =
            FuncSpecBackendResponseType::from_str(&response_type_str).map_err(GraphError::parse)?;
        let hidden: bool = bool::from_str(&read_key_value_line(reader, KEY_HIDDEN_STR)?)
            .map_err(GraphError::parse)?;
        let link_str = read_key_value_line(reader, KEY_LINK_STR)?;
        let link = if link_str.is_empty() {
            None
        } else {
            Some(Url::parse(&link_str).map_err(GraphError::parse)?)
        };
        let unique_id_str = read_key_value_line(reader, KEY_UNIQUE_ID_STR)?;
        let unique_id: Hash = Hash::from_str(&unique_id_str)?;

        Ok(Self {
            name,
            display_name,
            description,
            handler,
            code_base64,
            backend_kind,
            response_type,
            hidden,
            link,
            unique_id,
        })
    }
}

impl NodeChild for FuncSpec {
    type NodeType = PkgNode;

    fn as_node_with_children(&self) -> NodeWithChildren<Self::NodeType> {
        NodeWithChildren::new(
            NodeKind::Leaf,
            Self::NodeType::Func(FuncNode {
                name: self.name.to_string(),
                display_name: self.display_name.as_ref().cloned(),
                description: self.description.as_ref().cloned(),
                handler: self.handler.to_string(),
                code_base64: self.code_base64.to_string(),
                backend_kind: self.backend_kind,
                response_type: self.response_type,
                hidden: self.hidden,
                link: self.link.as_ref().cloned(),
                unique_id: self.unique_id,
            }),
            vec![],
        )
    }
}
