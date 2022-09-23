use super::{FuncAssociations, FuncError, FuncResult};
use crate::server::extract::{AccessBuilder, HandlerContext};
use axum::Json;
use dal::{
    qualification_prototype::QualificationPrototypeContextField, Func, FuncBackendKind, FuncId,
    QualificationPrototype, StandardModel, Visibility, WsEvent,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SaveFuncRequest {
    pub id: FuncId,
    pub handler: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub code: Option<String>,
    pub associations: Option<FuncAssociations>,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SaveFuncResponse {
    pub success: bool,
    pub is_revertable: bool,
}

pub async fn save_func(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<SaveFuncRequest>,
) -> FuncResult<Json<SaveFuncResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let mut func = Func::get_by_id(&ctx, &request.id)
        .await?
        .ok_or(FuncError::FuncNotFound)?;

    // Don't modify builtins, or for other tenancies
    if !ctx.check_tenancy(&func).await? {
        return Err(FuncError::NotWritable);
    }

    func.set_display_name(&ctx, Some(request.name)).await?;
    func.set_description(&ctx, request.description).await?;
    func.set_handler(&ctx, request.handler).await?;
    func.set_code_plaintext(&ctx, request.code.as_deref())
        .await?;

    match func.backend_kind() {
        FuncBackendKind::JsQualification => {
            let mut associations: Vec<QualificationPrototypeContextField> = vec![];
            if let Some(FuncAssociations::Qualification {
                schema_variant_ids,
                component_ids,
            }) = request.associations
            {
                associations.append(&mut schema_variant_ids.iter().map(|f| (*f).into()).collect());
                associations.append(&mut component_ids.iter().map(|f| (*f).into()).collect());
            };

            let _ = QualificationPrototype::associate_prototypes_with_func_and_objects(
                &ctx,
                func.id(),
                &associations,
            )
            .await?;
        }
        FuncBackendKind::JsAttribute => {
            // here we need to update the attribute prototypes based on the chosen props and contexts
        }
        _ => {}
    }

    let is_revertable = super::is_func_revertable(&ctx, &func).await?;

    WsEvent::change_set_written(&ctx).publish(&ctx).await?;

    ctx.commit().await?;

    Ok(Json(SaveFuncResponse {
        success: true,
        is_revertable,
    }))
}
