use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use strum::{AsRefStr, Display, EnumIter, EnumString};
use url::Url;

use crate::spec::authentication_func::AuthenticationFuncSpec;

use super::{
    ActionFuncSpec, LeafFunctionSpec, MapKeyFuncSpec, PropSpec, PropSpecData, PropSpecKind,
    PropSpecWidgetKind, RootPropFuncSpec, SiPropFuncSpec, SocketSpec, SpecError,
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
#[serde(rename_all = "camelCase")]
pub enum SchemaVariantSpecComponentType {
    #[serde(alias = "AggregationFrame")]
    #[strum(serialize = "AggregationFrame", serialize = "aggregationFrame")]
    AggregationFrame,
    #[default]
    #[serde(alias = "Component")]
    #[strum(serialize = "Component", serialize = "component")]
    Component,
    #[serde(alias = "ConfigurationFrameDown")]
    #[strum(
        serialize = "ConfigurationFrameDown",
        serialize = "configurationFrameDown",
        serialize = "ConfigurationFrame",
        serialize = "configurationFrame"
    )] // this was called ConfigurationFrame so we need to keep compatibility
    ConfigurationFrameDown,
    #[strum(serialize = "ConfigurationFrameUp", serialize = "configurationFrameUp")]
    ConfigurationFrameUp,
}

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
)]
pub enum SchemaVariantSpecPropRoot {
    Domain,
    ResourceValue,
    SecretDefinition,
    Secrets,
}

impl SchemaVariantSpecPropRoot {
    pub fn path_parts(&self) -> &'static [&'static str] {
        match self {
            Self::Domain => &["root", "domain"],
            Self::ResourceValue => &["root", "resource_value"],
            Self::SecretDefinition => &["root", "secret_definition"],
            Self::Secrets => &["root", "secrets"],
        }
    }
}

#[derive(Builder, Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[builder(build_fn(error = "SpecError"))]
pub struct SchemaVariantSpecData {
    #[builder(setter(into))]
    pub name: String,
    #[builder(setter(into, strip_option), default)]
    pub link: Option<Url>,
    #[builder(setter(into, strip_option), default)]
    pub color: Option<String>,

    #[builder(setter(into), default)]
    pub component_type: SchemaVariantSpecComponentType,
    #[builder(setter(into))]
    pub func_unique_id: String,
}

impl SchemaVariantSpecData {
    pub fn builder() -> SchemaVariantSpecDataBuilder {
        SchemaVariantSpecDataBuilder::default()
    }
}

impl SchemaVariantSpecDataBuilder {
    #[allow(unused_mut)]
    pub fn try_link<V>(&mut self, value: V) -> Result<&mut Self, V::Error>
    where
        V: TryInto<Url>,
    {
        let converted: Url = value.try_into()?;
        Ok(self.link(converted))
    }
}

#[derive(Builder, Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[builder(build_fn(error = "SpecError"))]
pub struct SchemaVariantSpec {
    #[builder(setter(into))]
    pub name: String,

    #[builder(setter(into, strip_option), default)]
    pub data: Option<SchemaVariantSpecData>,

    #[builder(setter(into, strip_option), default)]
    #[serde(default)]
    pub unique_id: Option<String>,

    #[builder(setter(into), default)]
    #[serde(default)]
    pub deleted: bool,

    #[builder(setter(each(name = "action_func"), into), default)]
    pub action_funcs: Vec<ActionFuncSpec>,

    #[builder(setter(each(name = "auth_func"), into), default)]
    pub auth_funcs: Vec<AuthenticationFuncSpec>,

    #[builder(setter(each(name = "leaf_function"), into), default)]
    pub leaf_functions: Vec<LeafFunctionSpec>,

    #[builder(setter(each(name = "socket"), into), default)]
    pub sockets: Vec<SocketSpec>,

    #[builder(setter(each(name = "si_prop_func"), into), default)]
    pub si_prop_funcs: Vec<SiPropFuncSpec>,

    #[builder(private, default = "Self::default_domain()")]
    pub domain: PropSpec,

    #[builder(private, default = "Self::default_secrets()")]
    pub secrets: PropSpec,

    #[builder(private, default)]
    pub secret_definition: Option<PropSpec>,

    #[builder(private, default = "Self::default_resource_value()")]
    pub resource_value: PropSpec,

    #[builder(setter(each(name = "root_prop_func"), into), default)]
    #[serde(default)]
    pub root_prop_funcs: Vec<RootPropFuncSpec>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FlatPropSpec {
    pub unique_id: Option<String>,
    pub data: Option<PropSpecData>,
    pub kind: PropSpecKind,
    pub name: String,
    // We assume child of maps and arrays cant be set by a function, only the whole array/map
    pub type_prop: Option<Box<PropSpec>>,
    pub map_key_funcs: Option<Vec<MapKeyFuncSpec>>,
}

impl SchemaVariantSpec {
    pub fn builder() -> SchemaVariantSpecBuilder {
        SchemaVariantSpecBuilder::default()
    }

    pub fn flatten_domain(&self) -> Result<HashMap<String, FlatPropSpec>, SpecError> {
        let mut work_queue = VecDeque::new();
        work_queue.push_back((
            vec!["root".to_owned(), "domain".to_owned()],
            self.domain.clone(),
        ));

        self.flatten_prop(work_queue)
    }

    pub fn flatten_secrets(&self) -> Result<HashMap<String, FlatPropSpec>, SpecError> {
        let mut work_queue = VecDeque::new();
        work_queue.push_back((
            vec!["root".to_owned(), "secrets".to_owned()],
            self.secrets.clone(),
        ));

        self.flatten_prop(work_queue)
    }

    pub fn flatten_secret_definition(&self) -> Result<HashMap<String, FlatPropSpec>, SpecError> {
        let mut work_queue = VecDeque::new();
        if let Some(definition) = self.secret_definition.clone() {
            work_queue.push_back((
                vec!["root".to_owned(), "secret_definition".to_owned()],
                definition,
            ));
        }

        self.flatten_prop(work_queue)
    }

    fn flatten_prop(
        &self,
        mut work_queue: VecDeque<(Vec<String>, PropSpec)>,
    ) -> Result<HashMap<String, FlatPropSpec>, SpecError> {
        let mut map = HashMap::new();
        while let Some((path, prop)) = work_queue.pop_front() {
            match &prop {
                PropSpec::String { .. } => {}
                PropSpec::Array { .. } => {}
                PropSpec::Boolean { .. } => {}
                PropSpec::Map { .. } => {}
                PropSpec::Number { .. } => {}
                PropSpec::Object { entries, .. } => {
                    for entry in entries {
                        let mut path = path.clone();
                        path.push(entry.name().to_owned());
                        work_queue.push_back((path, entry.clone()));
                    }
                }
            }

            map.insert(
                path.join("/"),
                FlatPropSpec {
                    unique_id: prop.unique_id().map(ToOwned::to_owned),
                    data: prop.data().cloned(),
                    kind: prop.kind(),
                    name: prop.name().to_owned(),
                    type_prop: match &prop {
                        PropSpec::Array { type_prop, .. } => Some(type_prop.clone()),
                        PropSpec::Map { type_prop, .. } => Some(type_prop.clone()),
                        _ => None,
                    },
                    map_key_funcs: match &prop {
                        PropSpec::Map { map_key_funcs, .. } => map_key_funcs.clone(),
                        _ => None,
                    },
                },
            );
        }
        Ok(map)
    }
}

impl SchemaVariantSpecBuilder {
    // XXX: these need to take in a unique_id
    fn default_domain() -> PropSpec {
        PropSpec::Object {
            name: "domain".to_string(),
            unique_id: None,
            data: Some(PropSpecData {
                name: "domain".to_string(),
                default_value: None,
                func_unique_id: None,
                inputs: None,
                widget_kind: Some(PropSpecWidgetKind::Header),
                widget_options: None,
                hidden: Some(false),
                doc_link: None,
                documentation: None,
                validation_format: None,
            }),
            entries: vec![],
        }
    }

    fn default_secrets() -> PropSpec {
        PropSpec::Object {
            name: "secrets".to_string(),
            unique_id: None,
            data: Some(PropSpecData {
                name: "secrets".to_string(),
                default_value: None,
                func_unique_id: None,
                inputs: None,
                widget_kind: Some(PropSpecWidgetKind::Header),
                widget_options: None,
                hidden: Some(false),
                doc_link: None,
                documentation: None,
                validation_format: None,
            }),
            entries: vec![],
        }
    }

    fn default_secret_definition() -> Option<PropSpec> {
        Some(PropSpec::Object {
            name: "secret_definition".to_string(),
            unique_id: None,
            data: Some(PropSpecData {
                name: "secret_definition".to_string(),
                default_value: None,
                func_unique_id: None,
                inputs: None,
                widget_kind: Some(PropSpecWidgetKind::Header),
                widget_options: None,
                hidden: Some(false),
                doc_link: None,
                documentation: None,
                validation_format: None,
            }),
            entries: vec![],
        })
    }

    fn default_resource_value() -> PropSpec {
        PropSpec::Object {
            name: "value".to_string(),
            unique_id: None,
            data: Some(PropSpecData {
                name: "value".to_string(),
                default_value: None,
                func_unique_id: None,
                inputs: None,
                widget_kind: Some(PropSpecWidgetKind::Header),
                widget_options: None,
                hidden: Some(false),
                doc_link: None,
                documentation: None,
                validation_format: None,
            }),
            entries: vec![],
        }
    }

    pub fn domain_prop(&mut self, item: impl Into<PropSpec>) -> &mut Self {
        self.prop(SchemaVariantSpecPropRoot::Domain, item)
    }

    pub fn secret_prop(&mut self, item: impl Into<PropSpec>) -> &mut Self {
        self.prop(SchemaVariantSpecPropRoot::Secrets, item)
    }

    pub fn secret_definition_prop(&mut self, item: impl Into<PropSpec>) -> &mut Self {
        self.prop(SchemaVariantSpecPropRoot::SecretDefinition, item)
    }

    pub fn resource_value_prop(&mut self, item: impl Into<PropSpec>) -> &mut Self {
        self.prop(SchemaVariantSpecPropRoot::ResourceValue, item)
    }

    #[allow(unused_mut)]
    pub fn prop(
        &mut self,
        root: SchemaVariantSpecPropRoot,
        item: impl Into<PropSpec>,
    ) -> &mut Self {
        let converted: PropSpec = item.into();
        match match root {
            SchemaVariantSpecPropRoot::Domain => {
                self.domain.get_or_insert_with(Self::default_domain)
            }
            SchemaVariantSpecPropRoot::ResourceValue => self
                .resource_value
                .get_or_insert_with(Self::default_resource_value),
            SchemaVariantSpecPropRoot::SecretDefinition => self
                .secret_definition
                .get_or_insert_with(Self::default_secret_definition)
                .as_mut()
                .expect("secret_definition was created with Some(...)"),
            SchemaVariantSpecPropRoot::Secrets => {
                self.secrets.get_or_insert_with(Self::default_secrets)
            }
        } {
            PropSpec::Object { entries, .. } => entries.push(converted),
            invalid => unreachable!(
                "{:?} prop under root should be Object but was found to be: {:?}",
                root, invalid
            ),
        };
        self
    }

    #[allow(unused_mut)]
    pub fn try_prop<I>(
        &mut self,
        root: SchemaVariantSpecPropRoot,
        item: I,
    ) -> Result<&mut Self, I::Error>
    where
        I: TryInto<PropSpec>,
    {
        let converted: PropSpec = item.try_into()?;
        Ok(self.prop(root, converted))
    }

    #[allow(unused_mut)]
    pub fn props(&mut self, value: Vec<PropSpec>) -> &mut Self {
        match self.domain.get_or_insert_with(Self::default_domain) {
            PropSpec::Object { entries, .. } => *entries = value,
            invalid => unreachable!(
                "domain prop is an object but was found to be: {:?}",
                invalid
            ),
        };
        self
    }
}
