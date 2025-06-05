use std::{
    io::{
        BufRead,
        Write,
    },
    str::FromStr,
};

use chrono::{
    DateTime,
    Utc,
};
use object_tree::{
    GraphError,
    NameStr,
    NodeChild,
    NodeKind,
    NodeWithChildren,
    ReadBytes,
    WriteBytes,
    read_key_value_line,
    read_key_value_line_opt,
    write_key_value_line,
    write_key_value_line_opt,
};
use url::Url;

use super::{
    PkgNode,
    read_common_fields,
    write_common_fields,
};
use crate::spec::{
    FuncSpec,
    FuncSpecBackendKind,
    FuncSpecBackendResponseType,
};

const KEY_NAME_STR: &str = "name";
const KEY_DISPLAY_NAME_STR: &str = "display_name";
const KEY_DESCRIPTION_STR: &str = "description";
const KEY_HANDLER_STR: &str = "handler";
const KEY_CODE_STR: &str = "code_base64";
const KEY_BACKEND_KIND_STR: &str = "backend_kind";
const KEY_RESPONSE_TYPE_STR: &str = "response_type";
const KEY_HIDDEN_STR: &str = "hidden";
const KEY_LINK_STR: &str = "link";
const KEY_IS_FROM_BUILTIN: &str = "is_from_builtin";
const KEY_IS_TRANSFORMATION: &str = "is_transformation";
const KEY_LAST_UPDATED: &str = "is_transformation";

#[derive(Clone, Debug)]
pub struct FuncData {
    pub name: String,
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub handler: String,
    pub code_base64: String,
    pub backend_kind: FuncSpecBackendKind,
    pub response_type: FuncSpecBackendResponseType,
    pub hidden: bool,
    pub link: Option<Url>,
    pub is_transformation: bool,
    pub last_updated_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug)]
pub struct FuncNode {
    pub name: String,
    pub data: Option<FuncData>,
    pub unique_id: String,
    pub deleted: bool,
    pub is_from_builtin: Option<bool>,
}

impl NameStr for FuncNode {
    fn name(&self) -> &str {
        &self.name
    }
}

impl WriteBytes for FuncNode {
    fn write_bytes<W: Write>(&self, writer: &mut W) -> Result<(), GraphError> {
        write_key_value_line(writer, KEY_NAME_STR, self.name())?;
        if let Some(data) = &self.data {
            write_key_value_line(
                writer,
                KEY_DISPLAY_NAME_STR,
                data.display_name.as_deref().unwrap_or(""),
            )?;
            write_key_value_line(
                writer,
                KEY_DESCRIPTION_STR,
                data.description.as_deref().unwrap_or(""),
            )?;
            write_key_value_line(writer, KEY_HANDLER_STR, &data.handler)?;
            write_key_value_line(writer, KEY_CODE_STR, &data.code_base64)?;
            write_key_value_line(writer, KEY_BACKEND_KIND_STR, data.backend_kind)?;
            write_key_value_line(writer, KEY_RESPONSE_TYPE_STR, data.response_type)?;
            write_key_value_line(writer, KEY_HIDDEN_STR, data.hidden)?;
            write_key_value_line(
                writer,
                KEY_LINK_STR,
                data.link.as_ref().map(|l| l.as_str()).unwrap_or(""),
            )?;

            write_key_value_line_opt(writer, KEY_IS_TRANSFORMATION, Some(data.is_transformation))?;
            write_key_value_line_opt(writer, KEY_LAST_UPDATED, data.last_updated_at)?;
        }

        write_common_fields(writer, Some(self.unique_id.as_str()), self.deleted)?;
        write_key_value_line_opt(writer, KEY_IS_FROM_BUILTIN, self.is_from_builtin)?;

        Ok(())
    }
}

impl ReadBytes for FuncNode {
    fn read_bytes<R: BufRead>(reader: &mut R) -> Result<Option<Self>, GraphError>
    where
        Self: std::marker::Sized,
    {
        let name = read_key_value_line(reader, KEY_NAME_STR)?;
        let data = match read_key_value_line_opt(reader, KEY_DISPLAY_NAME_STR)? {
            None => None,
            Some(display_name_str) => {
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
                let response_type = FuncSpecBackendResponseType::from_str(&response_type_str)
                    .map_err(GraphError::parse)?;
                let hidden = bool::from_str(&read_key_value_line(reader, KEY_HIDDEN_STR)?)
                    .map_err(GraphError::parse)?;
                let link_str = read_key_value_line(reader, KEY_LINK_STR)?;
                let link = if link_str.is_empty() {
                    None
                } else {
                    Some(Url::parse(&link_str).map_err(GraphError::parse)?)
                };

                let is_transformation_str = read_key_value_line_opt(reader, KEY_IS_TRANSFORMATION)?;

                let is_transformation = if let Some(str) = is_transformation_str {
                    bool::from_str(&str).map_err(GraphError::parse)?
                } else {
                    false
                };

                let updated_str = read_key_value_line_opt(reader, KEY_LAST_UPDATED)?;

                let last_updated_at = updated_str
                    .map(|str| DateTime::from_str(&str).map_err(GraphError::parse))
                    .transpose()?;

                Some(FuncData {
                    name: name.clone(),
                    display_name,
                    description,
                    handler,
                    code_base64,
                    backend_kind,
                    response_type,
                    hidden,
                    link,
                    is_transformation,
                    last_updated_at,
                })
            }
        };

        let (unique_id, deleted) = read_common_fields(reader)?;
        let is_from_builtin_str = read_key_value_line_opt(reader, KEY_IS_FROM_BUILTIN)?;
        let is_from_builtin = if let Some(is_from_builtin) = is_from_builtin_str {
            Some(bool::from_str(&is_from_builtin).map_err(GraphError::parse)?)
        } else {
            None
        };

        Ok(Some(Self {
            name,
            data,
            unique_id: unique_id.unwrap_or("".into()),
            deleted,
            is_from_builtin,
        }))
    }
}

impl NodeChild for FuncSpec {
    type NodeType = PkgNode;

    fn as_node_with_children(&self) -> NodeWithChildren<Self::NodeType> {
        let children = self
            .arguments
            .iter()
            .map(|arg| Box::new(arg.clone()) as Box<dyn NodeChild<NodeType = Self::NodeType>>)
            .collect();

        NodeWithChildren::new(
            NodeKind::Tree,
            Self::NodeType::Func(FuncNode {
                name: self.name.to_owned(),
                data: self.data.as_ref().map(|data| FuncData {
                    name: self.name.to_owned(),
                    display_name: data.display_name.as_ref().cloned(),
                    description: data.description.as_ref().cloned(),
                    handler: data.handler.to_string(),
                    code_base64: data.code_base64.to_string(),
                    backend_kind: data.backend_kind,
                    response_type: data.response_type,
                    hidden: data.hidden,
                    link: data.link.as_ref().cloned(),
                    is_transformation: data.is_transformation,
                    last_updated_at: data.last_updated_at,
                }),
                unique_id: self.unique_id.to_owned(),
                deleted: self.deleted,
                is_from_builtin: self.is_from_builtin.to_owned(),
            }),
            children,
        )
    }
}
