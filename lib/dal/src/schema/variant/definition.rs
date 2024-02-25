//! Create a [`SchemaVariant`](crate::SchemaVariant) with a [`Prop`](crate::Prop) tree via a
//! [`SchemaVariantDefinition`], stored in the database.
//! Do not use this struct to access or modify [`SchemaVariant`](crate::SchemaVariant) data.
//! After import, it only exists to map a [`SchemaVariant`](crate::SchemaVariant) to its
//! generating [`Func`](crate::Func). Data on this object will get outdated if the prop tree changes.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use telemetry::prelude::*;
use thiserror::Error;
use url::ParseError;

use crate::pkg::{get_component_type, PkgError};
use crate::prop::PropPath;
use crate::schema::variant::{SchemaVariantError, SchemaVariantResult};
use crate::{
    component::ComponentKind, impl_standard_model, pk, property_editor::schema::WidgetKind,
    standard_model, standard_model_accessor, Component, ComponentError, ComponentType, DalContext,
    FuncId, HistoryEventError, NatsError, PgError, PropId, PropKind, Schema, SchemaId,
    SchemaVariant, SchemaVariantId, SocketArity, StandardModel, StandardModelError, Tenancy,
    Timestamp, TransactionsError, Visibility,
};
use crate::{ChangeSetPk, WsEvent, WsEventResult, WsPayload};
use si_pkg::{
    AttrFuncInputSpec, MapKeyFuncSpec, PropSpec, SchemaSpec, SchemaSpecData, SchemaVariantSpec,
    SchemaVariantSpecData, SiPropFuncSpec, SiPropFuncSpecKind, SocketSpec, SocketSpecArity,
    SocketSpecData, SocketSpecKind, SpecError,
};

const DEFS_FOR_DEFAULT_VARIANTS: &str = include_str!("../../queries/defs_for_default_variants.sql");

#[remain::sorted]
#[derive(Error, Debug)]
pub enum SchemaVariantDefinitionError {
    #[error(transparent)]
    Component(#[from] Box<ComponentError>),
    #[error("Could not check for default variant: {0}")]
    CouldNotCheckForDefaultVariant(String),
    #[error("Could not get ui menu for schema: {0}")]
    CouldNotGetUiMenu(SchemaId),
    #[error("error decoding code_base64: {0}")]
    Decode(#[from] base64::DecodeError),
    #[error("default variant {0} not found")]
    DefaultVariantNotFound(String),
    #[error("history event error: {0}")]
    HistoryEvent(#[from] HistoryEventError),
    #[error("{0} is not a valid hex color string")]
    InvalidHexColor(String),
    #[error("schema spec has more than one variant, which we do not yet support")]
    MoreThanOneVariant,
    #[error("nats txn error: {0}")]
    Nats(#[from] NatsError),
    #[error("schema spec has no variants")]
    NoVariants,
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
    func_id: FuncId,
    description: Option<String>,
    schema_variant_id: Option<SchemaVariantId>,
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
        func_id: FuncId,
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
                    &func_id,
                    &description,
                ],
            )
            .await?;

        Ok(standard_model::finish_create_from_row(ctx, row).await?)
    }

    pub async fn list_for_default_variants(
        ctx: &DalContext,
    ) -> SchemaVariantDefinitionResult<Vec<Self>> {
        let rows = ctx
            .txns()
            .await?
            .pg()
            .query(
                DEFS_FOR_DEFAULT_VARIANTS,
                &[ctx.tenancy(), ctx.visibility()],
            )
            .await?;

        Ok(standard_model::objects_from_rows(rows)?)
    }

    pub async fn get_by_func_id(
        ctx: &DalContext,
        func_id: FuncId,
    ) -> SchemaVariantDefinitionResult<Option<Self>> {
        let row = ctx
            .txns()
            .await?
            .pg()
            .query_opt(
                "SELECT row_to_json(svd.*) AS object
                    FROM schema_variant_definitions_v1($1, $2) as svd
                    WHERE func_id = $3",
                &[ctx.tenancy(), ctx.visibility(), &func_id],
            )
            .await?;

        Ok(standard_model::object_option_from_row_option(row)?)
    }

    pub async fn get_by_schema_variant_id(
        ctx: &DalContext,
        schema_variant_id: &SchemaVariantId,
    ) -> SchemaVariantDefinitionResult<Option<Self>> {
        let row = ctx
            .txns()
            .await?
            .pg()
            .query_opt(
                "SELECT row_to_json(svd.*) AS object
                    FROM schema_variant_definitions_v1($1, $2) as svd
                    WHERE schema_variant_id = $3",
                &[ctx.tenancy(), ctx.visibility(), schema_variant_id],
            )
            .await?;

        Ok(standard_model::object_option_from_row_option(row)?)
    }

    pub async fn list_components(
        &self,
        ctx: &DalContext,
    ) -> SchemaVariantDefinitionResult<Vec<Component>> {
        Ok(if let Some(variant_id) = self.schema_variant_id {
            Component::list_for_schema_variant(ctx, variant_id)
                .await
                .map_err(Box::new)?
        } else {
            vec![]
        })
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
    standard_model_accessor!(func_id, Pk(FuncId), SchemaVariantDefinitionResult);
    standard_model_accessor!(
        schema_variant_id,
        Option<Pk(SchemaVariantId)>,
        SchemaVariantDefinitionResult
    );
    standard_model_accessor!(
        component_type,
        Enum(ComponentType),
        SchemaVariantDefinitionResult
    );
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SchemaVariantDefinitionCreatedPayload {
    schema_variant_definition_id: SchemaVariantDefinitionId,
    change_set_pk: ChangeSetPk,
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SchemaVariantDefinitionClonedPayload {
    schema_variant_definition_id: SchemaVariantDefinitionId,
    change_set_pk: ChangeSetPk,
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SchemaVariantDefinitionSavedPayload {
    schema_variant_definition_id: SchemaVariantDefinitionId,
    change_set_pk: ChangeSetPk,
}

impl WsEvent {
    pub async fn schema_variant_definition_created(
        ctx: &DalContext,
        schema_variant_definition_id: SchemaVariantDefinitionId,
    ) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::SchemaVariantDefinitionCreated(SchemaVariantDefinitionCreatedPayload {
                schema_variant_definition_id,
                change_set_pk: ctx.visibility().change_set_pk,
            }),
        )
        .await
    }

    pub async fn schema_variant_definition_cloned(
        ctx: &DalContext,
        schema_variant_definition_id: SchemaVariantDefinitionId,
    ) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::SchemaVariantDefinitionCloned(SchemaVariantDefinitionClonedPayload {
                schema_variant_definition_id,
                change_set_pk: ctx.visibility().change_set_pk,
            }),
        )
        .await
    }

    pub async fn schema_variant_definition_saved(
        ctx: &DalContext,
        schema_variant_definition_id: SchemaVariantDefinitionId,
    ) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::SchemaVariantDefinitionSaved(SchemaVariantDefinitionSavedPayload {
                schema_variant_definition_id,
                change_set_pk: ctx.visibility().change_set_pk,
            }),
        )
        .await
    }
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
        let mut data_builder = SchemaSpecData::builder();
        data_builder.name(&self.name);
        data_builder.category(&self.category);
        if let Some(menu_name) = &self.menu_name {
            data_builder.category_name(menu_name.as_str());
        }
        builder.data(data_builder.build()?);
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

impl SchemaVariantDefinitionMetadataJson {
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
                ));
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

impl SchemaVariantDefinitionJson {
    pub fn to_spec(
        &self,
        metadata: SchemaVariantDefinitionMetadataJson,
        identity_func_unique_id: &str,
        asset_func_spec_unique_id: &str,
        name: &str,
    ) -> SchemaVariantDefinitionResult<SchemaVariantSpec> {
        let mut builder = SchemaVariantSpec::builder();
        builder.name(name);

        let mut data_builder = SchemaVariantSpecData::builder();

        data_builder.name(name);
        data_builder.color(metadata.color);
        data_builder.component_type(metadata.component_type);
        if let Some(link) = metadata.link {
            data_builder.try_link(link.as_str())?;
        }

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
    ) -> SchemaVariantDefinitionResult<SchemaVariantDefinitionMetadataJson> {
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
                .ok_or(SchemaVariantDefinitionError::DefaultVariantNotFound(
                    default_variant_unique_id,
                ))?,
            None => schema_spec
                .variants
                .last()
                .ok_or(SchemaVariantDefinitionError::NoVariants)?,
        };

        let variant_spec_data =
            default_variant_spec
                .data
                .to_owned()
                .unwrap_or(SchemaVariantSpecData {
                    name: "v0".into(),
                    color: None,
                    link: None,
                    component_type: si_pkg::SchemaVariantSpecComponentType::Component,
                    func_unique_id: "0".into(),
                });

        let metadata = SchemaVariantDefinitionMetadataJson {
            name: schema_spec.name,
            menu_name: schema_data.category_name,
            category: schema_data.category,
            color: variant_spec_data
                .color
                .to_owned()
                .unwrap_or("000000".into()),
            component_kind: ComponentKind::Standard,
            component_type: variant_spec_data.component_type.into(),
            link: variant_spec_data.link.as_ref().map(|l| l.to_string()),
            description: None, // XXX - does this exist?
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
    pub fn to_spec(
        &self,
        identity_func_unique_id: &str,
    ) -> SchemaVariantDefinitionResult<MapKeyFuncSpec> {
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
    /// [`SchemaVariantDefinitionJson`] for the [`Prop`](crate::Prop) to be created.
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validation_format: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_value: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub map_key_funcs: Option<Vec<MapKeyFunc>>,
}

impl PropDefinition {
    pub fn to_spec(
        &self,
        identity_func_unique_id: &str,
    ) -> SchemaVariantDefinitionResult<PropSpec> {
        let mut builder = PropSpec::builder();
        builder.name(&self.name);
        builder.kind(self.kind);
        builder.has_data(true);
        if let Some(doc_url) = &self.doc_link {
            builder.try_doc_link(doc_url.as_str())?;
        }
        if let Some(docs) = &self.documentation {
            builder.documentation(docs);
        }
        if let Some(default_value) = &self.default_value {
            builder.default_value(default_value.to_owned());
        }
        if let Some(validation_format) = &self.validation_format {
            builder.validation_format(validation_format);
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
            builder.input(value_from.to_spec());
        }
        if let Some(hidden) = self.hidden {
            builder.hidden(hidden);
        }
        if let Some(map_key_funcs) = &self.map_key_funcs {
            for map_key_func in map_key_funcs {
                builder.map_key_func(map_key_func.to_spec(identity_func_unique_id)?);
            }
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
    ) -> SchemaVariantDefinitionResult<SocketSpec> {
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

/// The definition for the source of the information for a prop or a socket in a [`SchemaVariant`](crate::SchemaVariant).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum ValueFrom {
    InputSocket { socket_name: String },
    OutputSocket { socket_name: String },
    Prop { prop_path: Vec<String> },
}

impl ValueFrom {
    fn to_spec(&self) -> AttrFuncInputSpec {
        match self {
            ValueFrom::InputSocket { socket_name } => AttrFuncInputSpec::InputSocket {
                name: "identity".to_string(),
                socket_name: socket_name.to_owned(),
                unique_id: None,
                deleted: false,
            },
            ValueFrom::Prop { prop_path } => AttrFuncInputSpec::Prop {
                name: "identity".to_string(),
                prop_path: PropPath::new(prop_path).into(),
                unique_id: None,
                deleted: false,
            },
            ValueFrom::OutputSocket { socket_name } => AttrFuncInputSpec::OutputSocket {
                name: "identity".to_string(),
                socket_name: socket_name.to_owned(),
                unique_id: None,
                deleted: false,
            },
        }
    }
}

/// The definition for the source of the data for prop under "/root/"si" in a [`SchemaVariant`](crate::SchemaVariant).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SiPropValueFrom {
    kind: SiPropFuncSpecKind,
    value_from: ValueFrom,
}

impl SiPropValueFrom {
    fn to_spec(&self, identity_func_unique_id: &str) -> SiPropFuncSpec {
        SiPropFuncSpec {
            kind: self.kind,
            func_unique_id: identity_func_unique_id.to_owned(),
            inputs: vec![self.value_from.to_spec()],
            unique_id: None,
            deleted: false,
        }
    }
}
