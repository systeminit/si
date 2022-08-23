use super::{FuncError, FuncResult};
use crate::server::extract::{AccessBuilder, HandlerContext};
use axum::Json;
use dal::{
    qualification_prototype::QualificationPrototypeContextField, ComponentId, Func,
    FuncBackendKind, FuncId, QualificationPrototype, SchemaVariantId, StandardModel, Visibility,
    WsEvent,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SaveFuncRequest {
    pub id: FuncId,
    pub handler: Option<String>,
    pub kind: FuncBackendKind,
    pub name: String,
    pub code: Option<String>,
    pub schema_variants: Vec<SchemaVariantId>,
    pub components: Vec<ComponentId>,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SaveFuncResponse {
    pub success: bool,
}

pub async fn save_func(
    HandlerContext(builder, mut txns): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<SaveFuncRequest>,
) -> FuncResult<Json<SaveFuncResponse>> {
    let txns = txns.start().await?;
    let ctx = builder.build(request_ctx.build(request.visibility), &txns);

    let mut func = Func::get_by_id(&ctx, &request.id)
        .await?
        .ok_or(FuncError::FuncNotFound)?;

    // Don't modify builtins or objects in another tenancy/visibility
    if !ctx
        .check_standard_model_tenancy_and_visibility_match(&func)
        .await?
    {
        return Err(FuncError::NotWritable);
    }

    func.set_name(&ctx, request.name).await?;
    func.set_handler(&ctx, request.handler).await?;
    func.set_backend_kind(&ctx, request.kind).await?;
    func.set_code_plaintext(&ctx, request.code.as_deref())
        .await?;
    func.set_backend_response_type(&ctx, request.kind).await?;

    let mut associations: Vec<QualificationPrototypeContextField> = vec![];
    associations.append(
        &mut request
            .schema_variants
            .iter()
            .map(|f| (*f).into())
            .collect(),
    );
    associations.append(&mut request.components.iter().map(|f| (*f).into()).collect());

    if !associations.is_empty() {
        let _ = QualificationPrototype::associate_prototypes_with_func_and_objects(
            &ctx,
            func.id(),
            &associations,
        )
        .await?;
    }

    WsEvent::change_set_written(&ctx).publish(&ctx).await?;

    txns.commit().await?;

    Ok(Json(SaveFuncResponse { success: true }))
}
