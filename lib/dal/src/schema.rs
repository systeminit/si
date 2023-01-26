use serde::{Deserialize, Serialize};
use si_data_nats::NatsError;
use si_data_pg::PgError;
use telemetry::prelude::*;
use thiserror::Error;

use crate::func::binding_return_value::FuncBindingReturnValueError;
use crate::provider::external::ExternalProviderError;
use crate::provider::internal::InternalProviderError;
use crate::schema::variant::SchemaVariantError;
use crate::socket::SocketError;
use crate::WriteTenancy;
use crate::{
    component::ComponentKind, func::binding::FuncBindingError, impl_standard_model, pk,
    schema::ui_menu::SchemaUiMenuId, standard_model, standard_model_accessor,
    standard_model_has_many, standard_model_many_to_many, AttributeContextBuilderError,
    AttributePrototypeError, AttributeValueError, Component, DalContext, FuncError,
    HistoryEventError, PropError, ReadTenancyError, StandardModel, StandardModelError, Timestamp,
    ValidationPrototypeError, Visibility, Workspace, WorkspaceId, WsEventError,
};

pub use ui_menu::SchemaUiMenu;
pub use variant::root_prop::RootProp;
pub use variant::{SchemaVariant, SchemaVariantId};

pub mod ui_menu;
pub mod variant;

#[derive(Error, Debug)]
pub enum SchemaError {
    #[error("AttributeContextBuilder error: {0}")]
    AttributeContextBuilder(#[from] AttributeContextBuilderError),
    #[error("AttributePrototype error: {0}")]
    AttributePrototype(#[from] AttributePrototypeError),
    #[error("AttributeValue error: {0}")]
    AttributeValue(#[from] AttributeValueError),
    #[error("external provider error: {0}")]
    ExternalProvider(#[from] ExternalProviderError),
    #[error("func error: {0}")]
    Func(#[from] FuncError),
    #[error("func binding error: {0}")]
    FuncBinding(#[from] FuncBindingError),
    #[error("func binding return value error: {0}")]
    FuncBindingReturnValue(#[from] FuncBindingReturnValueError),
    #[error("func not found: {0}")]
    FuncNotFound(String),
    #[error("history event error: {0}")]
    HistoryEvent(#[from] HistoryEventError),
    #[error("internal provider error: {0}")]
    InternalProvider(#[from] InternalProviderError),
    #[error("missing a func in attribute update: {0} not found")]
    MissingFunc(String),
    #[error("nats txn error: {0}")]
    Nats(#[from] NatsError),
    #[error("no default variant for schema id: {0}")]
    NoDefaultVariant(SchemaId),
    #[error("schema not found: {0}")]
    NotFound(SchemaId),
    #[error("schema not found by name: {0}")]
    NotFoundByName(String),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("prop error: {0}")]
    Prop(#[from] PropError),
    #[error("read tenancy error: {0}")]
    ReadTenancy(#[from] ReadTenancyError),
    #[error("error serializing/deserializing json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("socket error: {0}")]
    Socket(#[from] SocketError),
    #[error("standard model error: {0}")]
    StandardModel(#[from] StandardModelError),
    #[error("schema ui menu not found: {0}")]
    SchemaUiMenuNotFound(SchemaUiMenuId),
    #[error("schema variant error: {0}")]
    Variant(#[from] SchemaVariantError),
    #[error("validation prototype error: {0}")]
    ValidationPrototype(#[from] ValidationPrototypeError),
    #[error("ws event error: {0}")]
    WsEvent(#[from] WsEventError),
}

pub type SchemaResult<T> = Result<T, SchemaError>;

pk!(SchemaPk);
pk!(SchemaId);

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Schema {
    pk: SchemaPk,
    id: SchemaId,
    name: String,
    #[serde(flatten)]
    tenancy: WriteTenancy,
    #[serde(flatten)]
    timestamp: Timestamp,
    #[serde(flatten)]
    visibility: Visibility,
    ui_hidden: bool,
    default_schema_variant_id: Option<SchemaVariantId>,
    component_kind: ComponentKind,
}

impl_standard_model! {
    model: Schema,
    pk: SchemaPk,
    id: SchemaId,
    table_name: "schemas",
    history_event_label_base: "schema",
    history_event_message_name: "Schema"
}

impl Schema {
    #[instrument(skip_all)]
    #[allow(clippy::too_many_arguments)]
    pub async fn new(
        ctx: &DalContext,
        name: impl AsRef<str>,
        component_kind: &ComponentKind,
    ) -> SchemaResult<Self> {
        let name = name.as_ref();
        let row = ctx
            .txns()
            .pg()
            .query_one(
                "SELECT object FROM schema_create_v1($1, $2, $3, $4)",
                &[
                    ctx.write_tenancy(),
                    ctx.visibility(),
                    &name,
                    &component_kind.as_ref(),
                ],
            )
            .await?;
        let object = standard_model::finish_create_from_row(ctx, row).await?;
        Ok(object)
    }

    standard_model_accessor!(name, String, SchemaResult);
    standard_model_accessor!(component_kind, Enum(ComponentKind), SchemaResult);
    standard_model_accessor!(ui_hidden, bool, SchemaResult);
    standard_model_accessor!(
        default_schema_variant_id,
        Option<Pk(SchemaVariantId)>,
        SchemaResult
    );

    standard_model_many_to_many!(
        lookup_fn: workspaces,
        associate_fn: add_workspace,
        disassociate_fn: remove_workspace,
        disassociate_all_fn: remove_all_workspaces,
        table_name: "schema_many_to_many_workspace",
        left_table: "schemas",
        left_id: SchemaId,
        right_table: "workspaces",
        right_id: WorkspaceId,
        which_table_is_this: "left",
        returns: Workspace,
        result: SchemaResult,
    );

    standard_model_has_many!(
        lookup_fn: ui_menus,
        table: "schema_ui_menu_belongs_to_schema",
        model_table: "schema_ui_menus",
        returns: SchemaUiMenu,
        result: SchemaResult,
    );

    standard_model_has_many!(
        lookup_fn: components,
        table: "component_belongs_to_schema",
        model_table: "components",
        returns: Component,
        result: SchemaResult,
    );

    standard_model_has_many!(
        lookup_fn: variants,
        table: "schema_variant_belongs_to_schema",
        model_table: "schema_variants",
        returns: SchemaVariant,
        result: SchemaResult,
    );

    standard_model_many_to_many!(
        lookup_fn: implements,
        associate_fn: add_implements_schema,
        disassociate_fn: remove_implements_schema,
        disassociate_all_fn: remove_all_implements_schemas,
        table_name: "schema_many_to_many_implements",
        left_table: "schemas",
        left_id: SchemaId,
        right_table: "schemas",
        right_id: SchemaId,
        which_table_is_this: "left",
        returns: Schema,
        result: SchemaResult,
    );

    pub async fn default_variant(&self, ctx: &DalContext) -> SchemaResult<SchemaVariant> {
        match self.default_schema_variant_id() {
            Some(schema_variant_id) => Ok(SchemaVariant::get_by_id(ctx, schema_variant_id)
                .await?
                .ok_or_else(|| SchemaError::NoDefaultVariant(*self.id()))?),
            None => Err(SchemaError::NoDefaultVariant(*self.id())),
        }
    }

    pub async fn default_schema_variant_id_for_name(
        ctx: &DalContext,
        name: impl AsRef<str>,
    ) -> SchemaResult<SchemaVariantId> {
        let name = name.as_ref();
        let schemas = Schema::find_by_attr(ctx, "name", &name).await?;
        let schema = schemas
            .first()
            .ok_or_else(|| SchemaError::NotFoundByName(name.into()))?;
        let schema_variant_id = schema
            .default_schema_variant_id()
            .ok_or_else(|| SchemaError::NoDefaultVariant(*schema.id()))?;

        Ok(*schema_variant_id)
    }
}
