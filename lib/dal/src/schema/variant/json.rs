use std::collections::HashMap;

use serde::{
    Deserialize,
    Serialize,
};
use serde_json::Value;
use si_pkg::{
    MapKeyFuncSpec,
    PropSpec,
    SchemaSpec,
    SchemaSpecData,
    SchemaVariantSpec,
    SchemaVariantSpecData,
    SocketSpec,
    SocketSpecArity,
    SocketSpecData,
    SocketSpecKind,
};

use crate::{
    ComponentType,
    PropKind,
    SchemaVariantError,
    SocketArity,
    property_editor::schema::WidgetKind,
    schema::variant::{
        DEFAULT_SCHEMA_VARIANT_COLOR,
        SchemaVariantResult,
        ValueFrom,
        value_from::SiPropValueFrom,
    },
};

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SchemaVariantMetadataJson {
    /// Name for the schema.
    pub schema_name: String,
    /// Version of this variant.
    pub version: String,
    /// Override for the UI name for this schema variant
    #[serde(alias = "display_name")]
    pub display_name: String,
    /// The category this schema variant belongs to
    pub category: String,
    /// The color for the component on the component diagram as a hex string
    pub color: String,
    #[serde(alias = "component_type")]
    pub component_type: ComponentType,
    pub link: Option<String>,
    pub description: Option<String>,
}

impl SchemaVariantMetadataJson {
    pub fn to_schema_spec(&self, variant: SchemaVariantSpec) -> SchemaVariantResult<SchemaSpec> {
        let mut builder = SchemaSpec::builder();
        builder.name(&self.schema_name);
        let mut data_builder = SchemaSpecData::builder();
        data_builder.name(&self.schema_name);
        data_builder.category(&self.category);
        data_builder.category_name(&self.display_name);
        builder.data(data_builder.build()?);
        builder.variant(variant);

        Ok(builder.build()?)
    }
}

/// The json definition for a [`SchemaVariant`](crate::SchemaVariant)'s [`Prop`](crate::Prop) tree (and
/// more in the future).
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SchemaVariantJson {
    /// The immediate child [`Props`](crate::Prop) underneath "/root/domain".
    #[serde(default)]
    pub props: Vec<PropDefinition>,
    /// The immediate child [`Props`](crate::Prop) underneath "/root/secrets".
    #[serde(default)]
    pub secret_props: Vec<PropDefinition>,
    /// The immediate child [`Props`](crate::Prop) underneath "/root/secretsDefinition".
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub secret_definition: Option<Vec<PropDefinition>>,
    /// The immediate child [`Props`](crate::Prop) underneath "/root/resource_value".
    #[serde(default)]
    pub resource_props: Vec<PropDefinition>,
    /// Identity relationships for [`Props`](crate::Prop) underneath "/root/si".
    #[serde(default)]
    pub si_prop_value_froms: Vec<SiPropValueFrom>,

    /// The input [`Sockets`](crate::Socket) and created for the [`variant`](crate::SchemaVariant).
    #[serde(default)]
    pub input_sockets: Vec<SocketDefinition>,
    /// The output [`Sockets`](crate::Socket) and created for the [`variant`](crate::SchemaVariant).
    #[serde(default)]
    pub output_sockets: Vec<SocketDefinition>,
    /// A map of documentation links to reference. To reference links (values) specify the key via
    /// the "doc_link_ref" field for a [`PropDefinition`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub doc_links: Option<HashMap<String, String>>,
    /// Extra optional UI data we're just passing all the way through
    #[serde(flatten)]
    pub ui_optionals: serde_json::Value,
}

impl SchemaVariantJson {
    pub fn to_spec(
        &self,
        metadata: SchemaVariantMetadataJson,
        identity_func_unique_id: &str,
        asset_func_spec_unique_id: &str,
    ) -> SchemaVariantResult<SchemaVariantSpec> {
        let SchemaVariantMetadataJson {
            schema_name: _, // we store this elsewhere
            category: _,    // we store this elsewhere
            version,
            display_name,
            color,
            component_type,
            link,
            description,
        } = metadata;
        let mut builder = SchemaVariantSpec::builder();
        builder.version(version.clone());

        let mut data_builder = SchemaVariantSpecData::builder();

        data_builder.version(version.clone());
        data_builder.display_name(display_name);
        data_builder.color(color);
        data_builder.component_type(component_type);
        if let Some(link) = link {
            data_builder.try_link(link.as_str())?;
        }
        data_builder.description(description);

        data_builder.func_unique_id(asset_func_spec_unique_id);
        builder.data(data_builder.build()?);

        for si_prop_value_from in &self.si_prop_value_froms {
            builder.si_prop_func(si_prop_value_from.to_spec(identity_func_unique_id));
        }
        for prop in &self.props {
            builder.domain_prop(prop.to_spec(identity_func_unique_id)?);
        }
        for prop in &self.secret_props {
            builder.secret_prop(prop.to_spec(identity_func_unique_id)?);
        }
        if let Some(props) = &self.secret_definition {
            for prop in props {
                builder.secret_definition_prop(prop.to_spec(identity_func_unique_id)?);
            }
        }
        for resource_prop in &self.resource_props {
            builder.resource_value_prop(resource_prop.to_spec(identity_func_unique_id)?);
        }
        for input_socket in &self.input_sockets {
            builder.socket(input_socket.to_spec(true, identity_func_unique_id)?);
        }
        for output_socket in &self.output_sockets {
            builder.socket(output_socket.to_spec(false, identity_func_unique_id)?);
        }

        Ok(builder.build()?)
    }

    pub fn metadata_from_spec(
        schema_spec: SchemaSpec,
    ) -> SchemaVariantResult<SchemaVariantMetadataJson> {
        let schema_data = schema_spec.data.unwrap_or(SchemaSpecData {
            name: schema_spec.name.to_owned(),
            default_schema_variant: None,
            category: "".into(),
            category_name: None,
            ui_hidden: false,
        });

        let default_variant_spec = match schema_data.default_schema_variant {
            Some(default_variant_unique_id) => schema_spec
                .variants
                .iter()
                .find(|variant| variant.unique_id.as_deref() == Some(&default_variant_unique_id))
                .ok_or(SchemaVariantError::DefaultVariantNotFound(
                    default_variant_unique_id,
                ))?,
            None => schema_spec
                .variants
                .last()
                .ok_or(SchemaVariantError::NoVariants)?,
        };

        let variant_spec_data =
            default_variant_spec
                .data
                .to_owned()
                .unwrap_or(SchemaVariantSpecData {
                    version: "v0".into(),
                    display_name: None,
                    color: None,
                    link: None,
                    component_type: si_pkg::SchemaVariantSpecComponentType::Component,
                    func_unique_id: "0".into(),
                    description: None,
                });

        let metadata = SchemaVariantMetadataJson {
            version: variant_spec_data.version,
            schema_name: schema_spec.name.clone(),
            display_name: schema_data.category_name.unwrap_or(schema_spec.name),
            category: schema_data.category,
            color: variant_spec_data
                .color
                .to_owned()
                .unwrap_or(DEFAULT_SCHEMA_VARIANT_COLOR.into()),
            component_type: variant_spec_data.component_type.into(),
            link: variant_spec_data.link.as_ref().map(|l| l.to_string()),
            description: variant_spec_data.description.to_owned(),
        };

        Ok(metadata)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PropWidgetDefinition {
    /// The [`kind`](crate::property_editor::schema::WidgetKind) of the [`Prop`](crate::Prop) to be created.
    kind: WidgetKind,
    /// The `Option<Value>` of the [`kind`](crate::property_editor::schema::WidgetKind) to be created.
    #[serde(default)]
    options: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MapKeyFunc {
    pub key: String,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_from: Option<ValueFrom>,
}

impl MapKeyFunc {
    pub fn to_spec(&self, identity_func_unique_id: &str) -> SchemaVariantResult<MapKeyFuncSpec> {
        let mut builder = MapKeyFuncSpec::builder();
        builder.func_unique_id(identity_func_unique_id);
        builder.key(&self.key);
        if let Some(value_from) = &self.value_from {
            builder.input(value_from.to_spec());
        };
        Ok(builder.build()?)
    }
}

/// The definition for a [`Prop`](crate::Prop) in a [`SchemaVariant`](crate::SchemaVariant).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PropDefinition {
    /// The name of the [`Prop`](crate::Prop) to be created.
    pub name: String,
    /// The [`kind`](crate::PropKind) of the [`Prop`](crate::Prop) to be created.
    pub kind: PropKind,
    /// An optional reference to a documentation link in the "doc_links" field for the
    /// [`SchemaVariantJson`] for the [`Prop`](crate::Prop) to be created.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub doc_link_ref: Option<String>,
    /// An optional documentation link for the [`Prop`](crate::Prop) to be created.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub doc_link: Option<String>,
    /// An optional set of inline documentation for the [`Prop`](crate::Prop) to be created.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub documentation: Option<String>,
    /// If our [`kind`](crate::PropKind) is [`Object`](crate::PropKind::Object), specify the
    /// child definition(s).
    #[serde(default)]
    pub children: Vec<PropDefinition>,
    /// If our [`kind`](crate::PropKind) is [`Array`](crate::PropKind::Array), specify the entry
    /// definition.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entry: Option<Box<PropDefinition>>,
    /// The [`WidgetDefinition`](crate::schema::variant::json::PropWidgetDefinition) of the
    /// [`Prop`](crate::Prop) to be created.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub widget: Option<PropWidgetDefinition>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    // The source of the information for the prop
    pub value_from: Option<ValueFrom>,
    // Whether the prop is hidden from the UI
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hidden: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validation_format: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_value: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub map_key_funcs: Option<Vec<MapKeyFunc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub suggest_sources: Option<Vec<PropSuggestion>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub suggest_as_source_for: Option<Vec<PropSuggestion>>,
}

/// Suggested property for suggestedSources and suggestAsSourceFor to link to
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PropSuggestion {
    /// The name of the schema the property is on, e.g. "AWS::EC2::VPC"
    pub schema: String,
    /// The path to the property (e.g. "/resource_value/VpcId")
    pub prop: String,
}

impl PropDefinition {
    pub fn to_spec(&self, identity_func_unique_id: &str) -> SchemaVariantResult<PropSpec> {
        let PropDefinition {
            name,
            kind,
            doc_link_ref: _, // TODO why is this not being copied
            doc_link,
            documentation,
            children,
            entry,
            widget,
            value_from,
            hidden,
            validation_format,
            default_value,
            map_key_funcs,
            suggest_sources,
            suggest_as_source_for,
        } = self;
        let mut builder = PropSpec::builder();
        builder.name(name);
        builder.kind(*kind);
        builder.has_data(true);
        if let Some(doc_url) = doc_link {
            builder.try_doc_link(doc_url.as_str())?;
        }
        if let Some(docs) = documentation {
            builder.documentation(docs);
        }
        if let Some(default_value) = &default_value {
            builder.default_value(default_value.to_owned());
        }
        match kind {
            PropKind::Array | PropKind::Map => {
                if let Some(entry) = &entry {
                    builder.type_prop(entry.to_spec(identity_func_unique_id)?);
                }
            }
            PropKind::Object => {
                for child in children {
                    builder.entry(child.to_spec(identity_func_unique_id)?);
                }
            }
            _ => {}
        }
        if let Some(widget) = widget {
            builder.widget_kind(widget.kind);
            if let Some(widget_options) = &widget.options {
                builder.widget_options(widget_options.to_owned());
            }
        }
        if let Some(value_from) = value_from {
            builder.func_unique_id(identity_func_unique_id);
            builder.input(value_from.to_spec());
        }
        if let Some(hidden) = hidden {
            builder.hidden(*hidden);
        }
        if let Some(map_key_funcs) = map_key_funcs {
            for map_key_func in map_key_funcs {
                builder.map_key_func(map_key_func.to_spec(identity_func_unique_id)?);
            }
        }
        if let Some(validation_format) = validation_format {
            builder.validation_format(validation_format);
        }
        let mut ui_optionals = HashMap::new();
        if let Some(suggest_sources) = suggest_sources {
            ui_optionals.insert(
                "suggestSources".to_string(),
                serde_json::to_value(suggest_sources)?,
            );
        }
        if let Some(suggest_as_source_for) = suggest_as_source_for {
            ui_optionals.insert(
                "suggestAsSourceFor".to_string(),
                serde_json::to_value(suggest_as_source_for)?,
            );
        }
        if !ui_optionals.is_empty() {
            builder.ui_optionals(ui_optionals);
        }

        Ok(builder.build()?)
    }
}

/// The definition for a [`Socket`](crate::Socket) in a [`SchemaVariant`](crate::SchemaVariant).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SocketDefinition {
    /// The name of the [`Socket`](crate::Socket) to be created.
    pub name: String,
    /// The type identifier of the [`Socket`](crate::Socket) to be created.
    pub connection_annotations: String,
    /// The [`arity`](https://en.wikipedia.org/wiki/Arity) of the [`Socket`](crate::Socket).
    /// Defaults to [`SocketArity::Many`](crate::SocketArity::Many) if nothing is provided.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arity: Option<SocketArity>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ui_hidden: Option<bool>,
    // The source of the information for the socket
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_from: Option<ValueFrom>,
}

impl SocketDefinition {
    pub fn to_spec(
        &self,
        is_input: bool,
        identity_func_unique_id: &str,
    ) -> SchemaVariantResult<SocketSpec> {
        let mut builder = SocketSpec::builder();
        let mut data_builder = SocketSpecData::builder();
        builder.name(&self.name);
        data_builder.name(&self.name);
        data_builder.connection_annotations(&self.connection_annotations);
        if is_input {
            data_builder.kind(SocketSpecKind::Input);
        } else {
            data_builder.kind(SocketSpecKind::Output);
        }

        if let Some(arity) = &self.arity {
            data_builder.arity(arity);
        } else {
            data_builder.arity(SocketSpecArity::Many);
        }
        if let Some(hidden) = &self.ui_hidden {
            data_builder.ui_hidden(*hidden);
        } else {
            data_builder.ui_hidden(false);
        }
        if let Some(value_from) = &self.value_from {
            data_builder.func_unique_id(identity_func_unique_id);
            builder.input(value_from.to_spec());
        }
        builder.data(data_builder.build()?);

        Ok(builder.build()?)
    }
}
