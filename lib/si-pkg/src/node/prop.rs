use std::{
    collections::HashMap,
    io::{
        BufRead,
        Write,
    },
    str::FromStr,
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
    prop_child::PropChild,
};
use crate::{
    PropSpec,
    PropSpecWidgetKind,
    spec::PropSpecData,
};

const KEY_KIND_STR: &str = "kind";
const KEY_NAME_STR: &str = "name";
const KEY_FUNC_UNIQUE_ID_STR: &str = "func_unique_id";
const KEY_DEFAULT_VALUE_STR: &str = "default_value";
const KEY_WIDGET_KIND_STR: &str = "widget_kind";
const KEY_WIDGET_OPTIONS_STR: &str = "widget_options";
const KEY_HIDDEN_STR: &str = "hidden";
const KEY_DOC_LINK_STR: &str = "doc_link";
const KEY_DOCUMENTATION_STR: &str = "documentation";
const KEY_VALIDATION_FORMAT_STR: &str = "validation_format";
const KEY_UI_OPTIONALS_STR: &str = "ui_optionals";
const KEY_UNIQUE_ID_STR: &str = "unique_id";
const KEY_CHILD_ORDER_STR: &str = "child_order";

const PROP_TY_STRING: &str = "string";
const PROP_TY_JSON: &str = "json";
const PROP_TY_INTEGER: &str = "integer";
const PROP_TY_FLOAT: &str = "float";
const PROP_TY_BOOLEAN: &str = "boolean";
const PROP_TY_MAP: &str = "map";
const PROP_TY_ARRAY: &str = "array";
const PROP_TY_OBJECT: &str = "object";

#[derive(Clone, Debug)]
pub struct PropNodeData {
    pub name: String,
    pub func_unique_id: Option<String>,
    pub default_value: Option<serde_json::Value>,
    pub widget_kind: PropSpecWidgetKind,
    pub widget_options: Option<serde_json::Value>,
    pub doc_link: Option<Url>,
    pub hidden: bool,
    pub documentation: Option<String>,
    pub validation_format: Option<String>,
    pub ui_optionals: HashMap<String, serde_json::Value>,
}

#[remain::sorted]
#[derive(Clone, Debug)]
pub enum PropNode {
    Array {
        name: String,
        data: Option<PropNodeData>,
        unique_id: Option<String>,
    },
    Boolean {
        name: String,
        data: Option<PropNodeData>,
        unique_id: Option<String>,
    },
    Float {
        name: String,
        data: Option<PropNodeData>,
        unique_id: Option<String>,
    },
    Integer {
        name: String,
        data: Option<PropNodeData>,
        unique_id: Option<String>,
    },
    Json {
        name: String,
        data: Option<PropNodeData>,
        unique_id: Option<String>,
    },
    Map {
        name: String,
        data: Option<PropNodeData>,
        unique_id: Option<String>,
    },
    Object {
        name: String,
        data: Option<PropNodeData>,
        unique_id: Option<String>,
        // The names of the child props in order
        child_order: Option<Vec<String>>,
    },
    String {
        name: String,
        data: Option<PropNodeData>,
        unique_id: Option<String>,
    },
}

impl PropNode {
    fn kind_str(&self) -> &'static str {
        match self {
            Self::String { .. } => PROP_TY_STRING,
            Self::Json { .. } => PROP_TY_JSON,
            Self::Float { .. } => PROP_TY_FLOAT,
            Self::Integer { .. } => PROP_TY_INTEGER,
            Self::Boolean { .. } => PROP_TY_BOOLEAN,
            Self::Map { .. } => PROP_TY_MAP,
            Self::Array { .. } => PROP_TY_ARRAY,
            Self::Object { .. } => PROP_TY_OBJECT,
        }
    }

    pub fn child_order(&self) -> Option<&Vec<String>> {
        match self {
            Self::Object { child_order, .. } => child_order.as_ref(),
            _ => None,
        }
    }
}

impl NameStr for PropNode {
    fn name(&self) -> &str {
        match self {
            Self::String { name, .. }
            | Self::Json { name, .. }
            | Self::Float { name, .. }
            | Self::Integer { name, .. }
            | Self::Boolean { name, .. }
            | Self::Map { name, .. }
            | Self::Array { name, .. }
            | Self::Object { name, .. } => name,
        }
    }
}

impl WriteBytes for PropNode {
    fn write_bytes<W: Write>(&self, writer: &mut W) -> Result<(), GraphError> {
        let (Self::String {
            name,
            data,
            unique_id,
            ..
        }
        | Self::Json {
            name,
            data,
            unique_id,
            ..
        }
        | Self::Float {
            name,
            data,
            unique_id,
            ..
        }
        | Self::Integer {
            name,
            data,
            unique_id,
            ..
        }
        | Self::Boolean {
            name,
            data,
            unique_id,
            ..
        }
        | Self::Map {
            name,
            data,
            unique_id,
            ..
        }
        | Self::Array {
            name,
            data,
            unique_id,
            ..
        }
        | Self::Object {
            name,
            data,
            unique_id,
            ..
        }) = self;

        write_key_value_line(writer, KEY_KIND_STR, self.kind_str())?;
        write_key_value_line(writer, KEY_NAME_STR, name)?;

        if let Some(PropNodeData {
            name: _, // this is duplicate data from the top-level name
            func_unique_id,
            default_value,
            widget_kind,
            widget_options,
            doc_link,
            hidden,
            documentation,
            validation_format,
            ui_optionals,
        }) = data
        {
            write_key_value_line(
                writer,
                KEY_FUNC_UNIQUE_ID_STR,
                func_unique_id
                    .as_ref()
                    .map(|fuid| fuid.to_owned())
                    .unwrap_or("".to_string()),
            )?;

            write_key_value_line(
                writer,
                KEY_DEFAULT_VALUE_STR,
                match default_value {
                    Some(dv) => serde_json::to_string(dv).map_err(GraphError::parse)?,
                    None => "".to_string(),
                },
            )?;

            write_key_value_line(writer, KEY_WIDGET_KIND_STR, widget_kind)?;

            write_key_value_line(
                writer,
                KEY_WIDGET_OPTIONS_STR,
                match &widget_options {
                    Some(options) => serde_json::to_string(options).map_err(GraphError::parse)?,
                    None => "".to_string(),
                },
            )?;

            write_key_value_line(writer, KEY_HIDDEN_STR, hidden)?;

            write_key_value_line(
                writer,
                KEY_DOC_LINK_STR,
                doc_link.as_ref().map(|l| l.as_str()).unwrap_or(""),
            )?;

            write_key_value_line_opt(writer, KEY_DOCUMENTATION_STR, documentation.as_ref())?;
            write_key_value_line_opt(
                writer,
                KEY_VALIDATION_FORMAT_STR,
                validation_format.as_ref(),
            )?;

            // If it's a non-empty object, write it out
            if !ui_optionals.is_empty() {
                write_key_value_line(
                    writer,
                    KEY_UI_OPTIONALS_STR,
                    serde_json::to_string(&ui_optionals).map_err(GraphError::parse)?,
                )?;
            }
        }

        if let Some(unique_id) = unique_id.as_deref() {
            write_key_value_line(writer, KEY_UNIQUE_ID_STR, unique_id)?;
        }

        if let Self::Object { child_order, .. } = self {
            write_key_value_line_opt(
                writer,
                KEY_CHILD_ORDER_STR,
                child_order
                    .as_ref()
                    .map(serde_json::to_string)
                    .transpose()
                    .map_err(GraphError::parse)?,
            )?;
        }

        Ok(())
    }
}

impl ReadBytes for PropNode {
    fn read_bytes<R: BufRead>(reader: &mut R) -> Result<Option<Self>, GraphError>
    where
        Self: std::marker::Sized,
    {
        let kind_str = read_key_value_line(reader, KEY_KIND_STR)?;
        let name = read_key_value_line(reader, KEY_NAME_STR)?;

        let data = match read_key_value_line_opt(reader, KEY_FUNC_UNIQUE_ID_STR)? {
            None => None,
            Some(func_unique_id_str) => {
                let func_unique_id = if func_unique_id_str.is_empty() {
                    None
                } else {
                    Some(func_unique_id_str)
                };

                let default_value_str = read_key_value_line(reader, KEY_DEFAULT_VALUE_STR)?;
                let default_value: Option<serde_json::Value> = if default_value_str.is_empty() {
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

                let documentation = read_key_value_line_opt(reader, KEY_DOCUMENTATION_STR)?;
                let validation_format = read_key_value_line_opt(reader, KEY_VALIDATION_FORMAT_STR)?;

                let ui_optionals = read_key_value_line_opt(reader, KEY_UI_OPTIONALS_STR)?
                    .map(|ui_optionals_str| serde_json::from_str(&ui_optionals_str))
                    .transpose()
                    .map_err(GraphError::parse)?
                    .unwrap_or_default();

                Some(PropNodeData {
                    name: name.to_owned(),
                    func_unique_id,
                    default_value,
                    widget_kind,
                    widget_options,
                    doc_link,
                    hidden,
                    documentation,
                    validation_format,
                    ui_optionals,
                })
            }
        };

        let unique_id = read_key_value_line_opt(reader, KEY_UNIQUE_ID_STR)?;

        let node = match kind_str.as_str() {
            PROP_TY_STRING => Self::String {
                name,
                data,
                unique_id,
            },
            PROP_TY_JSON => Self::Json {
                name,
                data,
                unique_id,
            },
            PROP_TY_INTEGER => Self::Integer {
                name,
                data,
                unique_id,
            },
            PROP_TY_FLOAT => Self::Float {
                name,
                data,
                unique_id,
            },
            PROP_TY_BOOLEAN => Self::Boolean {
                name,
                data,
                unique_id,
            },
            PROP_TY_MAP => Self::Map {
                name,
                data,
                unique_id,
            },
            PROP_TY_ARRAY => Self::Array {
                name,
                data,
                unique_id,
            },
            PROP_TY_OBJECT => {
                let child_order = read_key_value_line_opt(reader, "child_order")?
                    .map(|child_order_str| serde_json::from_str(&child_order_str))
                    .transpose()
                    .map_err(GraphError::parse)?;
                Self::Object {
                    name,
                    data,
                    unique_id,
                    child_order,
                }
            }
            invalid_kind => {
                return Err(GraphError::parse_custom(format!(
                    "invalid prop node kind: {invalid_kind}"
                )));
            }
        };

        Ok(Some(node))
    }
}

impl NodeChild for PropSpec {
    type NodeType = PkgNode;

    fn as_node_with_children(&self) -> NodeWithChildren<Self::NodeType> {
        let (name, data, unique_id, inputs) = match &self {
            Self::Array {
                name,
                data,
                unique_id,
                ..
            }
            | Self::Json {
                name,
                data,
                unique_id,
            }
            | Self::Boolean {
                name,
                data,
                unique_id,
            }
            | Self::Map {
                name,
                data,
                unique_id,
                ..
            }
            | Self::Number {
                name,
                data,
                unique_id,
            }
            | Self::Float {
                name,
                data,
                unique_id,
            }
            | Self::Object {
                name,
                data,
                unique_id,
                ..
            }
            | Self::String {
                name,
                data,
                unique_id,
            } => (
                name.to_owned(),
                data.to_owned().map(
                    |PropSpecData {
                         name,
                         default_value,
                         func_unique_id,
                         widget_kind,
                         widget_options,
                         hidden,
                         doc_link,
                         documentation,
                         validation_format,
                         ui_optionals,
                         inputs: _,
                     }| PropNodeData {
                        name,
                        default_value,
                        func_unique_id,
                        widget_kind: widget_kind.unwrap_or(PropSpecWidgetKind::from(self)),
                        widget_options,
                        hidden: hidden.unwrap_or(false),
                        doc_link,
                        documentation,
                        validation_format,
                        ui_optionals,
                    },
                ),
                unique_id.to_owned(),
                data.as_ref().and_then(|data| data.inputs.to_owned()),
            ),
        };

        match self {
            Self::String { .. } => NodeWithChildren::new(
                NodeKind::Tree,
                Self::NodeType::Prop(PropNode::String {
                    name,
                    data,
                    unique_id,
                }),
                vec![Box::new(PropChild::AttrFuncInputs(
                    inputs.to_owned().unwrap_or(vec![]),
                ))
                    as Box<dyn NodeChild<NodeType = Self::NodeType>>],
            ),
            Self::Json { .. } => NodeWithChildren::new(
                NodeKind::Tree,
                Self::NodeType::Prop(PropNode::Json {
                    name,
                    data,
                    unique_id,
                }),
                vec![Box::new(PropChild::AttrFuncInputs(
                    inputs.to_owned().unwrap_or(vec![]),
                ))
                    as Box<dyn NodeChild<NodeType = Self::NodeType>>],
            ),
            Self::Number { .. } => NodeWithChildren::new(
                NodeKind::Tree,
                Self::NodeType::Prop(PropNode::Integer {
                    name,
                    data,
                    unique_id,
                }),
                vec![Box::new(PropChild::AttrFuncInputs(
                    inputs.to_owned().unwrap_or(vec![]),
                ))
                    as Box<dyn NodeChild<NodeType = Self::NodeType>>],
            ),
            Self::Float { .. } => NodeWithChildren::new(
                NodeKind::Tree,
                Self::NodeType::Prop(PropNode::Float {
                    name,
                    data,
                    unique_id,
                }),
                vec![Box::new(PropChild::AttrFuncInputs(
                    inputs.to_owned().unwrap_or(vec![]),
                ))
                    as Box<dyn NodeChild<NodeType = Self::NodeType>>],
            ),
            Self::Boolean { .. } => NodeWithChildren::new(
                NodeKind::Tree,
                Self::NodeType::Prop(PropNode::Boolean {
                    name,
                    data,
                    unique_id,
                }),
                vec![Box::new(PropChild::AttrFuncInputs(
                    inputs.to_owned().unwrap_or(vec![]),
                ))
                    as Box<dyn NodeChild<NodeType = Self::NodeType>>],
            ),
            Self::Map {
                type_prop,
                map_key_funcs,
                ..
            } => NodeWithChildren::new(
                NodeKind::Tree,
                Self::NodeType::Prop(PropNode::Map {
                    name,
                    data,
                    unique_id,
                }),
                vec![
                    Box::new(PropChild::MapKeyFuncs(
                        map_key_funcs.to_owned().unwrap_or(vec![]),
                    )) as Box<dyn NodeChild<NodeType = Self::NodeType>>,
                    Box::new(PropChild::Props(vec![*type_prop.clone()]))
                        as Box<dyn NodeChild<NodeType = Self::NodeType>>,
                    Box::new(PropChild::AttrFuncInputs(
                        inputs.to_owned().unwrap_or(vec![]),
                    )) as Box<dyn NodeChild<NodeType = Self::NodeType>>,
                ],
            ),
            Self::Array { type_prop, .. } => NodeWithChildren::new(
                NodeKind::Tree,
                Self::NodeType::Prop(PropNode::Array {
                    name,
                    data,
                    unique_id,
                }),
                vec![
                    Box::new(PropChild::Props(vec![*type_prop.clone()]))
                        as Box<dyn NodeChild<NodeType = Self::NodeType>>,
                    Box::new(PropChild::AttrFuncInputs(
                        inputs.to_owned().unwrap_or(vec![]),
                    )) as Box<dyn NodeChild<NodeType = Self::NodeType>>,
                ],
            ),
            Self::Object { entries, .. } => NodeWithChildren::new(
                NodeKind::Tree,
                Self::NodeType::Prop(PropNode::Object {
                    name,
                    data,
                    unique_id,
                    child_order: Some(
                        entries
                            .iter()
                            .map(|entry| entry.name().to_string())
                            .collect(),
                    ),
                }),
                vec![
                    Box::new(PropChild::Props(entries.clone()))
                        as Box<dyn NodeChild<NodeType = Self::NodeType>>,
                    Box::new(PropChild::AttrFuncInputs(
                        inputs.to_owned().unwrap_or(vec![]),
                    )) as Box<dyn NodeChild<NodeType = Self::NodeType>>,
                ],
            ),
        }
    }
}
