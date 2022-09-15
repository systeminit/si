use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};

use dal::{
    attribute::context::AttributeContextBuilderError, context, AttributeContext,
    AttributeContextError, AttributePrototype, AttributePrototypeError, AttributeValue,
    AttributeValueError, AttributeValueId, ComponentError, DalContext, DalContextBuilder, Func,
    FuncBackendKind, FuncBinding, FuncBindingError, Prop, PropError, PropKind,
    QualificationPrototypeError, ReadTenancyError, StandardModel, StandardModelError, Transactions,
    TransactionsError, Visibility, WriteTenancyError, WsEventError,
};

use dal::attribute::value::dependent_update::collection::AttributeValueDependentCollectionHarness;
use dal::func::backend::js_attribute::FuncBackendJsAttributeArgs;
use dal::func::binding_return_value::FuncBindingReturnValueError;
use dal::job::definition::DependentValuesUpdate;
use thiserror::Error;

pub mod create_func;
pub mod exec_func;
pub mod get_func;
pub mod list_funcs;
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
    #[error("func is not revertable")]
    FuncNotRevertable,
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

async fn is_func_revertable<'t>(
    builder: DalContextBuilder,
    txns: &'t Transactions<'_>,
    request_ctx: context::AccessBuilder,
    func: &Func,
) -> FuncResult<bool> {
    let request_ctx = request_ctx.build(Visibility {
        change_set_pk: (-1).into(),
        deleted_at: None,
    });
    let head_ctx = builder.build(request_ctx, txns);

    let head_func = dbg!(Func::get_by_id(&head_ctx, func.id()).await?);

    Ok(head_func.is_some() && func.visibility().in_change_set())
}

// Note: much of this function will be replaced by the "update just this value" work
async fn update_attribute_value_by_func_for_context(
    ctx: &DalContext<'_, '_, '_>,
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
        component: veritech::ResolverFunctionComponent {
            data: veritech::ComponentView::default(),
            parents: Vec::new(), // do we need to fill this in with parent data?
        },
    };

    let (func_binding, mut fbrv) = if func_is_new {
        FuncBinding::find_or_create_and_execute(ctx, serde_json::to_value(args)?, *func.id())
            .await?
    } else {
        let func_binding = FuncBinding::new(
            ctx,
            serde_json::to_value(args)?,
            *func.id(),
            FuncBackendKind::JsAttribute,
        )
        .await?;
        let fbrv = func_binding.execute(ctx).await?;

        (func_binding, fbrv)
    };

    attribute_value
        .set_func_binding_id(ctx, *func_binding.id())
        .await?;
    attribute_value
        .set_func_binding_return_value_id(ctx, *fbrv.id())
        .await?;

    // If the value we just updated was for a Prop, we might have run a function that
    // generates a deep data structure. If the Prop is an Array/Map/Object, then the
    // value should be an empty Array/Map/Object, while the unprocessed value contains
    // the deep data structure.
    if attribute_value
        .context
        .is_least_specific_field_kind_prop()?
    {
        let processed_value = match fbrv.unprocessed_value().cloned() {
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
        fbrv.set_value(ctx, processed_value).await?;
    };

    AttributePrototype::update_for_context(
        ctx,
        *prototype.id(),
        context,
        *func.id(),
        *func_binding.id(),
        *fbrv.id(),
        parent_value_id,
        Some(value_id),
    )
    .await?;

    let dependent_attribute_values =
        AttributeValueDependentCollectionHarness::collect(ctx, attribute_value.context).await?;
    for dependent_attribute_value in dependent_attribute_values {
        ctx.enqueue_job(DependentValuesUpdate::new(
            ctx,
            *dependent_attribute_value.id(),
        ))
        .await;
    }

    Ok(())
}

pub fn routes() -> Router {
    Router::new()
        .route("/list_funcs", get(list_funcs::list_funcs))
        .route("/get_func", get(get_func::get_func))
        .route("/create_func", post(create_func::create_func))
        .route("/save_func", post(save_func::save_func))
        .route("/exec_func", post(exec_func::exec_func))
        .route("/revert_func", post(revert_func::revert_func))
}
