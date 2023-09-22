use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use strum::IntoEnumIterator;
use thiserror::Error;

use dal::{
    installed_pkg::InstalledPkgError,
    pkg::PkgError,
    schema::variant::definition::SchemaVariantDefinition,
    schema::variant::definition::{
        SchemaVariantDefinitionError as DalSchemaVariantDefinitionError, SchemaVariantDefinitionId,
    },
    ActionPrototype, ActionPrototypeContext, ActionPrototypeError, AttributePrototype,
    AttributePrototypeError, DalContext, ExternalProvider, ExternalProviderError, Func,
    FuncBackendKind, FuncBackendResponseType, FuncBindingError, FuncError, FuncId,
    InternalProvider, InternalProviderError, LeafInputLocation, LeafKind, SchemaError,
    SchemaVariant, SchemaVariantError, SchemaVariantId, StandardModel, StandardModelError,
    TenancyError, TransactionsError, UserError, ValidationPrototype, ValidationPrototypeError,
    WsEventError,
};
use si_pkg::{SiPkgError, SpecError};

use crate::server::state::AppState;
use crate::service::func::FuncError as SdfFuncError;

use self::save_variant_def::SaveVariantDefRequest;

use super::func::get_leaf_function_inputs;

pub mod clone_variant_def;
pub mod create_variant_def;
pub mod exec_variant_def;
pub mod get_variant_def;
pub mod list_variant_defs;
pub mod save_variant_def;

#[remain::sorted]
#[derive(Error, Debug)]
pub enum SchemaVariantDefinitionError {
    #[error(transparent)]
    ActionPrototype(#[from] ActionPrototypeError),
    #[error(transparent)]
    AttributePrototype(#[from] AttributePrototypeError),
    #[error(transparent)]
    ContextTransaction(#[from] TransactionsError),
    #[error("error creating schema variant from definition: {0}")]
    CouldNotCreateSchemaVariantFromDefinition(String),
    #[error(transparent)]
    ExternalProvider(#[from] ExternalProviderError),
    #[error("func error: {0}")]
    Func(#[from] FuncError),
    #[error(transparent)]
    FuncBinding(#[from] FuncBindingError),
    #[error("func execution error: {0}")]
    FuncExecution(FuncId),
    #[error("func has no handler: {0}")]
    FuncHasNoHandler(FuncId),
    #[error("func is empty: {0}")]
    FuncIsEmpty(FuncId),
    #[error("Func {0} not found")]
    FuncNotFound(FuncId),
    #[error(transparent)]
    InstalledPkg(#[from] InstalledPkgError),
    #[error(transparent)]
    InternalProvider(#[from] InternalProviderError),
    #[error("No new asset was created")]
    NoAssetCreated,
    #[error(transparent)]
    Pg(#[from] si_data_pg::PgError),
    #[error(transparent)]
    PgPool(#[from] si_data_pg::PgPoolError),
    #[error(transparent)]
    Pkg(#[from] PkgError),
    #[error(transparent)]
    Schema(#[from] SchemaError),
    #[error("could not find schema connected to variant definition {0}")]
    SchemaNotFound(SchemaVariantDefinitionId),
    #[error(transparent)]
    SchemaVariant(#[from] SchemaVariantError),
    #[error(transparent)]
    SchemaVariantDefinition(#[from] DalSchemaVariantDefinitionError),
    #[error("could not find schema variant {0} connected to variant definition {1}")]
    SchemaVariantNotFound(SchemaVariantId, SchemaVariantDefinitionId),
    #[error(transparent)]
    SdfFunc(#[from] SdfFuncError),
    #[error("json serialization error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error(transparent)]
    SiPkg(#[from] SiPkgError),
    #[error(transparent)]
    Spec(#[from] SpecError),
    #[error(transparent)]
    StandardModel(#[from] StandardModelError),
    #[error("tenancy error: {0}")]
    Tenancy(#[from] TenancyError),
    #[error("transparent")]
    User(#[from] UserError),
    #[error(transparent)]
    ValidationPrototype(#[from] ValidationPrototypeError),
    #[error("Schema Variant Definition {0} not found")]
    VariantDefinitionNotFound(SchemaVariantDefinitionId),
    #[error("Cannot update asset structure while in use by components, attribute functions, or validations")]
    VariantInUse,
    #[error("could not publish websocket event: {0}")]
    WsEvent(#[from] WsEventError),
}

pub type SchemaVariantDefinitionResult<T> = Result<T, SchemaVariantDefinitionError>;

impl IntoResponse for SchemaVariantDefinitionError {
    fn into_response(self) -> Response {
        let (status, error_message) = (StatusCode::INTERNAL_SERVER_ERROR, self.to_string());

        let body = Json(
            serde_json::json!({ "error": { "message": error_message, "code": 42, "statusCode": status.as_u16() } }),
        );

        (status, body).into_response()
    }
}

pub async fn save_variant_def(
    ctx: &DalContext,
    request: &SaveVariantDefRequest,
    updated_func_name: Option<String>,
) -> SchemaVariantDefinitionResult<()> {
    let mut variant_def = SchemaVariantDefinition::get_by_id(ctx, &request.id)
        .await?
        .ok_or(SchemaVariantDefinitionError::VariantDefinitionNotFound(
            request.id,
        ))?;
    variant_def.set_name(ctx, request.name.clone()).await?;
    variant_def
        .set_menu_name(ctx, request.menu_name.clone())
        .await?;
    variant_def
        .set_category(ctx, request.category.clone())
        .await?;
    variant_def.set_color(ctx, &request.color).await?;
    variant_def.set_link(ctx, request.link.clone()).await?;
    variant_def
        .set_description(ctx, request.description.clone())
        .await?;
    variant_def
        .set_component_type(ctx, request.component_type)
        .await?;

    let mut asset_func = Func::get_by_id(ctx, &variant_def.func_id()).await?.ok_or(
        SchemaVariantDefinitionError::FuncNotFound(variant_def.func_id()),
    )?;
    asset_func
        .set_code_plaintext(ctx, Some(&request.code))
        .await?;
    asset_func.set_handler(ctx, Some(&request.handler)).await?;

    if let Some(updated_name) = updated_func_name {
        asset_func.set_name(ctx, updated_name).await?;
    }

    Ok(())
}

pub async fn is_variant_def_locked(
    ctx: &DalContext,
    variant_def: &SchemaVariantDefinition,
) -> SchemaVariantDefinitionResult<(bool, bool)> {
    let has_components = !variant_def.list_components(ctx).await?.is_empty();

    let has_attr_funcs = if let Some(schema_variant_id) = variant_def.schema_variant_id().copied() {
        SchemaVariant::all_funcs(ctx, schema_variant_id)
            .await?
            .iter()
            .any(|func| {
                func.backend_kind() == &FuncBackendKind::JsValidation
                    || (func.backend_kind() == &FuncBackendKind::JsAttribute
                        && func.name() != "si:resourcePayloadToValue"
                        && func.backend_response_type() != &FuncBackendResponseType::CodeGeneration
                        && func.backend_response_type() != &FuncBackendResponseType::Qualification)
            })
    } else {
        false
    };

    Ok((has_components, has_attr_funcs))
}

pub async fn migrate_actions_to_new_schema_variant(
    ctx: &DalContext,
    previous_schema_variant_id: SchemaVariantId,
    new_schema_variant_id: SchemaVariantId,
) -> SchemaVariantDefinitionResult<()> {
    let mut actions = ActionPrototype::find_for_context(
        ctx,
        ActionPrototypeContext {
            schema_variant_id: previous_schema_variant_id,
        },
    )
    .await?;

    for mut action in actions.drain(..) {
        action
            .set_schema_variant_id(ctx, new_schema_variant_id)
            .await?;
    }

    Ok(())
}

pub async fn migrate_leaf_functions_to_new_schema_variant(
    ctx: &DalContext,
    leaf_func_migrations: Vec<LeafFuncMigration>,
    new_schema_variant_id: SchemaVariantId,
) -> SchemaVariantDefinitionResult<()> {
    for leaf_func_migration in leaf_func_migrations {
        SchemaVariant::upsert_leaf_function(
            ctx,
            new_schema_variant_id,
            None,
            leaf_func_migration.leaf_kind,
            &leaf_func_migration.input_locations,
            &leaf_func_migration.func,
        )
        .await?;

        // TODO: delete attribute prototypes for orphaned leaf funcs
    }

    Ok(())
}

/// Removes all attribute prototypes, values, props, internal/external providers, sockets and
/// validation prototypes for a schema variant. Actions are migrated directly, so they are not
/// removed.
pub async fn cleanup_orphaned_objects(
    ctx: &DalContext,
    schema_variant_id: SchemaVariantId,
) -> SchemaVariantDefinitionResult<()> {
    for prototype in AttributePrototype::list_for_schema_variant(ctx, schema_variant_id).await? {
        AttributePrototype::remove(ctx, prototype.id(), true).await?;
    }

    for mut prop in SchemaVariant::all_props(ctx, schema_variant_id)
        .await?
        .drain(..)
    {
        prop.delete_by_id(ctx).await?;
    }

    for mut external_provider in ExternalProvider::list_for_schema_variant(ctx, schema_variant_id)
        .await?
        .drain(..)
    {
        for mut socket in external_provider.sockets(ctx).await?.drain(..) {
            socket.delete_by_id(ctx).await?;
        }
        external_provider.delete_by_id(ctx).await?;
    }

    for mut internal_provider in InternalProvider::list_for_schema_variant(ctx, schema_variant_id)
        .await?
        .drain(..)
    {
        for mut socket in internal_provider.sockets(ctx).await?.drain(..) {
            socket.delete_by_id(ctx).await?;
        }
        internal_provider.delete_by_id(ctx).await?;
    }

    for mut validation_prototype in
        ValidationPrototype::list_for_schema_variant(ctx, schema_variant_id)
            .await?
            .drain(..)
    {
        validation_prototype.delete_by_id(ctx).await?;
    }

    Ok(())
}

pub struct LeafFuncMigration {
    pub func: Func,
    pub leaf_kind: LeafKind,
    pub input_locations: Vec<LeafInputLocation>,
}

pub async fn maybe_delete_schema_variant_connected_to_variant_def(
    ctx: &DalContext,
    variant_def: &mut SchemaVariantDefinition,
) -> SchemaVariantDefinitionResult<(Option<SchemaVariantId>, Vec<LeafFuncMigration>)> {
    if matches!(
        is_variant_def_locked(ctx, variant_def).await?,
        (true, _) | (_, true)
    ) {
        return Err(SchemaVariantDefinitionError::VariantInUse);
    }

    let maybe_previous_schema_variant_id = variant_def.schema_variant_id().copied();
    let mut leaf_func_migrations = vec![];
    if let Some(schema_variant_id) = maybe_previous_schema_variant_id {
        let mut variant = SchemaVariant::get_by_id(ctx, &schema_variant_id)
            .await?
            .ok_or(SchemaVariantDefinitionError::SchemaVariantNotFound(
                schema_variant_id,
                *variant_def.id(),
            ))?;

        for leaf_kind in LeafKind::iter() {
            let leaf_funcs =
                SchemaVariant::find_leaf_item_functions(ctx, *variant.id(), leaf_kind).await?;
            for (_, func) in leaf_funcs {
                let input_locations = get_leaf_function_inputs(ctx, *func.id()).await?;
                leaf_func_migrations.push(LeafFuncMigration {
                    func: func.to_owned(),
                    leaf_kind,
                    input_locations,
                });
            }
        }

        let mut schema =
            variant
                .schema(ctx)
                .await?
                .ok_or(SchemaVariantDefinitionError::SchemaNotFound(
                    *variant_def.id(),
                ))?;

        cleanup_orphaned_objects(ctx, *variant.id()).await?;

        variant.delete_by_id(ctx).await?;
        for mut ui_menu in schema.ui_menus(ctx).await? {
            ui_menu.delete_by_id(ctx).await?;
        }
        schema.delete_by_id(ctx).await?;

        variant_def
            .set_schema_variant_id(ctx, None::<SchemaVariantId>)
            .await?;
    }

    Ok((maybe_previous_schema_variant_id, leaf_func_migrations))
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/list_variant_defs",
            get(list_variant_defs::list_variant_defs),
        )
        .route("/get_variant_def", get(get_variant_def::get_variant_def))
        .route(
            "/save_variant_def",
            post(save_variant_def::save_variant_def),
        )
        .route(
            "/create_variant_def",
            post(create_variant_def::create_variant_def),
        )
        .route(
            "/exec_variant_def",
            post(exec_variant_def::exec_variant_def),
        )
        .route(
            "/clone_variant_def",
            post(clone_variant_def::clone_variant_def),
        )
}
