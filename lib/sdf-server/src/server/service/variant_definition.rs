use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use telemetry::prelude::*;
use thiserror::Error;

use dal::authentication_prototype::AuthenticationPrototypeError;
use dal::{
    attribute::prototype::argument::{AttributePrototypeArgument, AttributePrototypeArgumentError},
    func::argument::{FuncArgumentError, FuncArgumentId},
    installed_pkg::InstalledPkgError,
    pkg::PkgError,
    schema::variant::definition::SchemaVariantDefinition,
    schema::variant::definition::{
        SchemaVariantDefinitionError as DalSchemaVariantDefinitionError, SchemaVariantDefinitionId,
    },
    socket::SocketError,
    ActionPrototypeError, AttributeContext, AttributeContextBuilderError, AttributeContextError,
    AttributePrototype, AttributePrototypeError, AttributePrototypeId, AttributeValueError,
    ChangeSetError, DalContext, ExternalProvider, ExternalProviderError, Func, FuncBinding,
    FuncBindingError, FuncError, FuncId, InternalProvider, InternalProviderError,
    LeafInputLocation, LeafKind, Prop, PropError, PropKind, SchemaError, SchemaVariant,
    SchemaVariantError, SchemaVariantId, SocketId, StandardModel, StandardModelError, TenancyError,
    TransactionsError, UserError, WsEventError,
};
use si_pkg::{SiPkgError, SpecError};

use crate::server::state::AppState;
use crate::service::func::FuncError as SdfFuncError;

use self::save_variant_def::SaveVariantDefRequest;

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
    AuthenticationPrototype(#[from] AuthenticationPrototypeError),
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
    #[error(transparent)]
    FuncArgument(#[from] FuncArgumentError),
    #[error("func argument not found: {0}")]
    FuncArgumentNotFound(FuncArgumentId),
    #[error(transparent)]
    FuncBinding(#[from] FuncBindingError),
    #[error("func execution error: {0}")]
    FuncExecution(FuncId),
    #[error("func execution failure error: {0}")]
    FuncExecutionFailure(String),
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
    #[error("updating the schema variant found an invalid state: {0}")]
    InvalidState(String),
    #[error("No new asset was created")]
    NoAssetCreated,
    #[error(transparent)]
    ParseUrl(#[from] url::ParseError),
    #[error(transparent)]
    Pg(#[from] si_data_pg::PgError),
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
    #[error("type prop missing for map or array")]
    TypePropMissingForMapOrArray,
    #[error(transparent)]
    Ulid(#[from] ulid::DecodeError),
    #[error(transparent)]
    User(#[from] UserError),
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

    if let Some(updated_name) = updated_func_name {
        asset_func.set_name(ctx, updated_name).await?;
    }

    Ok(())
}

pub async fn is_variant_def_locked(
    ctx: &DalContext,
    variant_def: &SchemaVariantDefinition,
) -> SchemaVariantDefinitionResult<bool> {
    Ok(!variant_def.list_components(ctx).await?.is_empty())
}

// Returns prototypes that were not migrated

#[derive(Clone, Debug)]
pub struct ParentAttributeValueDefinition {
    prop_path: String,
    key: Option<String>,
}

#[derive(Clone, Debug)]
pub struct AttributeValueDefinition {
    pub parent: Option<ParentAttributeValueDefinition>,
    pub value: Option<serde_json::Value>,
}

#[derive(Clone, Debug)]
pub struct AttributePrototypeArgumentDefinition {
    kind: AttributePrototypeArgumentKind,
    func_argument_id: FuncArgumentId,
    name: String,
}

#[remain::sorted]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum AttributePrototypeArgumentKind {
    Invalid,
    Prop { kind: PropKind },
    Socket,
}

#[remain::sorted]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", content = "data")]
pub enum AttributePrototypeContextKind {
    ExternalProvider { name: String },
    Prop { path: String, kind: PropKind },
}

#[derive(Clone, Debug)]
pub struct AttributePrototypeDefinition {
    pub id: AttributePrototypeId,
    pub func_id: FuncId,
    pub attribute_value: AttributeValueDefinition,
    pub context: AttributePrototypeContextKind,
    pub arguments: Vec<AttributePrototypeArgumentDefinition>,
    pub key: Option<String>,
}

// Returns prototypes that were not migrated
pub async fn migrate_attribute_functions_to_new_schema_variant(
    ctx: &DalContext,
    attribute_prototypes: Vec<AttributePrototypeDefinition>,
    new_schema_variant: &SchemaVariant,
) -> SchemaVariantDefinitionResult<Vec<AttributePrototypeDefinition>> {
    let new_props = SchemaVariant::all_props(ctx, *new_schema_variant.id()).await?;
    let new_external_providers =
        ExternalProvider::list_for_schema_variant(ctx, *new_schema_variant.id()).await?;
    let new_internal_providers =
        InternalProvider::list_for_schema_variant(ctx, *new_schema_variant.id()).await?;

    let mut invalid_attribute_prototypes = Vec::new();
    'outer: for attribute_prototype in attribute_prototypes {
        let (new_context, new_attribute_prototype) = match &attribute_prototype.context {
            AttributePrototypeContextKind::ExternalProvider { name } => {
                if let Some(new_external_provider) =
                    new_external_providers.iter().find(|s| s.name() == name)
                {
                    let context = AttributeContext::builder()
                        .set_external_provider_id(*new_external_provider.id())
                        .to_context()?;
                    let prototype = AttributePrototype::find_for_context_and_key(
                        ctx,
                        context,
                        &attribute_prototype.key,
                    )
                    .await?
                    .pop();
                    (context, prototype)
                } else {
                    // Arguments don't match, bail on this prototype
                    invalid_attribute_prototypes.push(attribute_prototype);
                    continue 'outer;
                }
            }
            AttributePrototypeContextKind::Prop { path, kind } => {
                if let Some(new_prop) = new_props
                    .iter()
                    .find(|p| p.path().as_str() == path && p.kind() == kind)
                {
                    let context = AttributeContext::builder()
                        .set_prop_id(*new_prop.id())
                        .to_context()?;
                    let prototype = AttributePrototype::find_for_context_and_key(
                        ctx,
                        context,
                        &attribute_prototype.key,
                    )
                    .await?
                    .pop();
                    (context, prototype)
                } else {
                    // Arguments don't match, bail on this prototype
                    invalid_attribute_prototypes.push(attribute_prototype);
                    continue 'outer;
                }
            }
        };

        let mut arguments_to_create = Vec::new();
        for apa in &attribute_prototype.arguments {
            match apa.kind {
                AttributePrototypeArgumentKind::Prop { kind } => {
                    if let Some(new_prop) = new_props
                        .iter()
                        .find(|p| p.path().as_str() == apa.name && p.kind() == &kind)
                    {
                        let new_internal_provider =
                            InternalProvider::find_for_prop(ctx, *new_prop.id())
                                .await?
                                .ok_or_else(|| {
                                    InternalProviderError::NotFoundForProp(*new_prop.id())
                                })?;
                        arguments_to_create.push((apa, *new_internal_provider.id()));
                    } else {
                        // Arguments don't match, bail on this prototype
                        invalid_attribute_prototypes.push(attribute_prototype);
                        continue 'outer;
                    }
                }
                AttributePrototypeArgumentKind::Socket => {
                    if let Some(new_internal_provider) = new_internal_providers
                        .iter()
                        .find(|s| s.name() == apa.name && s.prop_id().is_none())
                    {
                        arguments_to_create.push((apa, *new_internal_provider.id()));
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

        if let Some(mut new_attribute_prototype) = new_attribute_prototype {
            new_attribute_prototype
                .set_func_id(ctx, attribute_prototype.func_id)
                .await?;
            for mut arg in AttributePrototypeArgument::list_for_attribute_prototype(
                ctx,
                *new_attribute_prototype.id(),
            )
            .await?
            {
                arg.delete_by_id(ctx).await?;
            }

            for (argument, id) in arguments_to_create {
                AttributePrototypeArgument::new_for_intra_component(
                    ctx,
                    *new_attribute_prototype.id(),
                    argument.func_argument_id,
                    id,
                )
                .await?;
            }

            continue;
        }

        let existing_value = &attribute_prototype.attribute_value;
        let new_maybe_parent_attribute_value = if let Some(parent) = &existing_value.parent {
            if let Some(new_prop) = new_props
                .iter()
                .find(|p| p.path().as_str() == parent.prop_path)
            {
                let context = AttributeContext::builder()
                    .set_prop_id(*new_prop.id())
                    .to_context()?;
                if let Some(prototype) =
                    AttributePrototype::find_for_context_and_key(ctx, context, &parent.key)
                        .await?
                        .pop()
                {
                    prototype
                        .attribute_values(ctx)
                        .await?
                        .pop()
                        .map(|v| *v.id())
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

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

        // TODO: create index map
        let prototype = AttributePrototype::new(
            ctx,
            attribute_prototype.func_id,
            *func_binding.id(),
            *fbrv.id(),
            new_context,
            attribute_prototype.key,
            new_maybe_parent_attribute_value,
        )
        .await?;

        for (argument, id) in arguments_to_create {
            AttributePrototypeArgument::new_for_intra_component(
                ctx,
                *prototype.id(),
                argument.func_argument_id,
                id,
            )
            .await?;
        }
    }

    Ok(invalid_attribute_prototypes)
}

/// Removes all attribute prototypes, values, props, internal/external providers, sockets and
/// validation prototypes for a schema variant. Actions are migrated directly, so they are not
/// removed.
pub async fn cleanup_orphaned_objects(
    ctx: &DalContext,
    schema_variant_id: SchemaVariantId,
) -> SchemaVariantDefinitionResult<Vec<AttributePrototypeDefinition>> {
    let mut attribute_prototypes = Vec::new();
    for prototype in AttributePrototype::list_for_schema_variant(ctx, schema_variant_id).await? {
        let args =
            AttributePrototypeArgument::list_for_attribute_prototype(ctx, *prototype.id()).await?;
        let mut arguments = Vec::with_capacity(args.len());
        for argument in args {
            let (kind, name) = if argument.tail_component_id().is_some()
                || argument.head_component_id().is_some()
            {
                (AttributePrototypeArgumentKind::Invalid, String::new())
            } else if let Some(internal_provider) =
                InternalProvider::get_by_id(ctx, &argument.internal_provider_id()).await?
            {
                let (kind, name) = if internal_provider.sockets(ctx).await?.pop().is_some() {
                    let kind = AttributePrototypeArgumentKind::Socket;
                    (kind, internal_provider.name().to_owned())
                } else if let Some(prop) = Prop::get_by_id(ctx, internal_provider.prop_id()).await?
                {
                    let kind = AttributePrototypeArgumentKind::Prop { kind: *prop.kind() };
                    (kind, prop.path().as_str().to_owned())
                } else {
                    return Err(SchemaVariantDefinitionError::InvalidState(format!(
                        "internal provider should have a prop or a socket: {argument:?}"
                    )));
                };
                (kind, name)
            } else {
                // External Providers require tail_component_id and
                // head_component_id, which should not exist at a schema level
                // attribute prototype argument, and are handled by the first branch
                // so unreachable
                return Err(SchemaVariantDefinitionError::InvalidState(format!("attribute prototype argument for custom function is taking an input that's not a prop or a input socket: {argument:?}")));
            };

            arguments.push(AttributePrototypeArgumentDefinition {
                func_argument_id: argument.func_argument_id(),
                name,
                kind,
            });
        }

        let attribute_value = if let Some(attribute_value) =
            prototype.attribute_values(ctx).await?.pop()
        {
            let prop = if let Some(parent) = attribute_value.parent_attribute_value(ctx).await? {
                Prop::get_by_id(ctx, &parent.context.prop_id()).await?
            } else {
                None
            };
            AttributeValueDefinition {
                parent: prop.map(|prop| ParentAttributeValueDefinition {
                    prop_path: prop.path().as_str().to_owned(),
                    key: attribute_value.key().map(ToOwned::to_owned),
                }),
                value: attribute_value.get_value(ctx).await?,
            }
        } else {
            AttributeValueDefinition {
                parent: None,
                value: None,
            }
        };

        let func = Func::get_by_id(ctx, &prototype.func_id())
            .await?
            .ok_or_else(|| SchemaVariantDefinitionError::FuncNotFound(prototype.func_id()))?;

        if !func.is_intrinsic() {
            attribute_prototypes.push(AttributePrototypeDefinition {
                id: *prototype.id(),
                func_id: prototype.func_id(),
                attribute_value,
                context: if let Some(prop) = prototype.context.prop(ctx).await? {
                    AttributePrototypeContextKind::Prop {
                        path: prop.path().as_str().to_owned(),
                        kind: *prop.kind(),
                    }
                } else if let Some(external_provider) =
                    prototype.context.external_provider(ctx).await?
                {
                    AttributePrototypeContextKind::ExternalProvider {
                        name: external_provider.name().to_owned(),
                    }
                } else if prototype.context.internal_provider_id().is_some() {
                    continue;
                } else if prototype.context.component_id().is_some() {
                    // We disabled that feature, if we decide to bring it back we will need to support it
                    // here too
                    return Err(SchemaVariantDefinitionError::InvalidState(format!(
                        "attribute prototype should not have a component: {prototype:?}"
                    )));
                } else {
                    return Err(SchemaVariantDefinitionError::InvalidState(format!(
                        "attribute prototype should have a context: {prototype:?}"
                    )));
                },
                key: prototype.key().map(ToOwned::to_owned),
                arguments,
            });
        }

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

    Ok(attribute_prototypes)
}

#[derive(Debug)]
pub struct LeafFuncMigration {
    pub func: Func,
    pub leaf_kind: LeafKind,
    pub input_locations: Vec<LeafInputLocation>,
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
