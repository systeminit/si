use crate::FuncUniqueId;
use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, Display, EnumIter, EnumString};
use url::Url;

use super::{
    ActionFuncSpec, FuncDescriptionSpec, LeafFunctionSpec, PropSpec, PropSpecWidgetKind,
    SiPropFuncSpec, SocketSpec, SpecError,
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
    AggregationFrame,
    #[default]
    Component,
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
}

#[derive(Builder, Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[builder(build_fn(error = "SpecError"))]
pub struct SchemaVariantSpec {
    #[builder(setter(into))]
    pub name: String,
    #[builder(setter(into, strip_option), default)]
    pub link: Option<Url>,
    #[builder(setter(into, strip_option), default)]
    pub color: Option<String>,

    #[builder(setter(into), default)]
    pub component_type: SchemaVariantSpecComponentType,

    #[builder(private, default = "Self::default_domain()")]
    pub domain: PropSpec,

    #[builder(private, default = "Self::default_resource_value()")]
    pub resource_value: PropSpec,

    #[builder(setter(into))]
    pub func_unique_id: FuncUniqueId,

    #[builder(setter(each(name = "action_func"), into), default)]
    pub action_funcs: Vec<ActionFuncSpec>,

    #[builder(setter(each(name = "leaf_function"), into), default)]
    pub leaf_functions: Vec<LeafFunctionSpec>,

    #[builder(setter(each(name = "func_description"), into), default)]
    pub func_descriptions: Vec<FuncDescriptionSpec>,

    #[builder(setter(each(name = "socket"), into), default)]
    pub sockets: Vec<SocketSpec>,

    #[builder(setter(each(name = "si_prop_func"), into), default)]
    pub si_prop_funcs: Vec<SiPropFuncSpec>,
}

impl SchemaVariantSpec {
    pub fn builder() -> SchemaVariantSpecBuilder {
        SchemaVariantSpecBuilder::default()
    }
}

impl SchemaVariantSpecBuilder {
    fn default_domain() -> PropSpec {
        PropSpec::Object {
            validations: None,
            default_value: None,
            name: "domain".to_string(),
            entries: vec![],
            func_unique_id: None,
            inputs: None,
            widget_kind: Some(PropSpecWidgetKind::Header),
            widget_options: None,
            hidden: Some(false),
            doc_link: None,
        }
    }

    fn default_resource_value() -> PropSpec {
        PropSpec::Object {
            validations: None,
            default_value: None,
            name: "value".to_string(),
            entries: vec![],
            func_unique_id: None,
            inputs: None,
            widget_kind: Some(PropSpecWidgetKind::Header),
            widget_options: None,
            hidden: Some(true),
            doc_link: None,
        }
    }

    #[allow(unused_mut)]
    pub fn try_link<V>(&mut self, value: V) -> Result<&mut Self, V::Error>
    where
        V: TryInto<Url>,
    {
        let converted: Url = value.try_into()?;
        Ok(self.link(converted))
    }

    pub fn domain_prop(&mut self, item: impl Into<PropSpec>) -> &mut Self {
        self.prop(SchemaVariantSpecPropRoot::Domain, item)
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
        } {
            PropSpec::Object { entries, .. } => entries.push(converted),
            invalid => unreachable!(
                "domain prop is an object but was found to be: {:?}",
                invalid
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
