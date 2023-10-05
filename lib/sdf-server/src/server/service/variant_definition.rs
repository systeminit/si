use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use telemetry::prelude::*;
use thiserror::Error;

use dal::{
    attribute::prototype::argument::{AttributePrototypeArgument, AttributePrototypeArgumentError},
    func::argument::FuncArgumentId,
    installed_pkg::InstalledPkgError,
    pkg::PkgError,
    schema::variant::definition::SchemaVariantDefinition,
    schema::variant::definition::{
        SchemaVariantDefinitionError as DalSchemaVariantDefinitionError, SchemaVariantDefinitionId,
    },
    socket::SocketEdgeKind,
    socket::SocketError,
    ActionPrototype, ActionPrototypeContext, ActionPrototypeError, AttributeContext,
    AttributeContextBuilderError, AttributeContextError, AttributePrototype,
    AttributePrototypeError, AttributePrototypeId, AttributeValueError, AttributeValueId,
    ChangeSetError, DalContext, ExternalProvider, ExternalProviderError, Func, FuncBackendKind,
    FuncBackendResponseType, FuncBinding, FuncBindingError, FuncError, FuncId, InternalProvider,
    InternalProviderError, InternalProviderId, LeafInputLocation, LeafKind, Prop, PropError,
    PropKind, SchemaError, SchemaVariant, SchemaVariantError, SchemaVariantId, SocketId,
    StandardModel, StandardModelError, TenancyError, TransactionsError, UserError,
    ValidationPrototype, ValidationPrototypeContext, ValidationPrototypeError,
    ValidationPrototypeId, WsEventError,
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
    AttributeContext(#[from] AttributeContextError),
    #[error(transparent)]
    AttributeContextBuilder(#[from] AttributeContextBuilderError),
    #[error(transparent)]
    AttributePrototype(#[from] AttributePrototypeError),
    #[error(transparent)]
    AttributePrototypeArgument(#[from] AttributePrototypeArgumentError),
    #[error(transparent)]
    AttributeValue(#[from] AttributeValueError),
    #[error(transparent)]
    ChangeSet(#[from] ChangeSetError),
    #[error(transparent)]
    ContextTransaction(#[from] TransactionsError),
    #[error("error creating schema variant from definition: {0}")]
    CouldNotCreateSchemaVariantFromDefinition(String),
    #[error(transparent)]
    ExternalProvider(#[from] ExternalProviderError),
    #[error("external provider not found for socket: {0}")]
    ExternalProviderNotFoundForSocket(SocketId),
    #[error("func error: {0}")]
    Func(#[from] FuncError),
    #[error("func argument not found: {0}")]
    FuncArgumentNotFound(FuncArgumentId),
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
    Hyper(#[from] hyper::http::Error),
    #[error(transparent)]
    InstalledPkg(#[from] InstalledPkgError),
    #[error(transparent)]
    InternalProvider(#[from] InternalProviderError),
    #[error("internal provider not found for socket: {0}")]
    InternalProviderNotFoundForSocket(SocketId),
    #[error("No new asset was created")]
    NoAssetCreated,
    #[error(transparent)]
    Pg(#[from] si_data_pg::PgError),
    #[error(transparent)]
    PgPool(#[from] si_data_pg::PgPoolError),
    #[error(transparent)]
    Pkg(#[from] PkgError),
    #[error(transparent)]
    Prop(#[from] PropError),
    #[error(transparent)]
    Schema(#[from] SchemaError),
    #[error("could not find schema connected to variant definition {0}")]
    SchemaNotFound(SchemaVariantDefinitionId),
    #[error("could not find schema connected to variant {0}")]
    SchemaNotFoundForVariant(SchemaVariantId),
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
    Socket(#[from] SocketError),
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

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ValidationPrototypeDefinition {
    pub id: ValidationPrototypeId,
    pub func_id: FuncId,
    pub args: serde_json::Value,
    pub link: Option<String>,
    pub prop_path: String,
    pub prop_kind: PropKind,
}

// Returns prototypes that were not migrated
pub async fn migrate_validation_functions_to_new_schema_variant(
    ctx: &DalContext,
    validation_prototypes: Vec<ValidationPrototypeDefinition>,
    new_schema_variant_id: SchemaVariantId,
) -> SchemaVariantDefinitionResult<Vec<ValidationPrototypeDefinition>> {
    let schema_variant = SchemaVariant::get_by_id(ctx, &new_schema_variant_id)
        .await?
        .ok_or(SchemaVariantError::NotFound(new_schema_variant_id))?;
    let schema = schema_variant.schema(ctx).await?.ok_or(
        SchemaVariantDefinitionError::SchemaNotFoundForVariant(new_schema_variant_id),
    )?;

    let new_props = SchemaVariant::all_props(ctx, new_schema_variant_id).await?;
    let mut invalid_validation_prototypes = Vec::new();
    for validation_prototype in validation_prototypes {
        if let Some(new_prop) = new_props.iter().find(|p| {
            p.path().as_str() == validation_prototype.prop_path
                && p.kind() == &validation_prototype.prop_kind
        }) {
            let mut prototype = ValidationPrototype::new(
                ctx,
                validation_prototype.func_id,
                validation_prototype.args,
                ValidationPrototypeContext::builder()
                    .set_prop_id(*new_prop.id())
                    .set_schema_variant_id(new_schema_variant_id)
                    .set_schema_id(*schema.id())
                    .to_context(ctx)
                    .await?,
            )
            .await?;
            prototype.set_link(ctx, validation_prototype.link).await?;
        } else {
            // Arguments don't match, bail on this prototype
            invalid_validation_prototypes.push(validation_prototype);
        }
    }
    Ok(invalid_validation_prototypes)
}

#[derive(Clone, Debug)]
pub struct AttributeValueDefinition {
    pub parent: Option<AttributeValueId>,
    pub value: Option<serde_json::Value>,
}

#[derive(Clone, Debug)]
pub struct AttributePrototypeArgumentDefinition {
    kind: AttributePrototypeArgumentKind,
    internal_provider_id: InternalProviderId,
    func_argument_id: FuncArgumentId,
    name: String,
}

#[remain::sorted]
#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
#[serde(tag = "type")]
pub enum AttributePrototypeArgumentKind {
    InternalProviderProp { kind: PropKind },
    InternalProviderSocket { kind: SocketEdgeKind },
    Invalid,
}

#[remain::sorted]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum AttributePrototypeContextKind {
    ExternalProviderSocket { name: String, kind: SocketEdgeKind },
    InternalProviderProp { path: String, kind: PropKind },
    InternalProviderSocket { name: String, kind: SocketEdgeKind },
    Prop { path: String, kind: PropKind },
}

#[derive(Clone, Debug)]
pub struct AttributePrototypeDefinition {
    pub id: AttributePrototypeId,
    pub func_id: FuncId,
    pub attribute_value: AttributeValueDefinition,
    pub context: AttributePrototypeContextKind,
    pub key: Option<String>,
    pub arguments: Vec<AttributePrototypeArgumentDefinition>,
}

// Returns prototypes that were not migrated
pub async fn migrate_attribute_functions_to_new_schema_variant(
    ctx: &DalContext,
    attribute_prototypes: Vec<AttributePrototypeDefinition>,
    new_schema_variant: &SchemaVariant,
) -> SchemaVariantDefinitionResult<Vec<AttributePrototypeDefinition>> {
    let new_props = SchemaVariant::all_props(ctx, *new_schema_variant.id()).await?;
    let new_sockets = new_schema_variant.sockets(ctx).await?;
    let mut invalid_attribute_prototypes = Vec::new();
    'outer: for attribute_prototype in attribute_prototypes {
        let existing_value = &attribute_prototype.attribute_value;
        let maybe_parent_attribute_value = existing_value.parent;

        let mut arguments_to_create = Vec::new();
        for apa in &attribute_prototype.arguments {
            match apa.kind {
                AttributePrototypeArgumentKind::InternalProviderProp { kind } => {
                    if let Some(new_prop) = new_props
                        .iter()
                        .find(|p| p.path().as_str() == apa.name && p.kind() == &kind)
                    {
                        let internal_provider =
                            InternalProvider::find_for_prop(ctx, *new_prop.id())
                                .await?
                                .ok_or_else(|| {
                                    InternalProviderError::NotFoundForProp(*new_prop.id())
                                })?;

                        let mut apa = apa.clone();
                        apa.internal_provider_id = *internal_provider.id();
                        arguments_to_create.push(apa);
                    } else {
                        // Arguments don't match, bail on this prototype
                        invalid_attribute_prototypes.push(attribute_prototype);
                        continue 'outer;
                    }
                }
                AttributePrototypeArgumentKind::InternalProviderSocket { kind } => {
                    if let Some(new_socket) = new_sockets
                        .iter()
                        .find(|s| s.name() == apa.name && s.edge_kind() == &kind)
                    {
                        let internal_provider =
                            new_socket.internal_provider(ctx).await?.ok_or_else(|| {
                                InternalProviderError::NotFoundForSocket(*new_socket.id())
                            })?;

                        let mut apa = apa.clone();
                        apa.internal_provider_id = *internal_provider.id();
                        arguments_to_create.push(apa);
                    } else {
                        // Arguments don't match, bail on this prototype
                        invalid_attribute_prototypes.push(attribute_prototype);
                        continue 'outer;
                    }
                }
                AttributePrototypeArgumentKind::Invalid => {
                    invalid_attribute_prototypes.push(attribute_prototype);
                    continue 'outer;
                }
            }
        }

        let (mut func_binding, fbrv) = FuncBinding::create_with_existing_value(
            ctx,
            serde_json::json!({}),
            existing_value.value.clone(),
            attribute_prototype.func_id,
        )
        .await?;

        // Clear out the function sha so we know to execute this on the first run in
        // AttributeValue::update_from_prototype_function
        func_binding.set_code_sha256(ctx, "0").await?;

        let context = match &attribute_prototype.context {
            AttributePrototypeContextKind::Prop { path, kind } => {
                if let Some(new_prop) = new_props
                    .iter()
                    .find(|p| p.path().as_str() == path && p.kind() == kind)
                {
                    AttributeContext::builder()
                        .set_prop_id(*new_prop.id())
                        .to_context()?
                } else {
                    // Arguments don't match, bail on this prototype
                    invalid_attribute_prototypes.push(attribute_prototype);
                    continue;
                }
            }
            AttributePrototypeContextKind::InternalProviderProp { path, kind } => {
                if let Some(new_prop) = new_props
                    .iter()
                    .find(|p| p.path().as_str() == path && p.kind() == kind)
                {
                    AttributeContext::builder()
                        .set_prop_id(*new_prop.id())
                        .to_context()?
                } else {
                    // Arguments don't match, bail on this prototype
                    invalid_attribute_prototypes.push(attribute_prototype);
                    continue;
                }
            }
            AttributePrototypeContextKind::InternalProviderSocket { name, kind } => {
                if let Some(new_socket) = new_sockets
                    .iter()
                    .find(|s| s.name() == name && s.edge_kind() == kind)
                {
                    let internal_provider =
                        new_socket.internal_provider(ctx).await?.ok_or_else(|| {
                            SchemaVariantDefinitionError::InternalProviderNotFoundForSocket(
                                *new_socket.id(),
                            )
                        })?;

                    AttributeContext::builder()
                        .set_internal_provider_id(*internal_provider.id())
                        .to_context()?
                } else {
                    // Arguments don't match, bail on this prototype
                    invalid_attribute_prototypes.push(attribute_prototype);
                    continue;
                }
            }
            AttributePrototypeContextKind::ExternalProviderSocket { name, kind } => {
                if let Some(new_socket) = new_sockets
                    .iter()
                    .find(|s| s.name() == name && s.edge_kind() == kind)
                {
                    let external_provider =
                        new_socket.external_provider(ctx).await?.ok_or_else(|| {
                            SchemaVariantDefinitionError::ExternalProviderNotFoundForSocket(
                                *new_socket.id(),
                            )
                        })?;

                    AttributeContext::builder()
                        .set_external_provider_id(*external_provider.id())
                        .to_context()?
                } else {
                    // Arguments don't match, bail on this prototype
                    invalid_attribute_prototypes.push(attribute_prototype);
                    continue;
                }
            }
        };

        let prototype = AttributePrototype::new(
            ctx,
            attribute_prototype.func_id,
            *func_binding.id(),
            *fbrv.id(),
            context,
            attribute_prototype.key,
            maybe_parent_attribute_value,
        )
        .await?;

        for argument in arguments_to_create {
            AttributePrototypeArgument::new_for_intra_component(
                ctx,
                *prototype.id(),
                argument.func_argument_id,
                argument.internal_provider_id,
            )
            .await?;
        }
    }

    Ok(invalid_attribute_prototypes)
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
) -> SchemaVariantDefinitionResult<(
    Vec<AttributePrototypeDefinition>,
    Vec<ValidationPrototypeDefinition>,
)> {
    let mut attribute_prototypes = Vec::new();
    for prototype in AttributePrototype::list_for_schema_variant(ctx, schema_variant_id).await? {
        let args =
            AttributePrototypeArgument::list_for_attribute_prototype(ctx, *prototype.id()).await?;
        let mut arguments = Vec::with_capacity(args.len());
        for argument in args {
            let (kind, name, internal_provider_id) = if argument.tail_component_id().is_some()
                || argument.head_component_id().is_some()
            {
                (
                    AttributePrototypeArgumentKind::Invalid,
                    String::new(),
                    InternalProviderId::NONE,
                )
            } else if let Some(internal_provider) =
                InternalProvider::get_by_id(ctx, &argument.internal_provider_id()).await?
            {
                let (kind, name) = if let Some(prop) =
                    Prop::get_by_id(ctx, internal_provider.prop_id()).await?
                {
                    let kind =
                        AttributePrototypeArgumentKind::InternalProviderProp { kind: *prop.kind() };
                    (kind, prop.path().as_str().to_string())
                } else if let Some(socket) = internal_provider.sockets(ctx).await?.pop() {
                    let kind = AttributePrototypeArgumentKind::InternalProviderSocket {
                        kind: *socket.edge_kind(),
                    };
                    (kind, socket.name().to_owned())
                } else {
                    // A internal provider should always have a socket or a prop
                    unreachable!();
                };
                (kind, name, *internal_provider.id())
            } else {
                // External Providers require tail_component_id and
                // head_component_id, which should not exist at a schema level
                // attribute prototype argument, and are handled by the first branch
                // so unreachable
                unreachable!();
            };

            arguments.push(AttributePrototypeArgumentDefinition {
                func_argument_id: argument.func_argument_id(),
                name,
                kind,
                internal_provider_id,
            });
        }

        let attribute_value =
            if let Some(attribute_value) = prototype.attribute_values(ctx).await?.pop() {
                AttributeValueDefinition {
                    parent: attribute_value
                        .parent_attribute_value(ctx)
                        .await?
                        .map(|a| *a.id()),
                    value: attribute_value.get_value(ctx).await?,
                }
            } else {
                AttributeValueDefinition {
                    parent: None,
                    value: None,
                }
            };

        attribute_prototypes.push(AttributePrototypeDefinition {
            id: *prototype.id(),
            func_id: prototype.func_id(),
            attribute_value,
            context: if let Some(prop) = prototype.context.prop(ctx).await? {
                AttributePrototypeContextKind::Prop {
                    path: prop.path().as_str().to_owned(),
                    kind: *prop.kind(),
                }
            } else if let Some(internal_provider) = prototype.context.internal_provider(ctx).await?
            {
                if let Some(prop) = Prop::get_by_id(ctx, internal_provider.prop_id()).await? {
                    AttributePrototypeContextKind::InternalProviderProp {
                        path: prop.path().as_str().to_owned(),
                        kind: *prop.kind(),
                    }
                } else if let Some(socket) = internal_provider.sockets(ctx).await?.pop() {
                    AttributePrototypeContextKind::InternalProviderSocket {
                        name: socket.name().to_owned(),
                        kind: *socket.edge_kind(),
                    }
                } else {
                    unreachable!()
                }
            } else if let Some(external_provider) = prototype.context.external_provider(ctx).await?
            {
                if let Some(socket) = external_provider.sockets(ctx).await?.pop() {
                    AttributePrototypeContextKind::ExternalProviderSocket {
                        name: socket.name().to_owned(),
                        kind: *socket.edge_kind(),
                    }
                } else {
                    unreachable!()
                }
            } else if prototype.context.component_id().is_some() {
                // We disabled that feature, if we decide to bring it back we will need to support it
                // here too
                unreachable!();
            } else {
                unreachable!("{:?}", prototype);
            },
            key: prototype.key().map(ToOwned::to_owned),
            arguments,
        });

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

    let mut validation_prototypes = Vec::new();
    for mut validation_prototype in
        ValidationPrototype::list_for_schema_variant(ctx, schema_variant_id).await?
    {
        validation_prototype.delete_by_id(ctx).await?;

        let prop = validation_prototype.prop(ctx).await?;
        validation_prototypes.push(ValidationPrototypeDefinition {
            id: *validation_prototype.id(),
            func_id: validation_prototype.func_id(),
            args: validation_prototype.args().clone(),
            link: validation_prototype.link().map(ToOwned::to_owned),
            prop_path: prop.path().as_str().to_owned(),
            prop_kind: *prop.kind(),
        });
    }

    Ok((attribute_prototypes, validation_prototypes))
}

#[derive(Debug)]
pub struct LeafFuncMigration {
    pub func: Func,
    pub leaf_kind: LeafKind,
    pub input_locations: Vec<LeafInputLocation>,
}

pub async fn maybe_delete_schema_variant_connected_to_variant_def(
    ctx: &DalContext,
    variant_def: &mut SchemaVariantDefinition,
    auto_reattach: bool,
) -> SchemaVariantDefinitionResult<(
    Option<SchemaVariantId>,
    Vec<LeafFuncMigration>,
    Vec<AttributePrototypeDefinition>,
    Vec<ValidationPrototypeDefinition>,
)> {
    let (has_components, has_attr_funcs) = is_variant_def_locked(ctx, variant_def).await?;
    if has_components || (has_attr_funcs && !auto_reattach) {
        return Err(SchemaVariantDefinitionError::VariantInUse);
    }

    let maybe_previous_schema_variant_id = variant_def.schema_variant_id().copied();
    let mut leaf_func_migrations = vec![];
    let (attribute_prototypes, validation_prototypes) =
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

            let (attribute_prototypes, validation_prototypes) =
                cleanup_orphaned_objects(ctx, *variant.id()).await?;

            variant.delete_by_id(ctx).await?;
            for mut ui_menu in schema.ui_menus(ctx).await? {
                ui_menu.delete_by_id(ctx).await?;
            }
            schema.delete_by_id(ctx).await?;

            variant_def
                .set_schema_variant_id(ctx, None::<SchemaVariantId>)
                .await?;
            (attribute_prototypes, validation_prototypes)
        } else {
            (Vec::new(), Vec::new())
        };

    Ok((
        maybe_previous_schema_variant_id,
        leaf_func_migrations,
        attribute_prototypes,
        validation_prototypes,
    ))
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
