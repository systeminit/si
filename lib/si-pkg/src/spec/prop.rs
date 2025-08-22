use std::collections::{
    HashMap,
    HashSet,
};

use derive_builder::UninitializedFieldError;
use indexmap::IndexMap;
use serde::{
    Deserialize,
    Serialize,
};
use strum::{
    AsRefStr,
    Display,
    EnumIter,
    EnumString,
};
use url::Url;

use super::{
    AttrFuncInputSpec,
    HasUniqueId,
    MapKeyFuncSpec,
    SpecError,
};

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
            PropSpec::String { .. }
            | PropSpec::Number { .. }
            | PropSpec::Float { .. }
            | PropSpec::Json { .. } => Self::Text,
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
    pub ui_optionals: Option<HashMap<String, serde_json::Value>>,
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
    Float {
        name: String,
        data: Option<PropSpecData>,
        unique_id: Option<String>,
    },
    #[serde(rename_all = "camelCase")]
    Json {
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

#[remain::sorted]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MergeSkip {
    FuncInputInputSocketMissing {
        prop_path: String,
        missing_socket_name: String,
        input_name: String,
        func_unique_id: String,
    },
    FuncInputOutputSocketMissing {
        prop_path: String,
        missing_socket_name: String,
        input_name: String,
        func_unique_id: String,
    },
    FuncInputPropMissing {
        prop_path: String,
        input_name: String,
        missing_prop_path: String,
        func_unique_id: String,
    },
    InputSocketMissing {
        socket_name: String,
    },
    OutputSocketMissing {
        socket_name: String,
    },
    PropKindMismatch {
        path: String,
        other_kind: PropSpecKind,
        self_kind: PropSpecKind,
    },
    PropMissing(String),
}

pub const PROP_PATH_SEPARATOR: &str = "\x0B";
const SI_PATH: &str = "root\x0Bsi";

pub(crate) enum InputMismatchTruth<'a, 'b> {
    PropSpecMap(&'a IndexMap<String, (&'b PropSpec, Option<String>)>),
    MissingPropSet(&'a HashSet<String>),
}

impl PropSpec {
    pub fn builder() -> PropSpecBuilder {
        PropSpecBuilder::default()
    }

    pub fn anonymize(&mut self) {
        let (unique_id, data) = match self {
            PropSpec::Array {
                type_prop,
                unique_id,
                data,
                ..
            }
            | PropSpec::Map {
                type_prop,
                unique_id,
                data,
                ..
            } => {
                type_prop.anonymize();
                (unique_id, data)
            }
            PropSpec::Object {
                entries,
                unique_id,
                data,
                ..
            } => {
                entries.iter_mut().for_each(|f| f.anonymize());
                (unique_id, data)
            }
            PropSpec::Boolean {
                unique_id, data, ..
            }
            | PropSpec::Float {
                unique_id, data, ..
            }
            | PropSpec::Json {
                unique_id, data, ..
            }
            | PropSpec::Number {
                unique_id, data, ..
            }
            | PropSpec::String {
                unique_id, data, ..
            } => (unique_id, data),
        };

        *unique_id = None;
        if let Some(PropSpecData {
            inputs: Some(inputs),
            ..
        }) = data
        {
            inputs.iter_mut().for_each(AttrFuncInputSpec::anonymize)
        }
    }

    pub(crate) fn to_builder_without_children(&self) -> PropSpecBuilder {
        let mut builder = PropSpec::builder();
        builder.name(self.name()).kind(self.kind());
        if let Some(PropSpecData {
            name: _, // handled at the PropSpec level
            validation_format,
            default_value,
            func_unique_id,
            inputs,
            widget_kind,
            widget_options,
            hidden,
            doc_link,
            documentation,
            ui_optionals,
        }) = self.data()
        {
            if let Some(validation_format) = validation_format {
                builder.validation_format(validation_format.as_str());
            }
            if let Some(default_value) = default_value {
                builder.default_value(default_value.to_owned());
            }
            if let Some(func_unique_id) = func_unique_id {
                builder.func_unique_id(func_unique_id.as_str());
            }
            if let Some(inputs) = inputs {
                builder.inputs(inputs.to_owned());
            }
            if let Some(widget_kind) = widget_kind {
                builder.widget_kind(widget_kind.to_owned());
            }
            if let Some(widget_options) = widget_options {
                builder.widget_options(widget_options.to_owned());
            }
            if let Some(doc_link) = doc_link {
                builder.doc_link(doc_link.to_owned());
            }
            if let Some(docs) = documentation {
                builder.documentation(docs.as_str());
            }
            if let &Some(hidden) = hidden {
                builder.hidden(hidden);
            }
            if let Some(ui_optionals) = ui_optionals {
                builder.ui_optionals(ui_optionals.to_owned());
            }
        }

        if let PropSpec::Map {
            map_key_funcs: Some(map_key_funcs),
            ..
        } = self
        {
            builder.map_key_funcs(map_key_funcs.to_owned());
        }

        builder
    }

    pub fn name(&self) -> &str {
        match self {
            Self::Array { name, .. }
            | Self::Boolean { name, .. }
            | Self::Map { name, .. }
            | Self::Json { name, .. }
            | Self::Number { name, .. }
            | Self::Float { name, .. }
            | Self::Object { name, .. }
            | Self::String { name, .. } => name.as_str(),
        }
    }

    pub fn kind(&self) -> PropSpecKind {
        match self {
            Self::Array { .. } => PropSpecKind::Array,
            Self::Boolean { .. } => PropSpecKind::Boolean,
            Self::Json { .. } => PropSpecKind::Json,
            Self::Map { .. } => PropSpecKind::Map,
            Self::Number { .. } => PropSpecKind::Number,
            Self::Float { .. } => PropSpecKind::Float,
            Self::Object { .. } => PropSpecKind::Object,
            Self::String { .. } => PropSpecKind::String,
        }
    }

    pub fn data(&self) -> Option<&PropSpecData> {
        match self {
            Self::Array { data, .. }
            | Self::Boolean { data, .. }
            | Self::Map { data, .. }
            | Self::Number { data, .. }
            | Self::Float { data, .. }
            | Self::Object { data, .. }
            | Self::Json { data, .. }
            | Self::String { data, .. } => data.as_ref(),
        }
    }

    pub fn inputs(&self) -> Option<&Vec<AttrFuncInputSpec>> {
        self.data().and_then(|data| data.inputs.as_ref())
    }

    pub fn func_unique_id(&self) -> Option<&str> {
        self.data().and_then(|data| data.func_unique_id.as_deref())
    }

    pub fn direct_children(&self) -> Vec<&PropSpec> {
        // would be better to just produce an iterator here
        match self {
            Self::Json { .. }
            | Self::Boolean { .. }
            | Self::Number { .. }
            | Self::Float { .. }
            | Self::String { .. } => vec![],
            Self::Object { entries, .. } => entries.iter().collect(),
            Self::Map { type_prop, .. } | Self::Array { type_prop, .. } => vec![type_prop.as_ref()],
        }
    }

    pub(crate) fn make_path(parts: &[impl Into<String> + Clone], with_sep: Option<&str>) -> String {
        parts
            .iter()
            .map(|part| part.clone().into())
            .collect::<Vec<String>>()
            .join(with_sep.unwrap_or(PROP_PATH_SEPARATOR))
    }

    /// Process a PropSpec tree into a mapping between the path of the prop and the PropSpec and a list of its direct child paths
    pub(crate) fn build_prop_spec_index_map(
        &self,
    ) -> IndexMap<String, (&PropSpec, Option<String>)> {
        let mut prop_map = IndexMap::new();

        let mut prop_queue = Vec::from([(self, self.name().to_owned(), None)]);
        while let Some((current_prop, current_path, maybe_parent)) = prop_queue.pop() {
            let children = current_prop.direct_children();
            let children_as_paths: Vec<String> = children
                .iter()
                .map(|&prop_spec| Self::make_path(&[current_path.as_str(), prop_spec.name()], None))
                .collect();

            prop_queue.extend(children.into_iter().enumerate().map(|(idx, spec)| {
                (
                    spec,
                    children_as_paths
                        .get(idx)
                        .expect("this index will exist")
                        .to_owned(),
                    Some(current_path.to_owned()),
                )
            }));

            prop_map.insert(current_path, (current_prop, maybe_parent));
        }

        prop_map
    }

    pub(crate) fn get_input_mismatches(
        current_path: &str,
        prop_truth: InputMismatchTruth,
        other_inputs: &[AttrFuncInputSpec],
        other_func_unique_id: &str,
        input_sockets: &[String],
        output_sockets: &[String],
    ) -> Vec<MergeSkip> {
        let mut merge_skips = vec![];

        for other_input in other_inputs {
            match other_input {
                AttrFuncInputSpec::Prop {
                    prop_path, name, ..
                } => {
                    // If we take a prop from the SI tree as an input we won't
                    // be present in the prop map but will exist when the schema
                    // variant is created
                    if prop_path.starts_with(SI_PATH) {
                        continue;
                    }

                    let prop_is_missing = match prop_truth {
                        InputMismatchTruth::MissingPropSet(missing_prop_set) => {
                            missing_prop_set.contains(prop_path)
                        }
                        InputMismatchTruth::PropSpecMap(prop_spec_map) => {
                            !prop_spec_map.contains_key(prop_path)
                        }
                    };

                    if prop_is_missing {
                        merge_skips.push(MergeSkip::FuncInputPropMissing {
                            prop_path: current_path.to_string(),
                            input_name: name.to_owned(),
                            missing_prop_path: prop_path.to_owned(),
                            func_unique_id: other_func_unique_id.to_string(),
                        })
                    }
                }
                AttrFuncInputSpec::InputSocket {
                    name, socket_name, ..
                } => {
                    if !input_sockets.contains(socket_name) {
                        merge_skips.push(MergeSkip::FuncInputInputSocketMissing {
                            prop_path: current_path.to_string(),
                            input_name: name.to_owned(),
                            missing_socket_name: socket_name.to_owned(),
                            func_unique_id: other_func_unique_id.to_string(),
                        })
                    }
                }
                AttrFuncInputSpec::OutputSocket {
                    name, socket_name, ..
                } => {
                    if !output_sockets.contains(socket_name) {
                        merge_skips.push(MergeSkip::FuncInputOutputSocketMissing {
                            prop_path: current_path.to_string(),
                            input_name: name.to_owned(),
                            missing_socket_name: socket_name.to_owned(),
                            func_unique_id: other_func_unique_id.to_string(),
                        })
                    }
                }
            }
        }

        merge_skips
    }

    pub fn merge_with(
        &self,
        other: &PropSpec,
        input_sockets: &[String],
        output_sockets: &[String],
    ) -> (PropSpec, Vec<MergeSkip>) {
        let other_map = other.build_prop_spec_index_map();
        let mut self_map = self.build_prop_spec_index_map();
        self_map.reverse(); // reversing this means we walk it leaves to parents

        let mut merge_skips: Vec<MergeSkip> = other_map
            .keys()
            .filter_map(|path| {
                if self_map.contains_key(path) {
                    None
                } else {
                    Some(MergeSkip::PropMissing(path.to_owned()))
                }
            })
            .collect();

        let mut child_map: HashMap<String, Vec<PropSpec>> = HashMap::new();
        for (current_path, (current_prop_spec, maybe_parent_path)) in &self_map {
            let mut current_prop_spec_builder = current_prop_spec.to_builder_without_children();

            // Merge in the inputs from the matching prop in other, if it exists
            if let Some(&(other_prop_spec, _)) = other_map.get(current_path) {
                let other_kind = other_prop_spec.kind();
                let self_kind = current_prop_spec.kind();

                if other_kind != self_kind {
                    merge_skips.push(MergeSkip::PropKindMismatch {
                        path: current_path.to_owned(),
                        other_kind,
                        self_kind,
                    });
                } else if let (Some(other_func_unique_id), Some(other_inputs)) =
                    (other_prop_spec.func_unique_id(), other_prop_spec.inputs())
                {
                    let mismatches = Self::get_input_mismatches(
                        current_path,
                        InputMismatchTruth::PropSpecMap(&self_map),
                        other_inputs.as_slice(),
                        other_func_unique_id,
                        input_sockets,
                        output_sockets,
                    );

                    if mismatches.is_empty() {
                        current_prop_spec_builder.func_unique_id(other_func_unique_id);
                        current_prop_spec_builder.inputs(other_inputs.to_owned());
                    } else {
                        merge_skips.extend(mismatches);
                    }
                }
            }

            match current_prop_spec.kind() {
                PropSpecKind::Map | PropSpecKind::Array => {
                    if let Some(children) = child_map.get(current_path) {
                        if let Some(type_child) = children.first() {
                            current_prop_spec_builder.type_prop(type_child.to_owned());
                        }
                    }
                }
                PropSpecKind::Object => {
                    if let Some(children) = child_map.get(current_path) {
                        for entry in children {
                            current_prop_spec_builder.entry(entry.to_owned());
                        }
                    }
                }
                _ => {}
            }

            let current_prop = current_prop_spec_builder
                .build()
                .expect("failure here is a programming error");

            match maybe_parent_path {
                Some(parent_path) => {
                    child_map
                        .entry(parent_path.to_owned())
                        .and_modify(|children| children.push(current_prop.clone()))
                        .or_insert_with(|| vec![current_prop]);
                }
                None => return (current_prop, merge_skips),
            }
        }

        // unreachable, but the compiler doesn't know this
        (self.to_owned(), vec![])
    }
}

#[remain::sorted]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum PropSpecKind {
    Array,
    Boolean,
    Float,
    Json,
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
    pub name: Option<String>,
    type_prop: Option<PropSpec>,
    validation_format: Option<String>,
    ui_optionals: Option<HashMap<String, serde_json::Value>>,
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
            ui_optionals: None,
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
    pub fn ui_optionals(
        &mut self,
        value: impl Into<HashMap<String, serde_json::Value>>,
    ) -> &mut Self {
        self.has_data = true;
        self.ui_optionals = Some(value.into());
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

    pub fn input(&mut self, value: impl Into<AttrFuncInputSpec>) -> &mut Self {
        self.has_data = true;
        self.inputs.push(value.into());
        self
    }

    pub fn inputs(&mut self, inputs: Vec<AttrFuncInputSpec>) -> &mut Self {
        self.has_data = true;
        self.inputs = inputs;
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

    pub fn map_key_funcs(&mut self, value: Vec<MapKeyFuncSpec>) -> &mut Self {
        self.has_data = true;
        self.map_key_funcs = value;
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
        let unique_id = self.unique_id.clone();
        let data = if self.has_data {
            Some(PropSpecData {
                name: name.clone(),
                validation_format: self.validation_format.clone(),
                default_value: self.default_value.clone(),
                func_unique_id: self.func_unique_id.clone(),
                inputs: Some(self.inputs.clone()),
                widget_kind: self.widget_kind,
                widget_options: self.widget_options.clone(),
                hidden: Some(self.hidden),
                doc_link: self.doc_link.clone(),
                documentation: self.documentation.clone(),
                ui_optionals: Some(self.ui_optionals.clone().unwrap_or_default()),
            })
        } else {
            None
        };

        Ok(match self.kind {
            Some(kind) => match kind {
                PropSpecKind::String => PropSpec::String {
                    name,
                    unique_id,
                    data,
                },
                PropSpecKind::Json => PropSpec::Json {
                    name,
                    unique_id,
                    data,
                },
                PropSpecKind::Number => PropSpec::Number {
                    name,
                    unique_id,
                    data,
                },
                PropSpecKind::Float => PropSpec::Float {
                    name,
                    unique_id,
                    data,
                },
                PropSpecKind::Boolean => PropSpec::Boolean {
                    name,
                    unique_id,
                    data,
                },
                PropSpecKind::Map => PropSpec::Map {
                    name,
                    unique_id,
                    data,
                    type_prop: match self.type_prop {
                        Some(ref value) => Box::new(value.clone()),
                        None => {
                            return Err(UninitializedFieldError::from("type_prop").into());
                        }
                    },
                    map_key_funcs: Some(self.map_key_funcs.to_owned()),
                },
                PropSpecKind::Array => PropSpec::Array {
                    name,
                    unique_id,
                    data,
                    type_prop: match self.type_prop {
                        Some(ref value) => Box::new(value.clone()),
                        None => {
                            return Err(UninitializedFieldError::from("type_prop").into());
                        }
                    },
                },
                PropSpecKind::Object => PropSpec::Object {
                    name,
                    unique_id,
                    data,
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

impl HasUniqueId for PropSpec {
    fn unique_id(&self) -> Option<&str> {
        match self {
            Self::Array { unique_id, .. }
            | Self::Boolean { unique_id, .. }
            | Self::Float { unique_id, .. }
            | Self::Json { unique_id, .. }
            | Self::Map { unique_id, .. }
            | Self::Number { unique_id, .. }
            | Self::Object { unique_id, .. }
            | Self::String { unique_id, .. } => unique_id.as_deref(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prop_merge() {
        let prop_a_path = PropSpec::make_path(&["root", "a"], None);
        let prop_b_path = PropSpec::make_path(&["root", "b"], None);
        let prop_c_path = PropSpec::make_path(&["root", "c"], None);
        let pride_path = PropSpec::make_path(&["root", "objét d'art", "l'orgueil"], None);

        let prop_b_input_spec = AttrFuncInputSpec::Prop {
            name: "arg_1".into(),
            prop_path: prop_a_path.to_owned(),
            unique_id: None,
            deleted: false,
        };

        let prop_tree_a = PropSpec::builder()
            .name("root")
            .kind(PropSpecKind::Object)
            .entry(
                PropSpec::builder()
                    .name("a")
                    .kind(PropSpecKind::String)
                    .func_unique_id("function_2")
                    .input(AttrFuncInputSpec::Prop {
                        name: "arg_1".into(),
                        prop_path: prop_c_path.to_owned(),
                        unique_id: None,
                        deleted: false,
                    })
                    .build()
                    .expect("able to build prop a"),
            )
            .entry(
                PropSpec::builder()
                    .name("b")
                    .kind(PropSpecKind::Number)
                    .func_unique_id("function_1")
                    .input(prop_b_input_spec.to_owned())
                    .build()
                    .expect("able to build prop b"),
            )
            .entry(
                PropSpec::builder()
                    .name("c")
                    .kind(PropSpecKind::String)
                    .build()
                    .expect("able to build prop a"),
            )
            .entry(
                PropSpec::builder()
                    .name("objét d'art")
                    .kind(PropSpecKind::Object)
                    .entry(
                        PropSpec::builder()
                            .name("l'orgueil")
                            .kind(PropSpecKind::Number)
                            .build()
                            .expect("before the fall"),
                    )
                    .entry(
                        PropSpec::builder()
                            .name("un morceau")
                            .kind(PropSpecKind::Object)
                            .entry(
                                PropSpec::builder()
                                    .name("le couleur? bleu")
                                    .kind(PropSpecKind::Number)
                                    .build()
                                    .expect("should build"),
                            )
                            .build()
                            .expect("morceau?"),
                    )
                    .build()
                    .expect("objét?"),
            )
            .build()
            .expect("able to build");

        let prop_tree_b = PropSpec::builder()
            .name("root")
            .kind(PropSpecKind::Object)
            .entry(
                PropSpec::builder()
                    .name("a")
                    .kind(PropSpecKind::String)
                    .build()
                    .expect("able to build prop a"),
            )
            .entry(
                PropSpec::builder()
                    .name("b")
                    .kind(PropSpecKind::Number)
                    .build()
                    .expect("able to build prop b"),
            )
            .entry(
                PropSpec::builder()
                    .name("objét d'art")
                    .kind(PropSpecKind::Object)
                    .entry(
                        PropSpec::builder()
                            .name("un morceau")
                            .kind(PropSpecKind::Object)
                            .entry(
                                PropSpec::builder()
                                    .name("le couleur? bleu")
                                    .kind(PropSpecKind::Number)
                                    .build()
                                    .expect("should build"),
                            )
                            .build()
                            .expect("morceau?"),
                    )
                    .build()
                    .expect("objét?"),
            )
            .entry(
                PropSpec::builder()
                    .name("objét de fart")
                    .kind(PropSpecKind::Object)
                    .entry(
                        PropSpec::builder()
                            .name("un morceau de pet")
                            .kind(PropSpecKind::Object)
                            .entry(
                                PropSpec::builder()
                                    .name("un chien")
                                    .kind(PropSpecKind::String)
                                    .build()
                                    .expect("a dog?"),
                            )
                            .build()
                            .expect("morceau?"),
                    )
                    .build()
                    .expect("objét?"),
            )
            .build()
            .expect("able to build");

        let (merged_prop_root, merge_skips) = prop_tree_b.merge_with(&prop_tree_a, &[], &[]);

        // Confirm merge skips are correct
        assert_eq!(
            &[
                MergeSkip::PropMissing(pride_path),
                MergeSkip::PropMissing(prop_c_path.to_owned()),
                MergeSkip::FuncInputPropMissing {
                    prop_path: prop_a_path,
                    input_name: "arg_1".into(),
                    missing_prop_path: prop_c_path,
                    func_unique_id: "function_2".into(),
                }
            ],
            merge_skips.as_slice(),
            "correct merge skips reported"
        );

        let prop_b_after_merge = merged_prop_root
            .build_prop_spec_index_map()
            .get(&prop_b_path)
            .expect("prop b present in new prop spec")
            .to_owned()
            .0;
        // Confirm attribute function copied over
        assert_eq!(
            Some(&vec![prop_b_input_spec]),
            prop_b_after_merge.inputs(),
            "attribute function for prop b copied over in merge"
        );
    }
}
