use crate::server::{impl_default_error_into_response, state::AppState};
use crate::service::func::get_func::GetFuncResponse;
use axum::{
    response::Response,
    routing::{get, post},
    Json, Router,
};
use dal::{
    attribute::context::{AttributeContextBuilder, AttributeContextBuilderError},
    func::{
        argument::{FuncArgument, FuncArgumentError, FuncArgumentId, FuncArgumentKind},
        binding_return_value::FuncBindingReturnValueError,
    },
    prop_tree::PropTreeError,
    prototype_context::PrototypeContextError,
    schema::variant::SchemaVariantError,
    AttributeContext, AttributeContextError, AttributePrototype, AttributePrototypeArgumentError,
    AttributePrototypeArgumentId, AttributePrototypeError, AttributePrototypeId,
    AttributeValueError, ComponentError, ComponentId, DalContext, Func, FuncBackendKind,
    FuncBackendResponseType, FuncBindingError, FuncId, InternalProviderError, InternalProviderId,
    PropError, PropId, PrototypeListForFuncError, SchemaVariantId, StandardModel,
    StandardModelError, TenancyError, TransactionsError, ValidationPrototype,
    ValidationPrototypeError, ValidationPrototypeId, WsEventError,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub mod create_func;
pub mod exec_func;
pub mod get_func;
pub mod list_funcs;
pub mod list_input_sources;
pub mod revert_func;
pub mod save_func;

#[derive(Error, Debug)]
pub enum FuncError {
    #[error(transparent)]
    Pg(#[from] si_data_pg::PgError),
    #[error(transparent)]
    PgPool(#[from] si_data_pg::PgPoolError),
    #[error("tenancy error: {0}")]
    Tenancy(#[from] TenancyError),
    #[error(transparent)]
    StandardModel(#[from] StandardModelError),
    #[error(transparent)]
    ContextTransaction(#[from] TransactionsError),
    #[error(transparent)]
    Func(#[from] dal::FuncError),
    #[error("could not publish websocket event: {0}")]
    WsEvent(#[from] WsEventError),
    #[error("json serialization error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("attribute prototype error: {0}")]
    AttributePrototype(#[from] AttributePrototypeError),
    #[error("attribute prototype argument error: {0}")]
    AttributePrototypeArgument(#[from] AttributePrototypeArgumentError),
    #[error("func binding error: {0}")]
    FuncBinding(#[from] FuncBindingError),
    #[error("component error: {0}")]
    Component(#[from] ComponentError),
    #[error("attribute value error: {0}")]
    AttributeValue(#[from] AttributeValueError),
    #[error("prop error: {0}")]
    Prop(#[from] PropError),
    #[error("attribute context builder error: {0}")]
    AttributeContextBuilder(#[from] AttributeContextBuilderError),
    #[error("attribute context error: {0}")]
    AttributeContext(#[from] AttributeContextError),
    #[error("func binding return value error: {0}")]
    FuncBindingReturnValue(#[from] FuncBindingReturnValueError),
    #[error("func argument error: {0}")]
    FuncArgument(#[from] FuncArgumentError),
    #[error("internal provider error: {0}")]
    InternalProvider(#[from] InternalProviderError),
    #[error("prop tree error: {0}")]
    PropTree(#[from] PropTreeError),
    #[error("schema variant error: {0}")]
    SchemaVariant(#[from] SchemaVariantError),
    #[error("prototype context error: {0}")]
    PrototypeContext(#[from] PrototypeContextError),
    #[error("prototype list for func error: {0}")]
    PrototypeListForFunc(#[from] PrototypeListForFuncError),
    #[error("validation prototype error: {0}")]
    ValidationPrototype(#[from] ValidationPrototypeError),

    #[error("Function not found")]
    FuncNotFound,
    #[error("Function is read-only")]
    NotWritable,
    #[error("Missing required options for creating a function")]
    MissingOptions,
    #[error("Cannot create that type of function")]
    FuncNotSupported,
    #[error("attribute value missing")]
    AttributeValueMissing,
    #[error("attribute prototype missing")]
    AttributePrototypeMissing,
    #[error("prop for value not found")]
    PropNotFound,
    #[error("func binding return value not found")]
    FuncBindingReturnValueMissing,
    #[error("func is not revertible")]
    FuncNotRevertible,
    #[error("func argument already exists for that name")]
    FuncArgumentAlreadyExists,
    #[error("func argument not found")]
    FuncArgNotFound,
    #[error("attribute prototype {0} has no prop_id")]
    AttributePrototypeMissingPropId(AttributePrototypeId),
    #[error("attribute prototype {0} schema_variant is missing")]
    AttributePrototypeMissingSchemaVariant(AttributePrototypeId),
    #[error("attribute prototype {0} schema is missing")]
    AttributePrototypeMissingSchema(AttributePrototypeId),
    #[error("attribute prototype {0} is missing its prop {1}")]
    AttributePrototypeMissingProp(AttributePrototypeId, PropId),
    #[error("attribute prototype {0} is missing argument {1}")]
    AttributePrototypeMissingArgument(AttributePrototypeId, AttributePrototypeArgumentId),
    #[error("attribute prototype argument {0} is internal provider id")]
    AttributePrototypeMissingInternalProviderId(AttributePrototypeArgumentId),
    #[error("func argument {0} missing attribute prototype argument for prototype {1}")]
    FuncArgumentMissingPrototypeArgument(FuncArgumentId, AttributePrototypeId),
    #[error("validation prototype {0} schema_variant is missing")]
    ValidationPrototypeMissingSchemaVariant(SchemaVariantId),
    #[error("validation prototype schema is missing")]
    ValidationPrototypeMissingSchema,
    #[error("component missing schema variant")]
    ComponentMissingSchemaVariant(ComponentId),
    #[error("schema variant missing schema")]
    SchemaVariantMissingSchema(SchemaVariantId),
    #[error("func {0} cannot be converted to frontend variant")]
    FuncCannotBeTurnedIntoVariant(FuncId),
    #[error("unexpected func variant ({0:?}) creating attribute func")]
    UnexpectedFuncVariantCreatingAttributeFunc(FuncVariant),
    #[error("cannot bind func to different prop kinds")]
    FuncDestinationPropKindMismatch,
    #[error("Function Execution Failed: {0}")]
    FuncExecutionFailed(String),
}

pub type FuncResult<T> = Result<T, FuncError>;

impl_default_error_into_response!(FuncError);

// Variants don't map 1:1 onto FuncBackendKind, since some JsAttribute functions
// are a special case (Qualification, CodeGeneration etc)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FuncVariant {
    Attribute,
    CodeGeneration,
    Confirmation,
    Command,
    Qualification,
    Validation,
}

impl From<FuncVariant> for FuncBackendKind {
    fn from(value: FuncVariant) -> Self {
        match value {
            FuncVariant::Command => FuncBackendKind::JsCommand,
            FuncVariant::Validation => FuncBackendKind::Validation,
            FuncVariant::Attribute
            | FuncVariant::CodeGeneration
            | FuncVariant::Confirmation
            | FuncVariant::Qualification => FuncBackendKind::JsAttribute,
        }
    }
}

impl TryFrom<&Func> for FuncVariant {
    type Error = FuncError;

    fn try_from(func: &Func) -> Result<Self, Self::Error> {
        match (func.backend_kind(), func.backend_response_type()) {
            (FuncBackendKind::JsAttribute, response_type) => match response_type {
                FuncBackendResponseType::CodeGeneration => Ok(FuncVariant::CodeGeneration),
                FuncBackendResponseType::Qualification => Ok(FuncVariant::Qualification),
                FuncBackendResponseType::Confirmation => Ok(FuncVariant::Confirmation),
                _ => Ok(FuncVariant::Attribute),
            },
            (FuncBackendKind::JsCommand, _) => Ok(FuncVariant::Command),
            (FuncBackendKind::JsValidation, _) => Ok(FuncVariant::Validation),
            _ => Err(FuncError::FuncCannotBeTurnedIntoVariant(*func.id())),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AttributePrototypeArgumentView {
    func_argument_id: FuncArgumentId,
    id: Option<AttributePrototypeArgumentId>,
    internal_provider_id: Option<InternalProviderId>,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AttributePrototypeView {
    id: AttributePrototypeId,
    component_id: Option<ComponentId>,
    prop_id: PropId,
    prototype_arguments: Vec<AttributePrototypeArgumentView>,
}

impl AttributePrototypeView {
    pub fn to_attribute_context(&self) -> FuncResult<AttributeContext> {
        Ok(match self.component_id {
            // Attribute contexts which set the defaults for a schema variant, have *only* the prop
            // set on the context
            None | Some(ComponentId::NONE) => AttributeContextBuilder::new()
                .set_prop_id(self.prop_id)
                .to_context()?,
            Some(component_id) => AttributeContextBuilder::new()
                .set_prop_id(self.prop_id)
                .set_component_id(component_id)
                .to_context()?,
        })
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ValidationPrototypeView {
    id: ValidationPrototypeId,
    schema_variant_id: SchemaVariantId,
    prop_id: PropId,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum FuncAssociations {
    #[serde(rename_all = "camelCase")]
    Qualification {
        schema_variant_ids: Vec<SchemaVariantId>,
        component_ids: Vec<ComponentId>,
    },
    #[serde(rename_all = "camelCase")]
    CodeGeneration {
        schema_variant_ids: Vec<SchemaVariantId>,
        component_ids: Vec<ComponentId>,
    },
    #[serde(rename_all = "camelCase")]
    Attribute {
        prototypes: Vec<AttributePrototypeView>,
        arguments: Vec<FuncArgumentView>,
    },
    #[serde(rename_all = "camelCase")]
    Confirmation {
        schema_variant_ids: Vec<SchemaVariantId>,
        component_ids: Vec<ComponentId>,
    },
    #[serde(rename_all = "camelCase")]
    Validation {
        prototypes: Vec<ValidationPrototypeView>,
    },
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FuncArgumentView {
    pub id: FuncArgumentId,
    pub name: String,
    pub kind: FuncArgumentKind,
    pub element_kind: Option<FuncArgumentKind>,
}

async fn is_func_revertible(ctx: &DalContext, func: &Func) -> FuncResult<bool> {
    // refetch to get updated visibility
    let is_in_change_set = match Func::get_by_id(ctx, func.id()).await? {
        Some(func) => func.visibility().in_change_set(),
        None => return Ok(false),
    };
    // Clone a new ctx vith head visibility
    let ctx = ctx.clone_with_head();
    let head_func = Func::get_by_id(&ctx, func.id()).await?;

    Ok(head_func.is_some() && is_in_change_set)
}

async fn prototype_view_for_attribute_prototype(
    ctx: &DalContext,
    func_id: FuncId,
    proto: &AttributePrototype,
) -> FuncResult<AttributePrototypeView> {
    let prop_id = if proto.context.prop_id().is_some() {
        proto.context.prop_id()
    } else {
        return Err(FuncError::AttributePrototypeMissingPropId(*proto.id()));
    };

    let component_id = if proto.context.component_id().is_some() {
        Some(proto.context.component_id())
    } else {
        None
    };

    let prototype_arguments =
        FuncArgument::list_for_func_with_prototype_arguments(ctx, func_id, *proto.id())
            .await?
            .iter()
            .map(
                |(func_arg, maybe_proto_arg)| AttributePrototypeArgumentView {
                    func_argument_id: *func_arg.id(),
                    id: maybe_proto_arg.as_ref().map(|proto_arg| *proto_arg.id()),
                    internal_provider_id: maybe_proto_arg
                        .as_ref()
                        .map(|proto_arg| proto_arg.internal_provider_id()),
                },
            )
            .collect();

    Ok(AttributePrototypeView {
        id: *proto.id(),
        prop_id,
        component_id,
        prototype_arguments,
    })
}

async fn attribute_prototypes_into_schema_variants_and_components(
    ctx: &DalContext,
    func_id: FuncId,
) -> FuncResult<(Vec<SchemaVariantId>, Vec<ComponentId>)> {
    let schema_variants_components =
        AttributePrototype::find_for_func_as_variant_and_component(ctx, func_id).await?;

    let mut schema_variant_ids = vec![];
    let mut component_ids = vec![];

    for (schema_variant_id, component_id) in schema_variants_components {
        if component_id == ComponentId::NONE {
            schema_variant_ids.push(schema_variant_id);
        } else {
            component_ids.push(component_id);
        }
    }

    Ok((schema_variant_ids, component_ids))
}

pub async fn get_func_view(ctx: &DalContext, func: &Func) -> FuncResult<GetFuncResponse> {
    let associations = match func.backend_kind() {
        FuncBackendKind::JsAttribute => match func.backend_response_type() {
            FuncBackendResponseType::CodeGeneration => {
                let (schema_variant_ids, component_ids) =
                    attribute_prototypes_into_schema_variants_and_components(ctx, *func.id())
                        .await?;

                Some(FuncAssociations::CodeGeneration {
                    schema_variant_ids,
                    component_ids,
                })
            }
            FuncBackendResponseType::Confirmation => {
                let (schema_variant_ids, component_ids) =
                    attribute_prototypes_into_schema_variants_and_components(ctx, *func.id())
                        .await?;

                Some(FuncAssociations::Confirmation {
                    schema_variant_ids,
                    component_ids,
                })
            }
            FuncBackendResponseType::Qualification => {
                let (schema_variant_ids, component_ids) =
                    attribute_prototypes_into_schema_variants_and_components(ctx, *func.id())
                        .await?;

                Some(FuncAssociations::Qualification {
                    schema_variant_ids,
                    component_ids,
                })
            }
            _ => {
                let protos = AttributePrototype::find_for_func(ctx, func.id()).await?;

                let mut prototype_views = vec![];
                for proto in &protos {
                    prototype_views.push(
                        prototype_view_for_attribute_prototype(ctx, *func.id(), proto).await?,
                    );
                }

                Some(FuncAssociations::Attribute {
                    prototypes: prototype_views,
                    arguments: FuncArgument::list_for_func(ctx, *func.id())
                        .await?
                        .iter()
                        .map(|arg| FuncArgumentView {
                            id: *arg.id(),
                            name: arg.name().to_owned(),
                            kind: arg.kind().to_owned(),
                            element_kind: arg.element_kind().cloned(),
                        })
                        .collect(),
                })
            }
        },
        FuncBackendKind::JsValidation => {
            let protos = ValidationPrototype::list_for_func(ctx, *func.id()).await?;

            Some(FuncAssociations::Validation {
                prototypes: protos
                    .iter()
                    .map(|proto| ValidationPrototypeView {
                        id: *proto.id(),
                        schema_variant_id: proto.context().schema_variant_id(),
                        prop_id: proto.context().prop_id(),
                    })
                    .collect(),
            })
        }

        _ => None,
    };

    let is_revertible = is_func_revertible(ctx, func).await?;

    Ok(GetFuncResponse {
        id: func.id().to_owned(),
        handler: func.handler().map(|h| h.to_owned()),
        variant: func.try_into()?,
        name: func
            .display_name()
            .unwrap_or_else(|| func.name())
            .to_owned(),
        description: func.description().map(|d| d.to_owned()),
        code: func.code_plaintext()?,
        is_builtin: func.builtin(),
        is_revertible,
        associations,
    })
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/list_funcs", get(list_funcs::list_funcs))
        .route("/get_func", get(get_func::get_func))
        .route("/create_func", post(create_func::create_func))
        .route("/save_func", post(save_func::save_func))
        .route("/exec_func", post(exec_func::exec_func))
        .route("/revert_func", post(revert_func::revert_func))
        .route(
            "/list_input_sources",
            get(list_input_sources::list_input_sources),
        )
}
