use derive_builder::UninitializedFieldError;
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, Display, EnumIter, EnumString};
use url::Url;

use super::{AttrFuncInputSpec, MapKeyFuncSpec, SpecError};

#[remain::sorted]
#[derive(
    Debug,
    Serialize,
    Deserialize,
    Clone,
    PartialEq,
    Eq,
    AsRefStr,
    Display,
    EnumIter,
    EnumString,
    Copy,
    Default,
)]
pub enum PropSpecWidgetKind {
    Array,
    Checkbox,
    CodeEditor,
    Color,
    ComboBox,
    Header,
    Map,
    Password,
    Secret,
    Select,
    #[default]
    Text,
    TextArea,
}

impl From<&PropSpec> for PropSpecWidgetKind {
    fn from(node: &PropSpec) -> Self {
        match node {
            PropSpec::Array { .. } => Self::Array,
            PropSpec::Boolean { .. } => Self::Checkbox,
            PropSpec::String { .. } | PropSpec::Number { .. } => Self::Text,
            PropSpec::Object { .. } => Self::Header,
            PropSpec::Map { .. } => Self::Map,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PropSpecData {
    pub name: String,
    pub validation_format: Option<String>,
    pub default_value: Option<serde_json::Value>,
    pub func_unique_id: Option<String>,
    pub inputs: Option<Vec<AttrFuncInputSpec>>,
    pub widget_kind: Option<PropSpecWidgetKind>,
    pub widget_options: Option<serde_json::Value>,
    pub hidden: Option<bool>,
    pub doc_link: Option<Url>,
    pub documentation: Option<String>,
}

#[remain::sorted]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum PropSpec {
    #[serde(rename_all = "camelCase")]
    Array {
        name: String,
        data: Option<PropSpecData>,
        unique_id: Option<String>,
        type_prop: Box<PropSpec>,
    },
    #[serde(rename_all = "camelCase")]
    Boolean {
        name: String,
        data: Option<PropSpecData>,
        unique_id: Option<String>,
    },
    #[serde(rename_all = "camelCase")]
    Map {
        name: String,
        data: Option<PropSpecData>,
        unique_id: Option<String>,
        type_prop: Box<PropSpec>,
        map_key_funcs: Option<Vec<MapKeyFuncSpec>>,
    },
    #[serde(rename_all = "camelCase")]
    Number {
        name: String,
        data: Option<PropSpecData>,
        unique_id: Option<String>,
    },
    #[serde(rename_all = "camelCase")]
    Object {
        name: String,
        data: Option<PropSpecData>,
        unique_id: Option<String>,
        entries: Vec<PropSpec>,
    },
    #[serde(rename_all = "camelCase")]
    String {
        name: String,
        data: Option<PropSpecData>,
        unique_id: Option<String>,
    },
}

impl PropSpec {
    pub fn builder() -> PropSpecBuilder {
        PropSpecBuilder::default()
    }
}

#[remain::sorted]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum PropSpecKind {
    Array,
    Boolean,
    Map,
    Number,
    Object,
    String,
}

#[derive(Clone, Debug)]
pub struct PropSpecBuilder {
    default_value: Option<serde_json::Value>,
    doc_link: Option<Url>,
    documentation: Option<String>,
    entries: Vec<PropSpec>,
    func_unique_id: Option<String>,
    hidden: bool,
    inputs: Vec<AttrFuncInputSpec>,
    kind: Option<PropSpecKind>,
    map_key_funcs: Vec<MapKeyFuncSpec>,
    name: Option<String>,
    type_prop: Option<PropSpec>,
    validation_format: Option<String>,
    widget_kind: Option<PropSpecWidgetKind>,
    widget_options: Option<serde_json::Value>,
    unique_id: Option<String>,
    has_data: bool,
}

impl Default for PropSpecBuilder {
    fn default() -> Self {
        Self {
            default_value: None,
            doc_link: None,
            documentation: None,
            entries: vec![],
            func_unique_id: None,
            hidden: false,
            inputs: vec![],
            kind: None,
            map_key_funcs: vec![],
            name: None,
            type_prop: None,
            validation_format: None,
            widget_kind: None,
            widget_options: None,
            unique_id: None,
            has_data: true,
        }
    }
}

impl PropSpecBuilder {
    #[allow(unused_mut)]
    pub fn default_value(&mut self, value: serde_json::Value) -> &mut Self {
        self.default_value = Some(value);
        self.has_data = true;
        self
    }

    #[allow(unused_mut)]
    pub fn kind(&mut self, value: impl Into<PropSpecKind>) -> &mut Self {
        self.kind = Some(value.into());
        self
    }

    pub fn get_kind(&self) -> Option<PropSpecKind> {
        self.kind
    }

    #[allow(unused_mut)]
    pub fn name(&mut self, value: impl Into<String>) -> &mut Self {
        self.name = Some(value.into());
        self
    }

    #[allow(unused_mut)]
    pub fn type_prop(&mut self, value: impl Into<PropSpec>) -> &mut Self {
        self.type_prop = Some(value.into());
        self
    }

    #[allow(unused_mut)]
    pub fn entry(&mut self, value: impl Into<PropSpec>) -> &mut Self {
        self.entries.push(value.into());
        self
    }

    #[allow(unused_mut)]
    pub fn validation_format(&mut self, value: impl Into<String>) -> &mut Self {
        self.has_data = true;
        self.validation_format = Some(value.into());
        self
    }

    #[allow(unused_mut)]
    pub fn entries(&mut self, value: Vec<impl Into<PropSpec>>) -> &mut Self {
        self.entries = value.into_iter().map(Into::into).collect();
        self
    }

    #[allow(unused_mut)]
    pub fn func_unique_id(&mut self, value: impl Into<String>) -> &mut Self {
        self.has_data = true;
        self.func_unique_id = Some(value.into());
        self
    }

    #[allow(unused_mut)]
    pub fn input(&mut self, value: impl Into<AttrFuncInputSpec>) -> &mut Self {
        self.has_data = true;
        self.inputs.push(value.into());
        self
    }

    pub fn widget_kind(&mut self, value: impl Into<PropSpecWidgetKind>) -> &mut Self {
        self.widget_kind = Some(value.into());
        self
    }

    pub fn widget_options(&mut self, value: impl Into<serde_json::Value>) -> &mut Self {
        self.widget_options = Some(value.into());
        self
    }

    pub fn hidden(&mut self, value: impl Into<bool>) -> &mut Self {
        self.hidden = value.into();
        self
    }

    pub fn doc_link(&mut self, value: impl Into<Url>) -> &mut Self {
        self.doc_link = Some(value.into());
        self
    }

    pub fn documentation(&mut self, value: impl Into<String>) -> &mut Self {
        self.documentation = Some(value.into());
        self
    }

    pub fn map_key_func(&mut self, value: impl Into<MapKeyFuncSpec>) -> &mut Self {
        self.has_data = true;
        self.map_key_funcs.push(value.into());
        self
    }

    pub fn has_data(&mut self, value: impl Into<bool>) -> &mut Self {
        self.has_data = value.into();
        self
    }

    pub fn unique_id(&mut self, value: impl Into<String>) -> &mut Self {
        self.unique_id = Some(value.into());
        self
    }

    #[allow(unused_mut)]
    pub fn try_doc_link<V>(&mut self, value: V) -> Result<&mut Self, V::Error>
    where
        V: TryInto<Url>,
    {
        let converted: Url = value.try_into()?;
        Ok(self.doc_link(converted))
    }

    /// Builds a new `Prop`.
    ///
    /// # Errors
    ///
    /// If a required field has not been initialized.
    pub fn build(&self) -> Result<PropSpec, SpecError> {
        let name = match self.name {
            Some(ref name) => name.clone(),
            None => {
                return Err(UninitializedFieldError::from("name").into());
            }
        };

        let maybe_data = if self.has_data {
            Some(PropSpecData {
                name: name.to_owned(),
                validation_format: self.validation_format.clone(),
                default_value: self.default_value.to_owned(),
                func_unique_id: self.func_unique_id.to_owned(),
                inputs: Some(self.inputs.clone()),
                widget_kind: self.widget_kind,
                widget_options: self.widget_options.to_owned(),
                hidden: Some(self.hidden),
                doc_link: self.doc_link.to_owned(),
                documentation: self.documentation.to_owned(),
            })
        } else {
            None
        };

        Ok(match self.kind {
            Some(kind) => match kind {
                PropSpecKind::String => PropSpec::String {
                    name: name.to_owned(),
                    unique_id: self.unique_id.to_owned(),
                    data: maybe_data,
                },
                PropSpecKind::Number => PropSpec::Number {
                    name: name.to_owned(),
                    unique_id: self.unique_id.to_owned(),
                    data: maybe_data,
                },
                PropSpecKind::Boolean => PropSpec::Boolean {
                    name: name.to_owned(),
                    unique_id: self.unique_id.to_owned(),
                    data: maybe_data,
                },
                PropSpecKind::Map => PropSpec::Map {
                    name: name.to_owned(),
                    unique_id: self.unique_id.to_owned(),
                    data: maybe_data,
                    type_prop: match self.type_prop {
                        Some(ref value) => Box::new(value.clone()),
                        None => {
                            return Err(UninitializedFieldError::from("type_prop").into());
                        }
                    },
                    map_key_funcs: Some(self.map_key_funcs.to_owned()),
                },
                PropSpecKind::Array => PropSpec::Array {
                    name: name.to_owned(),
                    unique_id: self.unique_id.to_owned(),
                    data: maybe_data,
                    type_prop: match self.type_prop {
                        Some(ref value) => Box::new(value.clone()),
                        None => {
                            return Err(UninitializedFieldError::from("type_prop").into());
                        }
                    },
                },
                PropSpecKind::Object => PropSpec::Object {
                    name: name.to_owned(),
                    unique_id: self.unique_id.to_owned(),
                    data: maybe_data,
                    entries: self.entries.clone(),
                },
            },
            None => {
                return Err(UninitializedFieldError::from("kind").into());
            }
        })
    }
}

impl TryFrom<PropSpecBuilder> for PropSpec {
    type Error = SpecError;

    fn try_from(value: PropSpecBuilder) -> Result<Self, Self::Error> {
        value.build()
    }
}
