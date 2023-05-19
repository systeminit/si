//! Create a [`SchemaVariant`](crate::SchemaVariant) with a [`Prop`](crate::Prop) tree via a
//! [`SchemaVariantDefinition`], stored in the database.

use async_recursion::async_recursion;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use telemetry::prelude::*;
use thiserror::Error;
use url::ParseError;

use crate::pkg::{get_component_type, PkgError};
use crate::prop::PROP_PATH_SEPARATOR;
use crate::schema::variant::{SchemaVariantError, SchemaVariantResult};
use crate::{
    component::ComponentKind, impl_standard_model, pk, property_editor::schema::WidgetKind,
    standard_model, standard_model_accessor, ComponentType, DalContext, ExternalProvider, Func,
    HistoryEventError, InternalProvider, NatsError, PgError, Prop, PropId, PropKind, RootProp,
    Schema, SchemaVariant, SchemaVariantId, SocketArity, StandardModel, StandardModelError,
    Tenancy, Timestamp, Visibility,
};
use crate::{SchemaError, SchemaId, TransactionsError};
use si_pkg::{
    AttrFuncInputSpec, FuncUniqueId, MapKeyFuncSpec, PropSpec, SchemaSpec, SchemaVariantSpec,
    SocketSpec, SocketSpecArity, SocketSpecKind, SpecError, ValidationSpec,
};

#[remain::sorted]
#[derive(Error, Debug)]
pub enum SchemaVariantDefinitionError {
    #[error("Could not check for default variant: {0}")]
    CouldNotCheckForDefaultVariant(String),
    #[error("Could not get ui menu for schema: {0}")]
    CouldNotGetUiMenu(SchemaId),
    #[error("error decoding code_base64: {0}")]
    Decode(#[from] base64::DecodeError),
    #[error("history event error: {0}")]
    HistoryEvent(#[from] HistoryEventError),
    #[error("{0} is not a valid hex color string")]
    InvalidHexColor(String),
    #[error("nats txn error: {0}")]
    Nats(#[from] NatsError),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("pkg error: {0}")]
    Pkg(#[from] Box<PkgError>),
    #[error(transparent)]
    SchemaVariant(#[from] Box<SchemaVariantError>),
    #[error("error serializing/deserializing json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("spec error: {0}")]
    Spec(#[from] SpecError),
    #[error("standard model error: {0}")]
    StandardModelError(#[from] StandardModelError),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error("url parse error: {0}")]
    Url(#[from] ParseError),
}

pub type SchemaVariantDefinitionResult<T> = Result<T, SchemaVariantDefinitionError>;

/// A cache of [`PropIds`](crate::Prop) where the _key_ is a tuple corresponding to the
/// [`Prop`](crate::Prop) name and the _parent_ [`PropId`](crate::Prop) who's child is the
/// [`PropId`](crate::Prop) in the _value_ of the entry.
///
/// It is recommended to start with the [`RootProp`](crate::RootProp) in order to descend into the
/// cache.
#[derive(Debug, Clone)]
pub struct PropCache(HashMap<(String, PropId), PropId>);

impl PropCache {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    /// Attempts to retrieve the [`PropId`](crate::Prop) value for a given [`Prop`](crate::Prop)
    /// name and parent [`PropId`](crate::Prop) key tuple. An error is returned if nothing is found.
    pub fn get(
        &self,
        prop_name: impl AsRef<str>,
        parent_prop_id: PropId,
    ) -> SchemaVariantResult<PropId> {
        // NOTE(nick): the string handling could probably be better here.
        let prop_name = prop_name.as_ref().to_string();
        let prop_id = *self.0.get(&(prop_name.clone(), parent_prop_id)).ok_or(
            SchemaVariantError::PropNotFoundInCache(prop_name, parent_prop_id),
        )?;
        Ok(prop_id)
    }

    /// Insert the [`PropId`](crate::Prop) into [`self`](Self). The returned `option` from the
    /// underlying method is ignored.
    pub fn insert(&mut self, key: (String, PropId), value: PropId) {
        self.0.insert(key, value);
    }
}

impl Default for PropCache {
    fn default() -> Self {
        Self::new()
    }
}

pk!(SchemaVariantDefinitionPk);
pk!(SchemaVariantDefinitionId);

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct SchemaVariantDefinition {
    pk: SchemaVariantDefinitionPk,
    id: SchemaVariantDefinitionId,
    #[serde(flatten)]
    tenancy: Tenancy,
    #[serde(flatten)]
    timestamp: Timestamp,
    #[serde(flatten)]
    visibility: Visibility,

    /// Name for this variant. Actually, this is the name for this [`Schema`](crate::Schema), we're
    /// punting on the issue of multiple variants for the moment.
    name: String,
    /// Override for the UI name for this schema
    menu_name: Option<String>,
    /// The category this schema variant belongs to
    category: String,
    /// The color for the component on the component diagram as a hex string
    color: String,
    component_kind: ComponentKind,
    component_type: ComponentType,
    link: Option<String>,
    definition: String,
    description: Option<String>,
}

impl_standard_model! {
    model: SchemaVariantDefinition,
    pk: SchemaVariantDefinitionPk,
    id: SchemaVariantDefinitionId,
    table_name: "schema_variant_definitions",
    history_event_label_base: "schema_variant_definition",
    history_event_message_name: "Schema Variant Definition",
}

impl SchemaVariantDefinition {
    #[instrument(skip_all)]
    pub async fn new_from_structs(
        ctx: &DalContext,
        metadata: SchemaVariantDefinitionMetadataJson,
        definition: SchemaVariantDefinitionJson,
    ) -> SchemaVariantDefinitionResult<SchemaVariantDefinition> {
        SchemaVariantDefinition::new(
            ctx,
            metadata.name,
            metadata.menu_name,
            metadata.category,
            metadata.link,
            metadata.color,
            metadata.component_kind,
            metadata.description,
            serde_json::to_string_pretty(&definition)?,
        )
        .await
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn new(
        ctx: &DalContext,
        name: String,
        menu_name: Option<String>,
        category: String,
        link: Option<String>,
        color: String,
        component_kind: ComponentKind,
        description: Option<String>,
        definition: String,
    ) -> SchemaVariantDefinitionResult<SchemaVariantDefinition> {
        let row = ctx
            .txns()
            .await?
            .pg()
            .query_one(
                "SELECT object FROM schema_variant_definition_create_v1(
                    $1,
                    $2,
                    $3,
                    $4,
                    $5,
                    $6,
                    $7,
                    $8,
                    $9,
                    $10
                )",
                &[
                    ctx.tenancy(),
                    ctx.visibility(),
                    &name,
                    &menu_name,
                    &category,
                    &link,
                    &color,
                    &component_kind.as_ref(),
                    &definition,
                    &description,
                ],
            )
            .await?;

        Ok(standard_model::finish_create_from_row(ctx, row).await?)
    }

    pub async fn existing_default_schema_variant_id(
        &self,
        ctx: &DalContext,
    ) -> SchemaVariantDefinitionResult<Option<SchemaVariantId>> {
        Ok(
            match Schema::default_schema_variant_id_for_name(ctx, self.name()).await {
                Ok(schema_variant_id) => Some(schema_variant_id),
                Err(SchemaError::NotFoundByName(_)) | Err(SchemaError::NoDefaultVariant(_)) => None,
                Err(e) => {
                    return Err(
                        SchemaVariantDefinitionError::CouldNotCheckForDefaultVariant(e.to_string()),
                    );
                }
            },
        )
    }

    standard_model_accessor!(name, String, SchemaVariantDefinitionResult);
    standard_model_accessor!(menu_name, Option<String>, SchemaVariantDefinitionResult);
    standard_model_accessor!(category, String, SchemaVariantDefinitionResult);
    standard_model_accessor!(color, String, SchemaVariantDefinitionResult);
    standard_model_accessor!(
        component_kind,
        Enum(ComponentKind),
        SchemaVariantDefinitionResult
    );
    standard_model_accessor!(link, Option<String>, SchemaVariantDefinitionResult);
    standard_model_accessor!(description, Option<String>, SchemaVariantDefinitionResult);
    standard_model_accessor!(definition, String, SchemaVariantDefinitionResult);
    standard_model_accessor!(
        component_type,
        Enum(ComponentType),
        SchemaVariantDefinitionResult
    );
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SchemaVariantDefinitionMetadataJson {
    /// Name for this variant. Actually, this is the name for this [`Schema`](crate::Schema), we're
    /// punting on the issue of multiple variants for the moment.
    pub name: String,
    /// Override for the UI name for this schema
    #[serde(alias = "menu_name")]
    pub menu_name: Option<String>,
    /// The category this schema variant belongs to
    pub category: String,
    /// The color for the component on the component diagram as a hex string
    pub color: String,
    #[serde(alias = "component_kind")]
    pub component_kind: ComponentKind,
    #[serde(alias = "component_type")]
    pub component_type: ComponentType,
    pub link: Option<String>,
    pub description: Option<String>,
}

impl SchemaVariantDefinitionMetadataJson {
    pub fn to_spec(&self, variant: SchemaVariantSpec) -> SchemaVariantDefinitionResult<SchemaSpec> {
        let mut builder = SchemaSpec::builder();
        builder.name(&self.name);
        builder.category(&self.category);
        if let Some(menu_name) = &self.menu_name {
            builder.category_name(menu_name.as_str());
        }
        builder.variant(variant);

        Ok(builder.build()?)
    }
}

impl From<SchemaVariantDefinition> for SchemaVariantDefinitionMetadataJson {
    fn from(value: SchemaVariantDefinition) -> Self {
        SchemaVariantDefinitionMetadataJson {
            name: value.name,
            menu_name: value.menu_name,
            category: value.category,
            color: value.color,
            component_kind: value.component_kind,
            component_type: value.component_type,
            link: value.link,
            description: value.description,
        }
    }
}

impl From<&SchemaVariantDefinition> for SchemaVariantDefinitionMetadataJson {
    fn from(value: &SchemaVariantDefinition) -> Self {
        SchemaVariantDefinitionMetadataJson {
            name: value.name.clone(),
            menu_name: value.menu_name.clone(),
            category: value.category.clone(),
            color: value.color.clone(),
            component_kind: value.component_kind,
            component_type: value.component_type,
            link: value.link.clone(),
            description: value.description.clone(),
        }
    }
}

impl SchemaVariantDefinitionMetadataJson {
    #[instrument(skip_all)]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        name: &str,
        menu_name: Option<&str>,
        category: &str,
        color: &str,
        component_kind: ComponentKind,
        link: Option<&str>,
        description: Option<&str>,
        component_type: ComponentType,
    ) -> SchemaVariantDefinitionMetadataJson {
        SchemaVariantDefinitionMetadataJson {
            name: name.to_string(),
            menu_name: menu_name.map(|s| s.to_string()),
            category: category.to_string(),
            color: color.to_string(),
            component_kind,
            component_type,
            link: link.map(|l| l.to_string()),
            description: description.map(|d| d.to_string()),
        }
    }

    pub async fn from_schema_and_variant(
        ctx: &DalContext,
        schema: &Schema,
        variant: &SchemaVariant,
    ) -> SchemaVariantDefinitionResult<Self> {
        let (menu_name, category) = match schema.ui_menus(ctx).await {
            Ok(ui_menus) => match ui_menus.get(0) {
                Some(ui_menu) => (
                    Some(ui_menu.name().to_string()),
                    ui_menu.category().to_string(),
                ),
                None => (None, "".to_string()),
            },
            Err(_) => {
                return Err(SchemaVariantDefinitionError::CouldNotGetUiMenu(
                    *schema.id(),
                ))
            }
        };

        Ok(SchemaVariantDefinitionMetadataJson {
            name: schema.name().to_string(),
            menu_name,
            category,
            color: variant
                .color(ctx)
                .await
                .map_err(Box::new)?
                .unwrap_or_else(|| "baddad".to_string()),
            component_kind: *schema.component_kind(),
            link: variant.link().map(|l| l.to_string()),
            description: None,
            component_type: get_component_type(ctx, variant)
                .await
                .map_err(Box::new)?
                .into(),
        })
    }
}

/// The definition for a [`SchemaVariant`](crate::SchemaVariant)'s [`Prop`](crate::Prop) tree (and
/// more in the future).
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SchemaVariantDefinitionJson {
    /// The immediate child [`Props`](crate::Prop) underneath "/root/domain".
    #[serde(default)]
    pub props: Vec<PropDefinition>,
    /// The input [`Sockets`](crate::Socket) and corresponding
    /// explicit [`InternalProviders`](crate::InternalProvider) created for the
    /// [`variant`](crate::SchemaVariant).
    #[serde(default)]
    pub input_sockets: Vec<SocketDefinition>,
    /// The output [`Sockets`](crate::Socket) and corresponding
    /// [`ExternalProviders`](crate::ExternalProvider) created for the
    /// [`variant`](crate::SchemaVariant).
    #[serde(default)]
    pub output_sockets: Vec<SocketDefinition>,
    /// A map of documentation links to reference. To reference links (values) specify the key via
    /// the "doc_link_ref" field for a [`PropDefinition`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub doc_links: Option<HashMap<String, String>>,
}

impl TryFrom<SchemaVariantDefinition> for SchemaVariantDefinitionJson {
    type Error = SchemaVariantDefinitionError;

    fn try_from(value: SchemaVariantDefinition) -> Result<Self, Self::Error> {
        Ok(serde_json::from_str(&value.definition)?)
    }
}

impl TryFrom<&SchemaVariantDefinition> for SchemaVariantDefinitionJson {
    type Error = SchemaVariantDefinitionError;

    fn try_from(value: &SchemaVariantDefinition) -> Result<Self, Self::Error> {
        Ok(serde_json::from_str(&value.definition)?)
    }
}

impl SchemaVariantDefinitionJson {
    pub fn to_spec(
        &self,
        metadata: SchemaVariantDefinitionMetadataJson,
        identity_func_unique_id: FuncUniqueId,
    ) -> SchemaVariantDefinitionResult<SchemaVariantSpec> {
        let mut builder = SchemaVariantSpec::builder();
        builder.name("v0");
        for prop in &self.props {
            builder.domain_prop(prop.to_spec(identity_func_unique_id)?);
        }
        builder.color(metadata.color);
        builder.component_type(metadata.component_type);
        if let Some(link) = metadata.link {
            builder.try_link(link.as_str())?;
        }
        for input_socket in &self.input_sockets {
            builder.socket(input_socket.to_spec(true)?);
        }
        for output_socket in &self.output_sockets {
            builder.socket(output_socket.to_spec(false)?);
        }

        Ok(builder.build()?)
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
    pub fn to_spec(
        &self,
        identity_func_unique_id: FuncUniqueId,
    ) -> SchemaVariantDefinitionResult<MapKeyFuncSpec> {
        let mut builder = MapKeyFuncSpec::builder();
        builder.func_unique_id(identity_func_unique_id);
        builder.key(&self.key);
        if let Some(value_from) = &self.value_from {
            match value_from {
                ValueFrom::InputSocket { socket_name } => {
                    builder.input(AttrFuncInputSpec::InputSocket {
                        name: "identity".to_string(),
                        socket_name: socket_name.to_owned(),
                    });
                }
                ValueFrom::Prop { prop_path } => {
                    builder.input(AttrFuncInputSpec::Prop {
                        name: "identity".to_string(),
                        prop_path: prop_path.join(PROP_PATH_SEPARATOR),
                    });
                }
                ValueFrom::OutputSocket { socket_name } => {
                    builder.input(AttrFuncInputSpec::OutputSocket {
                        name: "identity".to_string(),
                        socket_name: socket_name.to_owned(),
                    });
                }
            }
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
    /// [`SchemaVariantDefinitionJson`] for the [`Prop`](crate::Prop) to be created.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub doc_link_ref: Option<String>,
    /// An optional documentation link for the [`Prop`](crate::Prop) to be created.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub doc_link: Option<String>,
    /// If our [`kind`](crate::PropKind) is [`Object`](crate::PropKind::Object), specify the
    /// child definition(s).
    #[serde(default)]
    pub children: Vec<PropDefinition>,
    /// If our [`kind`](crate::PropKind) is [`Array`](crate::PropKind::Array), specify the entry
    /// definition.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entry: Option<Box<PropDefinition>>,
    /// The [`WidgetDefinition`](crate::schema::variant::definition::PropWidgetDefinition) of the
    /// [`Prop`](crate::Prop) to be created.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub widget: Option<PropWidgetDefinition>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    // The source of the information for the prop
    pub value_from: Option<ValueFrom>,
    // Whether the prop is hidden from the UI
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hidden: Option<bool>,
    // The list of validations specific to the prop.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validations: Option<Vec<ValidationSpec>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_value: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub map_key_funcs: Option<MapKeyFunc>,
}

impl PropDefinition {
    pub fn to_spec(
        &self,
        identity_func_unique_id: FuncUniqueId,
    ) -> SchemaVariantDefinitionResult<PropSpec> {
        let mut builder = PropSpec::builder();
        builder.name(&self.name);
        builder.kind(self.kind.into());
        if let Some(doc_url) = &self.doc_link {
            builder.try_doc_link(doc_url.as_str())?;
        }
        if let Some(default_value) = &self.default_value {
            builder.default_value(default_value.to_owned());
        }
        if let Some(validations) = &self.validations {
            for validation in validations {
                builder.validation(validation.to_owned());
            }
        }
        match self.kind {
            PropKind::Array | PropKind::Map => {
                if let Some(entry) = &self.entry {
                    builder.type_prop(entry.to_spec(identity_func_unique_id)?);
                }
            }
            PropKind::Object => {
                for child in &self.children {
                    builder.entry(child.to_spec(identity_func_unique_id)?);
                }
            }
            _ => {}
        }
        if let Some(widget) = &self.widget {
            builder.widget_kind(widget.kind);
            if let Some(widget_options) = &widget.options {
                builder.widget_options(widget_options.to_owned());
            }
        }
        if let Some(value_from) = &self.value_from {
            builder.func_unique_id(identity_func_unique_id);
            match value_from {
                ValueFrom::InputSocket { socket_name } => {
                    builder.input(AttrFuncInputSpec::InputSocket {
                        name: "identity".to_string(),
                        socket_name: socket_name.to_owned(),
                    });
                }
                ValueFrom::Prop { prop_path } => {
                    builder.input(AttrFuncInputSpec::Prop {
                        name: "identity".to_string(),
                        prop_path: prop_path.join(PROP_PATH_SEPARATOR),
                    });
                }
                ValueFrom::OutputSocket { socket_name } => {
                    builder.input(AttrFuncInputSpec::OutputSocket {
                        name: "identity".to_string(),
                        socket_name: socket_name.to_owned(),
                    });
                }
            }
        }
        if let Some(hidden) = self.hidden {
            builder.hidden(hidden);
        }
        if let Some(map_key_funcs) = &self.map_key_funcs {
            builder.map_key_func(map_key_funcs.to_spec(identity_func_unique_id)?);
        }

        Ok(builder.build()?)
    }
}

/// The definition for a [`Socket`](crate::Socket) in a [`SchemaVariant`](crate::SchemaVariant).
/// A corresponding [`provider`](crate::provider) will be created as well.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SocketDefinition {
    /// The name of the [`Socket`](crate::Socket) to be created.
    pub name: String,
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
    pub fn to_spec(&self, is_input: bool) -> SchemaVariantDefinitionResult<SocketSpec> {
        let mut builder = SocketSpec::builder();
        builder.name(&self.name);
        if is_input {
            builder.kind(SocketSpecKind::Input);
        } else {
            builder.kind(SocketSpecKind::Output);
        }

        if let Some(arity) = &self.arity {
            builder.arity(arity);
        } else {
            builder.arity(SocketSpecArity::Many);
        }
        if let Some(hidden) = &self.ui_hidden {
            builder.ui_hidden(*hidden);
        } else {
            builder.ui_hidden(false);
        }
        if let Some(value_from) = &self.value_from {
            match value_from {
                ValueFrom::InputSocket { socket_name } => {
                    builder.input(AttrFuncInputSpec::InputSocket {
                        name: "identity".to_string(),
                        socket_name: socket_name.to_owned(),
                    });
                }
                ValueFrom::Prop { prop_path } => {
                    builder.input(AttrFuncInputSpec::Prop {
                        name: "identity".to_string(),
                        prop_path: prop_path.join(PROP_PATH_SEPARATOR),
                    });
                }
                ValueFrom::OutputSocket { socket_name } => {
                    builder.input(AttrFuncInputSpec::OutputSocket {
                        name: "identity".to_string(),
                        socket_name: socket_name.to_owned(),
                    });
                }
            }
        }

        Ok(builder.build()?)
    }
}

/// The definition for the source of the information for a prop or a socket in a [`SchemaVariant`](crate::SchemaVariant).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum ValueFrom {
    InputSocket { socket_name: String },
    OutputSocket { socket_name: String },
    Prop { prop_path: Vec<String> },
}

// Not sure if this fits here still
impl SchemaVariant {
    /// Create a [`SchemaVariant`] like [`usual`](Self::new()), but use the
    /// [`SchemaVariantDefinition`] to create a [`Prop`](crate::Prop) tree as well with a
    /// [`cache`](PropCache).
    pub async fn new_with_definition(
        ctx: &DalContext,
        schema_variant_definition_metadata: SchemaVariantDefinitionMetadataJson,
        schema_variant_definition: SchemaVariantDefinitionJson,
        variant_name: &str,
    ) -> SchemaVariantResult<(
        Self,
        RootProp,
        PropCache,
        Vec<InternalProvider>,
        Vec<ExternalProvider>,
    )> {
        let schema_name = schema_variant_definition_metadata.name.clone();

        let schema_id = match Schema::find_by_name(ctx, &schema_name).await {
            Ok(schema) => *schema.id(),
            Err(SchemaError::NotFoundByName(_)) => {
                let schema = Schema::new(ctx, &schema_name, &ComponentKind::Standard)
                    .await
                    .map_err(Box::new)?;
                *schema.id()
            }
            Err(e) => Err(Box::new(e))?,
        };

        let (schema_variant, root_prop) = Self::new(ctx, schema_id, variant_name).await?;
        let schema_variant_id = *schema_variant.id();

        // NOTE(nick): allow users to use a definition without props... just in case, I guess.
        let mut prop_cache = PropCache::new();
        let doc_links = schema_variant_definition
            .doc_links
            .clone()
            .unwrap_or_default();
        for prop_definition in schema_variant_definition.props {
            Self::walk_definition(
                ctx,
                &mut prop_cache,
                prop_definition,
                root_prop.domain_prop_id,
                &doc_links,
                schema_variant_id,
            )
            .await?;
        }

        // Only find the identity func if we have sockets to create.
        // FIXME(nick,wendy): allow other funcs to be specified in the definition manifest(s).
        let mut explicit_internal_providers = Vec::new();
        let mut external_providers = Vec::new();

        if !schema_variant_definition.input_sockets.is_empty()
            || !schema_variant_definition.output_sockets.is_empty()
        {
            let (identity_func, identity_func_binding, identity_func_binding_return_value) =
                Func::identity_with_binding_and_return_value(ctx).await?;
            let identity_func_id = *identity_func.id();
            let identity_func_binding_id = *identity_func_binding.id();
            let identity_func_binding_return_value_id = *identity_func_binding_return_value.id();

            for input_socket_definition in schema_variant_definition.input_sockets {
                let arity = match input_socket_definition.arity {
                    Some(found_arity) => found_arity,
                    None => SocketArity::Many,
                };
                let (explicit_internal_provider, _) = InternalProvider::new_explicit_with_socket(
                    ctx,
                    schema_variant_id,
                    input_socket_definition.name,
                    identity_func_id,
                    identity_func_binding_id,
                    identity_func_binding_return_value_id,
                    arity,
                    false,
                )
                .await?;
                explicit_internal_providers.push(explicit_internal_provider);
            }

            for output_socket_definition in schema_variant_definition.output_sockets {
                let arity = match output_socket_definition.arity {
                    Some(found_arity) => found_arity,
                    None => SocketArity::Many,
                };
                let (external_provider, _) = ExternalProvider::new_with_socket(
                    ctx,
                    schema_id,
                    schema_variant_id,
                    output_socket_definition.name,
                    None,
                    identity_func_id,
                    identity_func_binding_id,
                    identity_func_binding_return_value_id,
                    arity,
                    false,
                )
                .await?;
                external_providers.push(external_provider);
            }
        }

        schema_variant
            .set_color(ctx, schema_variant_definition_metadata.color)
            .await?;

        Ok((
            schema_variant,
            root_prop,
            prop_cache,
            explicit_internal_providers,
            external_providers,
        ))
    }

    /// A recursive walk of [`PropDefinition`] that populates the [`cache`](PropCache) as each
    /// [`Prop`](crate::Prop) is created.
    #[async_recursion]
    async fn walk_definition(
        ctx: &DalContext,
        prop_cache: &mut PropCache,
        definition: PropDefinition,
        parent_prop_id: PropId,
        doc_links: &HashMap<String, String>,
        schema_variant_id: SchemaVariantId,
    ) -> SchemaVariantResult<()> {
        // Start by creating the prop and setting the parent. We cache the id for later.
        let widget = match definition.widget {
            Some(widget) => Some((widget.kind, widget.options)),
            None => None,
        };
        let mut prop = Prop::new(
            ctx,
            definition.name.clone(),
            definition.kind,
            widget,
            schema_variant_id,
            Some(parent_prop_id),
        )
        .await?;
        let prop_id = *prop.id();

        // Always cache the prop that was created.
        prop_cache.insert((prop.name().to_string(), parent_prop_id), prop_id);

        // Either use the doc link or the doc link ref. Do not use both.
        match (definition.doc_link.is_some(), definition.doc_link_ref) {
            (true, Some(_)) => {
                return Err(SchemaVariantError::MultipleDocLinksProvided(
                    definition.name.clone(),
                ));
            }
            (true, None) => prop.set_doc_link(ctx, definition.doc_link).await?,
            (false, Some(doc_link_ref)) => match doc_links.get(&doc_link_ref) {
                Some(link) => prop.set_doc_link(ctx, Some(link)).await?,
                None => return Err(SchemaVariantError::LinkNotFoundForDocLinkRef(doc_link_ref)),
            },
            (false, None) => {}
        }

        // Determine if we need to descend and check the "entry" and "children" fields accordingly.
        match definition.kind {
            PropKind::Object => {
                if definition.entry.is_some() {
                    return Err(SchemaVariantError::FoundEntryForObject(
                        definition.name.clone(),
                    ));
                }
                if definition.children.is_empty() {
                    return Err(SchemaVariantError::MissingChildrenForObject(
                        definition.name.clone(),
                    ));
                }
                for child in definition.children {
                    Self::walk_definition(ctx, prop_cache, child, prop_id, doc_links, schema_variant_id).await?;
                }
            }
            PropKind::Array => match definition.entry {
                Some(entry) => {
                    if !definition.children.is_empty() {
                        return Err(SchemaVariantError::FoundChildrenForArray(
                            definition.name.clone(),
                        ));
                    }
                    Self::walk_definition(ctx, prop_cache, *entry, prop_id, doc_links, schema_variant_id).await?;
                }
                None => {
                    return Err(SchemaVariantError::MissingEntryForArray(
                        definition.name.clone(),
                    ));
                }
            },
            PropKind::Map => todo!("maps not yet implemented simply because nick didn't need them yet and didn't want an untested solution"),
            _ => match (definition.entry.is_none(), definition.children.is_empty()) {
                (false, false) => {
                    return Err(SchemaVariantError::FoundChildrenAndEntryForPrimitive(
                        definition.name.clone(),
                    ));
                }
                (false, true) => {
                    return Err(SchemaVariantError::FoundEntryForPrimitive(
                        definition.name.clone(),
                    ));
                }
                (true, false) => {
                    return Err(SchemaVariantError::FoundChildrenForPrimitive(
                        definition.name.clone(),
                    ));
                }
                (true, true) => {}
            },
        }

        Ok(())
    }
}
