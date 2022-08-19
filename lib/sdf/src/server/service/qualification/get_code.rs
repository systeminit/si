// deprecated 
use axum::{extract::Query, Json};
use dal::{
    Func, QualificationPrototype, QualificationPrototypeId, StandardModel, SystemId, Visibility,
};
use serde::{Deserialize, Serialize};

use super::{QualificationError, QualificationResult};
use crate::server::extract::{AccessBuilder, HandlerContext};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetCodeRequest {
    pub prototype_id: QualificationPrototypeId,
    pub system_id: Option<SystemId>,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct QualificationPrototypeView {
    title: String,
    description: Option<String>,
    link: Option<String>,
    is_component_specific: bool,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetCodeResponse {
    code: String,
    prototype: QualificationPrototypeView,
}

pub async fn get_code(
    HandlerContext(builder, mut txns): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<GetCodeRequest>,
) -> QualificationResult<Json<GetCodeResponse>> {
    let txns = txns.start().await?;
    let ctx = builder.build(request_ctx.build(request.visibility), &txns);

    // TODO: actually use the system to filter qualifications
    let _system_id = request.system_id.unwrap_or(SystemId::NONE);

    let prototype = QualificationPrototype::get_by_id(&ctx, &request.prototype_id)
        .await?
        .ok_or(QualificationError::PrototypeNotFound(request.prototype_id))?;
    let mut func = Func::get_by_id(&ctx, &prototype.func_id())
        .await?
        .ok_or(QualificationError::FuncNotFound)?;

    // We can't edit universal stuff created by another tenancy
    let is_in_our_tenancy = ctx
        .write_tenancy()
        .check(
            ctx.pg_txn(),
            &func.tenancy().clone_into_read_tenancy(&ctx).await?,
        )
        .await?;

    // Must be exactly in our visibility for us to edit
    let is_in_our_visibility = func.visibility().change_set_pk == ctx.visibility().change_set_pk;

    // Clone the qualification into our tenancy + visibility
    if !is_in_our_tenancy || !is_in_our_visibility {
        let mut new_func = Func::new(
            &ctx,
            func.name().to_owned(),
            *func.backend_kind(),
            *func.backend_response_type(),
        )
        .await?;
        new_func.set_id(&ctx, func.id()).await?;
        new_func.set_handler(&ctx, func.handler()).await?;
        new_func.set_code_base64(&ctx, func.code_base64()).await?;
        func = new_func;
    }

    let code = String::from_utf8(base64::decode(
        func.code_base64()
            .ok_or_else(|| QualificationError::FuncCodeNotFound(*func.id()))?,
    )?)?;

    txns.commit().await?;
    Ok(Json(GetCodeResponse {
        code,
        prototype: QualificationPrototypeView {
            title: prototype.title().into(),
            link: prototype.link().map(Into::into),
            description: prototype.description().map(Into::into),
            is_component_specific: !prototype.context().component_id().is_none(),
        },
    }))
}
