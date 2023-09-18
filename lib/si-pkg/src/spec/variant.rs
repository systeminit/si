use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, Display, EnumIter, EnumString};
use url::Url;

use super::{
    ActionFuncSpec, LeafFunctionSpec, PropSpec, PropSpecData, PropSpecWidgetKind, SiPropFuncSpec,
    SocketSpec, SpecError,
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
    #[serde(alias = "ConfigurationFrame")]
    #[strum(serialize = "ConfigurationFrame", serialize = "configurationFrame")]
    ConfigurationFrame,
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
}

impl SchemaVariantSpec {
    pub fn builder() -> SchemaVariantSpecBuilder {
        SchemaVariantSpecBuilder::default()
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
                validations: None,
                doc_link: None,
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
                validations: None,
                doc_link: None,
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
                validations: None,
                doc_link: None,
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
                validations: None,
                doc_link: None,
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
