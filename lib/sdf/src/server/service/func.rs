use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::service::func::get_func::GetFuncResponse;
use dal::func::argument::FuncArgument;
use dal::{
    attribute::context::AttributeContextBuilder,
    attribute::context::AttributeContextBuilderError,
    func::{
        argument::{FuncArgumentError, FuncArgumentId, FuncArgumentKind},
        backend::js_attribute::FuncBackendJsAttributeArgs,
        binding_return_value::FuncBindingReturnValueError,
    },
    job::definition::DependentValuesUpdate,
    prop_tree::PropTreeError,
    prototype_context::{HasPrototypeContext, PrototypeContext, PrototypeContextError},
    schema::variant::SchemaVariantError,
    AttributeContext, AttributeContextError, AttributePrototype, AttributePrototypeArgumentError,
    AttributePrototypeArgumentId, AttributePrototypeError, AttributePrototypeId, AttributeValue,
    AttributeValueError, AttributeValueId, CodeGenerationPrototype, CodeGenerationPrototypeError,
    CodeLanguage, ComponentError, ComponentId, ConfirmationPrototype, ConfirmationPrototypeError,
    DalContext, Func, FuncBackendKind, FuncBinding, FuncBindingError, FuncId,
    InternalProviderError, InternalProviderId, Prop, PropError, PropId, PropKind,
    PrototypeListForFunc, PrototypeListForFuncError, QualificationPrototype,
    QualificationPrototypeError, ReadTenancyError, SchemaVariant, SchemaVariantId, StandardModel,
    StandardModelError, TransactionsError, Visibility, WriteTenancyError, WsEventError,
};

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
    Pg(#[from] si_data::PgError),
    #[error(transparent)]
    PgPool(#[from] si_data::PgPoolError),
    #[error("read tenancy error: {0}")]
    ReadTenancy(#[from] ReadTenancyError),
    #[error("write tenancy error: {0}")]
    WriteTenancy(#[from] WriteTenancyError),
    #[error(transparent)]
    StandardModel(#[from] StandardModelError),
    #[error(transparent)]
    ContextTransaction(#[from] TransactionsError),
    #[error(transparent)]
    Func(#[from] dal::FuncError),
    #[error("could not publish websocket event: {0}")]
    WsEvent(#[from] WsEventError),
    #[error(transparent)]
    QualificationPrototype(#[from] QualificationPrototypeError),
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
    #[error("code generation prototype error: {0}")]
    CodeGenerationPrototype(#[from] CodeGenerationPrototypeError),
    #[error("prototype context error: {0}")]
    PrototypeContext(#[from] PrototypeContextError),
    #[error("prototype list for func error: {0}")]
    PrototypeListForFunc(#[from] PrototypeListForFuncError),
    #[error("confirmation prototype error: {0}")]
    ConfirmationPrototype(#[from] ConfirmationPrototypeError),

    #[error("Function not found")]
    FuncNotFound,
    #[error("Function is read-only")]
    NotWritable,
    #[error("Missing required options for creating a function")]
    MissingOptions,
    #[error("Cannot create that type of function")]
    FuncNotSupported,
    #[error("we don't know what to do if the prototype is in the universal tenancy")]
    UniversalError,
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
    #[error("attribute prototype {0} schema_variant is missingj")]
    AttributePrototypeMissingSchemaVariant(AttributePrototypeId),
    #[error("attribute prototype {0} schema is missing")]
    AttributePrototypeMissingSchema(AttributePrototypeId),
    #[error("attribute prototype {0} is missing its prop {1}")]
    AttributePrototypeMissingProp(AttributePrototypeId, PropId),
    #[error("attribute prototype {0} is missing argument {1}")]
    AttributePrototypeMissingArgument(AttributePrototypeId, AttributePrototypeArgumentId),
}

pub type FuncResult<T> = Result<T, FuncError>;

impl IntoResponse for FuncError {
    fn into_response(self) -> Response {
        let (status, error_message) = (StatusCode::INTERNAL_SERVER_ERROR, self.to_string());

        let body = Json(
            serde_json::json!({ "error": { "message": error_message, "code": 42, "statusCode": status.as_u16() } }),
        );

        (status, body).into_response()
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
    schema_variant_id: SchemaVariantId,
    component_id: Option<ComponentId>,
    prop_id: PropId,
    prototype_arguments: Vec<AttributePrototypeArgumentView>,
}

impl AttributePrototypeView {
    pub async fn into_context(&self, ctx: &DalContext) -> FuncResult<AttributeContext> {
        // should context need schema id if it has schema variant id, since a schema variant
        // can only belong to one schema?
        let schema_id = *SchemaVariant::get_by_id(ctx, &self.schema_variant_id)
            .await?
            .ok_or(FuncError::AttributePrototypeMissingSchemaVariant(self.id))?
            .schema(ctx)
            .await?
            .ok_or(FuncError::AttributePrototypeMissingSchema(self.id))?
            .id();

        Ok(AttributeContextBuilder::new()
            .set_prop_id(self.prop_id)
            .set_schema_id(schema_id)
            .set_schema_variant_id(self.schema_variant_id)
            .set_component_id(self.component_id.unwrap_or(ComponentId::NONE))
            .to_context()?)
    }
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
        format: CodeLanguage,
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
    // Clone a new ctx vith head visibility
    let ctx = ctx.clone_with_new_visibility(Visibility::new_head(false));
    let head_func = Func::get_by_id(&ctx, func.id()).await?;

    Ok(head_func.is_some() && func.visibility().in_change_set())
}

// Note: much of this function will be replaced by the "update just this value" work
async fn update_attribute_value_by_func_for_context(
    ctx: &DalContext,
    value_id: AttributeValueId,
    parent_value_id: Option<AttributeValueId>,
    func: &Func,
    context: AttributeContext,
    func_is_new: bool,
) -> FuncResult<()> {
    // grab provided attribute value
    let attribute_value = AttributeValue::get_by_id(ctx, &value_id)
        .await?
        .ok_or(FuncError::AttributeValueMissing)?;

    // if context does not match, we need to create a new, unset value in the provided context
    let (mut attribute_value, value_id) = if attribute_value.context != context {
        let (_, value_id) =
            AttributeValue::update_for_context(ctx, value_id, parent_value_id, context, None, None)
                .await?;

        let attribute_value = AttributeValue::get_by_id(ctx, &value_id)
            .await?
            .ok_or(FuncError::AttributeValueMissing)?;

        (attribute_value, value_id)
    } else {
        (attribute_value, value_id)
    };

    let prototype = attribute_value
        .attribute_prototype(ctx)
        .await?
        .ok_or(FuncError::AttributePrototypeMissing)?;

    // These are the wrong args but what we need right now before optimizations are merged in
    let args = FuncBackendJsAttributeArgs {
        component: veritech_client::ResolverFunctionComponent {
            data: veritech_client::ComponentView::default(),
            parents: Vec::new(), // do we need to fill this in with parent data?
        },
    };

    let (func_binding, mut func_binding_return_value) = if func_is_new {
        let (func_binding, func_binding_return_value, _) =
            FuncBinding::find_or_create_and_execute(ctx, serde_json::to_value(args)?, *func.id())
                .await?;
        (func_binding, func_binding_return_value)
    } else {
        let func_binding = FuncBinding::new(
            ctx,
            serde_json::to_value(args)?,
            *func.id(),
            FuncBackendKind::JsAttribute,
        )
        .await?;
        let func_binding_return_value = func_binding.execute(ctx).await?;
        (func_binding, func_binding_return_value)
    };

    attribute_value
        .set_func_binding_id(ctx, *func_binding.id())
        .await?;
    attribute_value
        .set_func_binding_return_value_id(ctx, *func_binding_return_value.id())
        .await?;

    // If the value we just updated was for a Prop, we might have run a function that
    // generates a deep data structure. If the Prop is an Array/Map/Object, then the
    // value should be an empty Array/Map/Object, while the unprocessed value contains
    // the deep data structure.
    if attribute_value
        .context
        .is_least_specific_field_kind_prop()?
    {
        let processed_value = match func_binding_return_value.unprocessed_value().cloned() {
            Some(unprocessed_value) => {
                let prop = Prop::get_by_id(ctx, &attribute_value.context.prop_id())
                    .await?
                    .ok_or(FuncError::PropNotFound)?;

                match prop.kind() {
                    PropKind::Object | PropKind::Map => Some(serde_json::json!({})),
                    PropKind::Array => Some(serde_json::json!([])),
                    _ => Some(unprocessed_value),
                }
            }
            None => None,
        };
        func_binding_return_value
            .set_value(ctx, processed_value)
            .await?;
    };

    AttributePrototype::update_for_context(
        ctx,
        *prototype.id(),
        context,
        *func.id(),
        *func_binding.id(),
        *func_binding_return_value.id(),
        parent_value_id,
        Some(value_id),
    )
    .await?;

    ctx.enqueue_job(DependentValuesUpdate::new(ctx, *attribute_value.id()))
        .await;

    Ok(())
}
async fn prototype_view_for_prototype(
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

    let schema_variant_id = if proto.context.schema_variant_id().is_none() {
        let prop = Prop::get_by_id(ctx, &prop_id)
            .await?
            .ok_or_else(|| FuncError::AttributePrototypeMissingProp(*proto.id(), prop_id))?;

        match prop.schema_variants(ctx).await?.pop() {
            Some(schema_variant) => *schema_variant.id(),
            None => {
                return Err(FuncError::AttributePrototypeMissingSchemaVariant(
                    *proto.id(),
                ))
            }
        }
    } else {
        proto.context.schema_variant_id()
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
        schema_variant_id,
        prototype_arguments,
    })
}

fn prototype_context_into_schema_variants_and_components<P, C>(
    prototype_contexts: &[P],
) -> (Vec<SchemaVariantId>, Vec<ComponentId>)
where
    P: HasPrototypeContext<C>,
    C: PrototypeContext,
{
    let mut schema_variant_ids = vec![];
    let mut component_ids = vec![];

    for context in prototype_contexts {
        if context.context().component_id().is_some() {
            component_ids.push(context.context().component_id())
        } else if context.context().schema_variant_id().is_some() {
            schema_variant_ids.push(context.context().schema_variant_id());
        }
    }

    (schema_variant_ids, component_ids)
}

pub async fn get_func_view(ctx: &DalContext, func: &Func) -> FuncResult<GetFuncResponse> {
    let associations = match func.backend_kind() {
        FuncBackendKind::JsAttribute => {
            let protos = AttributePrototype::find_for_func(ctx, func.id()).await?;
            let mut prototype_views = vec![];

            for proto in &protos {
                prototype_views.push(prototype_view_for_prototype(ctx, *func.id(), proto).await?);
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
        FuncBackendKind::JsCodeGeneration => {
            let protos = CodeGenerationPrototype::list_for_func(ctx, *func.id()).await?;

            let format = match protos.get(0) {
                Some(proto) => *proto.format(),
                None => CodeLanguage::Unknown,
            };

            let (schema_variant_ids, component_ids) =
                prototype_context_into_schema_variants_and_components(&protos);

            Some(FuncAssociations::CodeGeneration {
                schema_variant_ids,
                component_ids,
                format,
            })
        }
        FuncBackendKind::JsConfirmation => {
            let protos = ConfirmationPrototype::list_for_func(ctx, *func.id()).await?;

            let (schema_variant_ids, component_ids) =
                prototype_context_into_schema_variants_and_components(&protos);

            Some(FuncAssociations::Confirmation {
                schema_variant_ids,
                component_ids,
            })
        }
        FuncBackendKind::JsQualification => {
            let protos = QualificationPrototype::list_for_func(ctx, *func.id()).await?;

            let (schema_variant_ids, component_ids) =
                prototype_context_into_schema_variants_and_components(&protos);

            Some(FuncAssociations::Qualification {
                schema_variant_ids,
                component_ids,
            })
        }

        _ => None,
    };

    let is_revertible = is_func_revertible(ctx, func).await?;

    Ok(GetFuncResponse {
        id: func.id().to_owned(),
        handler: func.handler().map(|h| h.to_owned()),
        kind: func.backend_kind().to_owned(),
        name: func
            .display_name()
            .unwrap_or_else(|| func.name())
            .to_owned(),
        description: func.description().map(|d| d.to_owned()),
        code: func.code_plaintext()?,
        is_builtin: func.is_builtin(),
        is_revertible,
        associations,
    })
}

pub fn routes() -> Router {
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
