use object_tree::{
    read_key_value_line, write_key_value_line, GraphError, NameStr, NodeChild, NodeKind,
    NodeWithChildren, ReadBytes, WriteBytes,
};
use std::{
    io::{BufRead, Write},
    str::FromStr,
};
use url::Url;

use crate::{FuncUniqueId, PropSpec, PropSpecSharedInfo, PropSpecWidgetKind};

use super::{prop_child::PropChild, PkgNode};

const KEY_KIND_STR: &str = "kind";
const KEY_NAME_STR: &str = "name";
const KEY_FUNC_UNIQUE_ID_STR: &str = "func_unique_id";
const KEY_DEFAULT_VALUE_STR: &str = "default_value";
const KEY_WIDGET_KIND_STR: &str = "widget_kind";
const KEY_WIDGET_OPTIONS_STR: &str = "widget_options";
const KEY_HIDDEN_STR: &str = "hidden";
const KEY_DOC_LINK_STR: &str = "doc_link";

const PROP_TY_STRING: &str = "string";
const PROP_TY_INTEGER: &str = "integer";
const PROP_TY_BOOLEAN: &str = "boolean";
const PROP_TY_MAP: &str = "map";
const PROP_TY_ARRAY: &str = "array";
const PROP_TY_OBJECT: &str = "object";

#[derive(Clone, Debug)]
pub enum PropNode {
    String {
        info: PropSpecSharedInfo,
        default_value: Option<String>,
    },
    Integer {
        info: PropSpecSharedInfo,
        default_value: Option<i64>,
    },
    Boolean {
        info: PropSpecSharedInfo,
        default_value: Option<bool>,
    },
    Map {
        info: PropSpecSharedInfo,
        default_value: Option<serde_json::Value>,
    },
    Array {
        info: PropSpecSharedInfo,
        default_value: Option<serde_json::Value>,
    },
    Object {
        info: PropSpecSharedInfo,
        default_value: Option<serde_json::Value>,
    },
}

impl PropNode {
    fn kind_str(&self) -> &'static str {
        match self {
            Self::String { .. } => PROP_TY_STRING,
            Self::Integer { .. } => PROP_TY_INTEGER,
            Self::Boolean { .. } => PROP_TY_BOOLEAN,
            Self::Map { .. } => PROP_TY_MAP,
            Self::Array { .. } => PROP_TY_ARRAY,
            Self::Object { .. } => PROP_TY_OBJECT,
        }
    }

    fn info(&self) -> &PropSpecSharedInfo {
        match self {
            Self::String { info, .. }
            | Self::Integer { info, .. }
            | Self::Boolean { info, .. }
            | Self::Map { info, .. }
            | Self::Array { info, .. }
            | Self::Object { info, .. } => &info,
        }
    }
}

impl NameStr for PropNode {
    fn name(&self) -> &str {
        match self {
            Self::String { info, .. }
            | Self::Integer { info, .. }
            | Self::Boolean { info, .. }
            | Self::Map { info, .. }
            | Self::Array { info, .. }
            | Self::Object { info, .. } => &info.name,
        }
    }
}

impl WriteBytes for PropNode {
    fn write_bytes<W: Write>(&self, writer: &mut W) -> Result<(), GraphError> {
        write_key_value_line(writer, KEY_KIND_STR, self.kind_str())?;
        write_key_value_line(writer, KEY_NAME_STR, self.name())?;

        let func_unique_id = self.info().func_unique_id;
        write_key_value_line(
            writer,
            KEY_FUNC_UNIQUE_ID_STR,
            func_unique_id
                .map(|fuid| fuid.to_string())
                .unwrap_or("".to_string()),
        )?;

        write_key_value_line(
            writer,
            KEY_DEFAULT_VALUE_STR,
            match &self {
                Self::String { default_value, .. } => match default_value {
                    Some(dv) => serde_json::to_string(dv).map_err(GraphError::parse)?,
                    None => "".to_string(),
                },
                Self::Integer { default_value, .. } => match default_value {
                    Some(dv) => serde_json::to_string(dv).map_err(GraphError::parse)?,
                    None => "".to_string(),
                },
                Self::Boolean { default_value, .. } => match default_value {
                    Some(dv) => serde_json::to_string(dv).map_err(GraphError::parse)?,
                    None => "".to_string(),
                },
                Self::Map { default_value, .. }
                | Self::Array { default_value, .. }
                | Self::Object { default_value, .. } => match default_value {
                    Some(dv) => serde_json::to_string(dv).map_err(GraphError::parse)?,
                    None => "".to_string(),
                },
            },
        )?;

        write_key_value_line(
            writer,
            KEY_WIDGET_KIND_STR,
            match self.info().widget_kind {
                Some(widget_kind) => widget_kind.to_string(),
                None => "".to_string(),
            },
        )?;

        write_key_value_line(
            writer,
            KEY_WIDGET_OPTIONS_STR,
            match &self.info().widget_options {
                Some(options) => serde_json::to_string(options).map_err(GraphError::parse)?,
                None => "".to_string(),
            },
        )?;

        write_key_value_line(writer, KEY_HIDDEN_STR, self.info().hidden)?;

        write_key_value_line(
            writer,
            KEY_DOC_LINK_STR,
            self.info()
                .doc_link
                .as_ref()
                .map(|l| l.as_str())
                .unwrap_or(""),
        )?;

        Ok(())
    }
}

impl ReadBytes for PropNode {
    fn read_bytes<R: BufRead>(reader: &mut R) -> Result<Self, GraphError>
    where
        Self: std::marker::Sized,
    {
        let kind_str = read_key_value_line(reader, KEY_KIND_STR)?;
        let name = read_key_value_line(reader, KEY_NAME_STR)?;
        let func_unique_id_str = read_key_value_line(reader, KEY_FUNC_UNIQUE_ID_STR)?;
        let func_unique_id = if func_unique_id_str.is_empty() {
            None
        } else {
            Some(FuncUniqueId::from_str(&func_unique_id_str).map_err(GraphError::parse)?)
        };

        let default_value_str = read_key_value_line(reader, KEY_DEFAULT_VALUE_STR)?;
        let default_value_json: Option<serde_json::Value> = if default_value_str.is_empty() {
            None
        } else {
            Some(serde_json::from_str(&default_value_str).map_err(GraphError::parse)?)
        };

        let widget_kind_str = read_key_value_line(reader, KEY_WIDGET_KIND_STR)?;
        let widget_kind =
            PropSpecWidgetKind::from_str(&widget_kind_str).map_err(GraphError::parse)?;

        let widget_options_str = read_key_value_line(reader, KEY_WIDGET_OPTIONS_STR)?;
        let widget_options = if widget_options_str.is_empty() {
            None
        } else {
            serde_json::from_str(&widget_options_str).map_err(GraphError::parse)?
        };

        let hidden = bool::from_str(&read_key_value_line(reader, KEY_HIDDEN_STR)?)
            .map_err(GraphError::parse)?;

        let doc_link_str = read_key_value_line(reader, KEY_DOC_LINK_STR)?;
        let doc_link = if doc_link_str.is_empty() {
            None
        } else {
            Some(Url::parse(&doc_link_str).map_err(GraphError::parse)?)
        };

        let info = PropSpecSharedInfo {
            name,
            func_unique_id,
            widget_kind: Some(widget_kind),
            widget_options,
            hidden,
            doc_link,
        };

        let node = match kind_str.as_str() {
            PROP_TY_STRING => Self::String {
                default_value: match default_value_json {
                    None => None,
                    Some(value) => {
                        if value.is_string() {
                            value.as_str().map(|s| s.to_owned())
                        } else {
                            return Err(GraphError::parse_custom(
                                "String prop must get a string as a default value",
                            ));
                        }
                    }
                },
                info,
            },
            PROP_TY_INTEGER => Self::Integer {
                default_value: match default_value_json {
                    None => None,
                    Some(value) => {
                        if value.is_i64() {
                            value.as_i64()
                        } else {
                            return Err(GraphError::parse_custom(
                                "Integer prop must get an i64 as a default value",
                            ));
                        }
                    }
                },
                info,
            },
            PROP_TY_BOOLEAN => Self::Boolean {
                default_value: match default_value_json {
                    None => None,
                    Some(value) => {
                        if value.is_boolean() {
                            value.as_bool()
                        } else {
                            return Err(GraphError::parse_custom(
                                "Boolean prop must get a bool as a default value",
                            ));
                        }
                    }
                },
                info,
            },
            PROP_TY_MAP => Self::Map {
                default_value: default_value_json,
                info,
            },
            PROP_TY_ARRAY => Self::Array {
                default_value: default_value_json,
                info,
            },
            PROP_TY_OBJECT => Self::Object {
                default_value: default_value_json,
                info,
            },
            invalid_kind => {
                return Err(GraphError::parse_custom(format!(
                    "invalid prop node kind: {invalid_kind}"
                )))
            }
        };

        Ok(node)
    }
}

impl NodeChild for PropSpec {
    type NodeType = PkgNode;

    fn as_node_with_children(&self) -> NodeWithChildren<Self::NodeType> {
        match self {
            Self::String {
                default_value,
                validations,
                inputs,
                info,
            } => NodeWithChildren::new(
                NodeKind::Tree,
                Self::NodeType::Prop(PropNode::String {
                    default_value: default_value.to_owned(),
                    info: info.to_owned(),
                }),
                vec![
                    Box::new(PropChild::Validations(
                        validations.to_owned().unwrap_or(vec![]),
                    )) as Box<dyn NodeChild<NodeType = Self::NodeType>>,
                    Box::new(PropChild::AttrFuncInputs(
                        inputs.to_owned().unwrap_or(vec![]),
                    )) as Box<dyn NodeChild<NodeType = Self::NodeType>>,
                ],
            ),
            Self::Number {
                default_value,
                validations,
                inputs,
                info,
            } => NodeWithChildren::new(
                NodeKind::Tree,
                Self::NodeType::Prop(PropNode::Integer {
                    default_value: default_value.to_owned(),
                    info: info.to_owned(),
                }),
                vec![
                    Box::new(PropChild::Validations(
                        validations.to_owned().unwrap_or(vec![]),
                    )) as Box<dyn NodeChild<NodeType = Self::NodeType>>,
                    Box::new(PropChild::AttrFuncInputs(
                        inputs.to_owned().unwrap_or(vec![]),
                    )) as Box<dyn NodeChild<NodeType = Self::NodeType>>,
                ],
            ),
            Self::Boolean {
                default_value,
                validations,
                inputs,
                info,
            } => NodeWithChildren::new(
                NodeKind::Tree,
                Self::NodeType::Prop(PropNode::Boolean {
                    default_value: default_value.to_owned(),
                    info: info.to_owned(),
                }),
                vec![
                    Box::new(PropChild::Validations(
                        validations.to_owned().unwrap_or(vec![]),
                    )) as Box<dyn NodeChild<NodeType = Self::NodeType>>,
                    Box::new(PropChild::AttrFuncInputs(
                        inputs.to_owned().unwrap_or(vec![]),
                    )) as Box<dyn NodeChild<NodeType = Self::NodeType>>,
                ],
            ),
            Self::Map {
                default_value,
                type_prop,
                validations,
                inputs,
                info,
            } => NodeWithChildren::new(
                NodeKind::Tree,
                Self::NodeType::Prop(PropNode::Map {
                    default_value: default_value.to_owned(),
                    info: info.to_owned(),
                }),
                vec![
                    Box::new(PropChild::Props(vec![*type_prop.clone()]))
                        as Box<dyn NodeChild<NodeType = Self::NodeType>>,
                    Box::new(PropChild::Validations(
                        validations.to_owned().unwrap_or(vec![]),
                    )) as Box<dyn NodeChild<NodeType = Self::NodeType>>,
                    Box::new(PropChild::AttrFuncInputs(
                        inputs.to_owned().unwrap_or(vec![]),
                    )) as Box<dyn NodeChild<NodeType = Self::NodeType>>,
                ],
            ),
            Self::Array {
                default_value,
                type_prop,
                validations,
                inputs,
                info,
            } => NodeWithChildren::new(
                NodeKind::Tree,
                Self::NodeType::Prop(PropNode::Array {
                    default_value: default_value.to_owned(),
                    info: info.to_owned(),
                }),
                vec![
                    Box::new(PropChild::Props(vec![*type_prop.clone()]))
                        as Box<dyn NodeChild<NodeType = Self::NodeType>>,
                    Box::new(PropChild::Validations(
                        validations.to_owned().unwrap_or(vec![]),
                    )) as Box<dyn NodeChild<NodeType = Self::NodeType>>,
                    Box::new(PropChild::AttrFuncInputs(
                        inputs.to_owned().unwrap_or(vec![]),
                    )) as Box<dyn NodeChild<NodeType = Self::NodeType>>,
                ],
            ),
            Self::Object {
                default_value,
                entries,
                validations,
                inputs,
                info,
            } => NodeWithChildren::new(
                NodeKind::Tree,
                Self::NodeType::Prop(PropNode::Object {
                    default_value: default_value.to_owned(),
                    info: info.to_owned(),
                }),
                vec![
                    Box::new(PropChild::Props(entries.clone()))
                        as Box<dyn NodeChild<NodeType = Self::NodeType>>,
                    Box::new(PropChild::Validations(
                        validations.to_owned().unwrap_or(vec![]),
                    )) as Box<dyn NodeChild<NodeType = Self::NodeType>>,
                    Box::new(PropChild::AttrFuncInputs(
                        inputs.to_owned().unwrap_or(vec![]),
                    )) as Box<dyn NodeChild<NodeType = Self::NodeType>>,
                ],
            ),
        }
    }
}
