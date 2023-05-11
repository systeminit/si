use derive_builder::UninitializedFieldError;
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, Display, EnumIter, EnumString};
use url::Url;

use super::{AttrFuncInputSpec, FuncUniqueId, SpecError, ValidationSpec};

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
    Color,
    ComboBox,
    Header,
    Map,
    SecretSelect,
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

#[remain::sorted]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum PropSpec {
    #[serde(rename_all = "camelCase")]
    Array {
        name: String,
        default_value: Option<serde_json::Value>,
        type_prop: Box<PropSpec>,
        validations: Option<Vec<ValidationSpec>>,
        func_unique_id: Option<FuncUniqueId>,
        inputs: Option<Vec<AttrFuncInputSpec>>,
        widget_kind: Option<PropSpecWidgetKind>,
        widget_options: Option<serde_json::Value>,
        hidden: Option<bool>,
        doc_link: Option<Url>,
    },
    #[serde(rename_all = "camelCase")]
    Boolean {
        name: String,
        default_value: Option<bool>,
        validations: Option<Vec<ValidationSpec>>,
        func_unique_id: Option<FuncUniqueId>,
        inputs: Option<Vec<AttrFuncInputSpec>>,
        widget_kind: Option<PropSpecWidgetKind>,
        widget_options: Option<serde_json::Value>,
        hidden: Option<bool>,
        doc_link: Option<Url>,
    },
    #[serde(rename_all = "camelCase")]
    Map {
        name: String,
        default_value: Option<serde_json::Value>,
        type_prop: Box<PropSpec>,
        validations: Option<Vec<ValidationSpec>>,
        func_unique_id: Option<FuncUniqueId>,
        inputs: Option<Vec<AttrFuncInputSpec>>,
        widget_kind: Option<PropSpecWidgetKind>,
        widget_options: Option<serde_json::Value>,
        hidden: Option<bool>,
        doc_link: Option<Url>,
    },
    #[serde(rename_all = "camelCase")]
    Number {
        name: String,
        default_value: Option<i64>,
        validations: Option<Vec<ValidationSpec>>,
        func_unique_id: Option<FuncUniqueId>,
        inputs: Option<Vec<AttrFuncInputSpec>>,
        widget_kind: Option<PropSpecWidgetKind>,
        widget_options: Option<serde_json::Value>,
        hidden: Option<bool>,
        doc_link: Option<Url>,
    },
    #[serde(rename_all = "camelCase")]
    Object {
        name: String,
        default_value: Option<serde_json::Value>,
        entries: Vec<PropSpec>,
        validations: Option<Vec<ValidationSpec>>,
        func_unique_id: Option<FuncUniqueId>,
        inputs: Option<Vec<AttrFuncInputSpec>>,
        widget_kind: Option<PropSpecWidgetKind>,
        widget_options: Option<serde_json::Value>,
        hidden: Option<bool>,
        doc_link: Option<Url>,
    },
    #[serde(rename_all = "camelCase")]
    String {
        name: String,
        default_value: Option<String>,
        validations: Option<Vec<ValidationSpec>>,
        func_unique_id: Option<FuncUniqueId>,
        inputs: Option<Vec<AttrFuncInputSpec>>,
        widget_kind: Option<PropSpecWidgetKind>,
        widget_options: Option<serde_json::Value>,
        hidden: Option<bool>,
        doc_link: Option<Url>,
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

#[derive(Clone, Debug, Default)]
pub struct PropSpecBuilder {
    kind: Option<PropSpecKind>,
    name: Option<String>,
    default_value: Option<serde_json::Value>,
    type_prop: Option<PropSpec>,
    entries: Vec<PropSpec>,
    validations: Vec<ValidationSpec>,
    func_unique_id: Option<FuncUniqueId>,
    inputs: Vec<AttrFuncInputSpec>,
    widget_kind: Option<PropSpecWidgetKind>,
    widget_options: Option<serde_json::Value>,
    doc_link: Option<Url>,
    hidden: bool,
}

impl PropSpecBuilder {
    #[allow(unused_mut)]
    pub fn default_value(&mut self, value: serde_json::Value) -> &mut Self {
        self.default_value = Some(value);
        self
    }

    #[allow(unused_mut)]
    pub fn kind(&mut self, value: PropSpecKind) -> &mut Self {
        self.kind = Some(value);
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
    pub fn validation(&mut self, value: impl Into<ValidationSpec>) -> &mut Self {
        self.validations.push(value.into());
        self
    }

    #[allow(unused_mut)]
    pub fn entries(&mut self, value: Vec<impl Into<PropSpec>>) -> &mut Self {
        self.entries = value.into_iter().map(Into::into).collect();
        self
    }

    #[allow(unused_mut)]
    pub fn func_unique_id(&mut self, value: FuncUniqueId) -> &mut Self {
        self.func_unique_id = Some(value);
        self
    }

    #[allow(unused_mut)]
    pub fn input(&mut self, value: impl Into<AttrFuncInputSpec>) -> &mut Self {
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

        let validations = self.validations.clone();
        let inputs = self.inputs.clone();
        let func_unique_id = self.func_unique_id;
        let widget_kind = self.widget_kind;
        let widget_options = self.widget_options.to_owned();
        let hidden = self.hidden;
        let doc_link = self.doc_link.to_owned();

        Ok(match self.kind {
            Some(kind) => match kind {
                PropSpecKind::String => PropSpec::String {
                    name,
                    default_value: match &self.default_value {
                        Some(serde_json::Value::String(s)) => Some(s.to_owned()),
                        Some(_) => {
                            return Err(SpecError::ValidationError(
                                "String prop must get a string as a default value".to_string(),
                            ));
                        }
                        None => None,
                    },
                    validations: Some(validations),
                    func_unique_id,
                    inputs: Some(inputs),
                    widget_kind,
                    widget_options,
                    hidden: Some(hidden),
                    doc_link,
                },
                PropSpecKind::Number => PropSpec::Number {
                    name,
                    default_value: match &self.default_value {
                        Some(value) => {
                            if value.is_i64() {
                                value.as_i64()
                            } else {
                                return Err(SpecError::ValidationError(
                                    "Number props must get an i64 as a default value".to_string(),
                                ));
                            }
                        }
                        None => None,
                    },
                    validations: Some(validations),
                    func_unique_id,
                    inputs: Some(inputs),
                    widget_kind,
                    widget_options,
                    hidden: Some(hidden),
                    doc_link,
                },
                PropSpecKind::Boolean => PropSpec::Boolean {
                    name,
                    default_value: match &self.default_value {
                        Some(value) => {
                            if value.is_boolean() {
                                value.as_bool()
                            } else {
                                return Err(SpecError::ValidationError(
                                    "Boolean props must get a bool as a default value".to_string(),
                                ));
                            }
                        }
                        None => None,
                    },
                    validations: Some(validations),
                    func_unique_id,
                    inputs: Some(inputs),
                    widget_kind,
                    widget_options,
                    hidden: Some(hidden),
                    doc_link,
                },
                PropSpecKind::Map => PropSpec::Map {
                    name,
                    // TODO: Validate these types
                    default_value: self.default_value.to_owned(),
                    type_prop: match self.type_prop {
                        Some(ref value) => Box::new(value.clone()),
                        None => {
                            return Err(UninitializedFieldError::from("type_prop").into());
                        }
                    },
                    validations: Some(validations),
                    func_unique_id,
                    inputs: Some(inputs),
                    widget_kind,
                    widget_options,
                    hidden: Some(hidden),
                    doc_link,
                },
                PropSpecKind::Array => PropSpec::Array {
                    name,
                    default_value: self.default_value.to_owned(),
                    type_prop: match self.type_prop {
                        Some(ref value) => Box::new(value.clone()),
                        None => {
                            return Err(UninitializedFieldError::from("type_prop").into());
                        }
                    },
                    validations: Some(validations),
                    func_unique_id,
                    inputs: Some(inputs),
                    widget_kind,
                    widget_options,
                    hidden: Some(hidden),
                    doc_link,
                },
                PropSpecKind::Object => PropSpec::Object {
                    name,
                    default_value: self.default_value.to_owned(),
                    entries: self.entries.clone(),
                    validations: Some(validations),
                    func_unique_id,
                    inputs: Some(inputs),
                    widget_kind,
                    widget_options,
                    hidden: Some(hidden),
                    doc_link,
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
