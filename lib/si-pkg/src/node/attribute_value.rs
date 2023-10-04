use std::{
    io::{BufRead, Write},
    str::FromStr,
};

use object_tree::{
    read_key_value_line, write_key_value_line, GraphError, NodeChild, NodeKind, NodeWithChildren,
    ReadBytes, WriteBytes,
};

use super::{attribute_value_child::AttributeValueChild, PkgNode};
use crate::spec::{
    AttributeValuePath, AttributeValueSpec, FuncSpecBackendKind, FuncSpecBackendResponseType,
};

const KEY_BACKEND_KIND_STR: &str = "backend_kind";
const KEY_RESPONSE_TYPE_STR: &str = "response_type";
const KEY_CODE_STR: &str = "code";
const KEY_FUNC_BINDING_ARGS_STR: &str = "func_binding_args";
const KEY_FUNC_UNIQUE_ID_STR: &str = "func_unique_id";
const KEY_HANDLER_STR: &str = "handler";
const KEY_KEY_STR: &str = "key";
const KEY_OUTPUT_STREAM_STR: &str = "output_stream";
const KEY_PARENT_PATH_STR: &str = "parent_path";
const KEY_PATH_STR: &str = "path";
const KEY_SEALED_PROXY_STR: &str = "sealed_proxy";
const KEY_UNPROCESSED_VALUE_STR: &str = "unprocessed_value";
const KEY_VALUE_STR: &str = "value";
const KEY_COMPONENT_SPECIFIC_STR: &str = "component_specific";

#[derive(Clone, Debug)]
pub struct AttributeValueNode {
    pub backend_kind: FuncSpecBackendKind,
    pub code_base64: Option<String>,
    pub func_binding_args: serde_json::Value,
    pub func_unique_id: String,
    pub handler: Option<String>,
    pub key: Option<String>,
    pub output_stream: Option<serde_json::Value>,
    pub parent_path: Option<AttributeValuePath>,
    pub path: AttributeValuePath,
    pub response_type: FuncSpecBackendResponseType,
    pub sealed_proxy: bool,
    pub component_specific: bool,
    pub unprocessed_value: Option<serde_json::Value>,
    pub value: Option<serde_json::Value>,
}

impl WriteBytes for AttributeValueNode {
    fn write_bytes<W: Write>(&self, writer: &mut W) -> Result<(), GraphError> {
        write_key_value_line(writer, KEY_BACKEND_KIND_STR, self.backend_kind)?;

        write_key_value_line(
            writer,
            KEY_CODE_STR,
            self.code_base64.as_deref().unwrap_or(""),
        )?;

        write_key_value_line(
            writer,
            KEY_FUNC_BINDING_ARGS_STR,
            serde_json::to_string(&self.func_binding_args).map_err(GraphError::parse)?,
        )?;

        write_key_value_line(writer, KEY_FUNC_UNIQUE_ID_STR, &self.func_unique_id)?;
        write_key_value_line(
            writer,
            KEY_HANDLER_STR,
            self.handler.as_deref().unwrap_or(""),
        )?;
        write_key_value_line(writer, KEY_KEY_STR, self.key.as_deref().unwrap_or(""))?;

        let output_stream = if let Some(output_stream_value) = &self.output_stream {
            serde_json::to_string(output_stream_value).map_err(GraphError::parse)?
        } else {
            "".into()
        };
        write_key_value_line(writer, KEY_OUTPUT_STREAM_STR, output_stream)?;

        let parent_path_str = if let Some(parent_path) = &self.parent_path {
            serde_json::to_string(parent_path).map_err(GraphError::parse)?
        } else {
            "".into()
        };
        write_key_value_line(writer, KEY_PARENT_PATH_STR, parent_path_str)?;
        write_key_value_line(
            writer,
            KEY_PATH_STR,
            serde_json::to_string(&self.path).map_err(GraphError::parse)?,
        )?;
        write_key_value_line(writer, KEY_RESPONSE_TYPE_STR, self.response_type)?;
        write_key_value_line(writer, KEY_SEALED_PROXY_STR, self.sealed_proxy)?;
        write_key_value_line(writer, KEY_COMPONENT_SPECIFIC_STR, self.component_specific)?;

        let unprocessed_value_str = if let Some(unprocessed_value) = &self.unprocessed_value {
            serde_json::to_string(unprocessed_value).map_err(GraphError::parse)?
        } else {
            "".into()
        };
        write_key_value_line(writer, KEY_UNPROCESSED_VALUE_STR, unprocessed_value_str)?;

        let value_str = if let Some(value) = &self.value {
            serde_json::to_string(value).map_err(GraphError::parse)?
        } else {
            "".into()
        };
        write_key_value_line(writer, KEY_VALUE_STR, value_str)?;

        Ok(())
    }
}

impl ReadBytes for AttributeValueNode {
    fn read_bytes<R: BufRead>(reader: &mut R) -> Result<Option<Self>, GraphError>
    where
        Self: std::marker::Sized,
    {
        let backend_kind_str = read_key_value_line(reader, KEY_BACKEND_KIND_STR)?;
        let backend_kind =
            FuncSpecBackendKind::from_str(&backend_kind_str).map_err(GraphError::parse)?;

        let code_base64_str = read_key_value_line(reader, KEY_CODE_STR)?;
        let code_base64 = if code_base64_str.is_empty() {
            None
        } else {
            Some(code_base64_str)
        };

        let func_binding_args_str = read_key_value_line(reader, KEY_FUNC_BINDING_ARGS_STR)?;
        let func_binding_args: serde_json::Value =
            serde_json::from_str(&func_binding_args_str).map_err(GraphError::parse)?;

        let func_unique_id = read_key_value_line(reader, KEY_FUNC_UNIQUE_ID_STR)?;

        let handler_str = read_key_value_line(reader, KEY_HANDLER_STR)?;
        let handler = if handler_str.is_empty() {
            None
        } else {
            Some(handler_str)
        };

        let key_str = read_key_value_line(reader, KEY_KEY_STR)?;
        let key = if key_str.is_empty() {
            None
        } else {
            Some(key_str)
        };

        let output_stream_str = read_key_value_line(reader, KEY_OUTPUT_STREAM_STR)?;
        let output_stream = if output_stream_str.is_empty() {
            None
        } else {
            Some(serde_json::from_str(&output_stream_str).map_err(GraphError::parse)?)
        };

        let parent_path_str = read_key_value_line(reader, KEY_PARENT_PATH_STR)?;
        let parent_path = if parent_path_str.is_empty() {
            None
        } else {
            Some(serde_json::from_str(&parent_path_str).map_err(GraphError::parse)?)
        };

        let path_str = read_key_value_line(reader, KEY_PATH_STR)?;
        let path = serde_json::from_str(&path_str).map_err(GraphError::parse)?;

        let response_type_str = read_key_value_line(reader, KEY_RESPONSE_TYPE_STR)?;
        let response_type =
            FuncSpecBackendResponseType::from_str(&response_type_str).map_err(GraphError::parse)?;

        let sealed_proxy = bool::from_str(&read_key_value_line(reader, KEY_SEALED_PROXY_STR)?)
            .map_err(GraphError::parse)?;
        let component_specific =
            bool::from_str(&read_key_value_line(reader, KEY_COMPONENT_SPECIFIC_STR)?)
                .map_err(GraphError::parse)?;

        let unprocessed_value_str = read_key_value_line(reader, KEY_UNPROCESSED_VALUE_STR)?;
        let unprocessed_value = if unprocessed_value_str.is_empty() {
            None
        } else {
            Some(serde_json::from_str(&unprocessed_value_str).map_err(GraphError::parse)?)
        };

        let value_str = read_key_value_line(reader, KEY_VALUE_STR)?;
        let value = if value_str.is_empty() {
            None
        } else {
            Some(serde_json::from_str(&value_str).map_err(GraphError::parse)?)
        };

        Ok(Some(Self {
            backend_kind,
            code_base64,
            func_binding_args,
            func_unique_id,
            handler,
            key,
            output_stream,
            parent_path,
            path,
            response_type,
            sealed_proxy,
            component_specific,
            unprocessed_value,
            value,
        }))
    }
}

impl NodeChild for AttributeValueSpec {
    type NodeType = PkgNode;

    fn as_node_with_children(&self) -> NodeWithChildren<Self::NodeType> {
        NodeWithChildren::new(
            NodeKind::Tree,
            Self::NodeType::AttributeValue(AttributeValueNode {
                backend_kind: self.backend_kind,
                code_base64: self.code_base64.as_ref().cloned(),
                func_binding_args: self.func_binding_args.to_owned(),
                func_unique_id: self.func_unique_id.to_owned(),
                handler: self.handler.to_owned(),
                key: self.key.to_owned(),
                output_stream: self.output_stream.as_ref().cloned(),
                parent_path: self.parent_path.as_ref().cloned(),
                path: self.path.to_owned(),
                response_type: self.reponse_type,
                sealed_proxy: self.sealed_proxy,
                component_specific: self.component_specific,
                unprocessed_value: self.unprocessed_value.as_ref().cloned(),
                value: self.value.as_ref().cloned(),
            }),
            vec![
                Box::new(AttributeValueChild::AttrFuncInputs(self.inputs.to_owned()))
                    as Box<dyn NodeChild<NodeType = Self::NodeType>>,
            ],
        )
    }
}
