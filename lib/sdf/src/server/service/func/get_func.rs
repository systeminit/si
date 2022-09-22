use super::{FuncAssociations, FuncError, FuncResult};
use crate::server::extract::{AccessBuilder, HandlerContext};
use axum::{extract::Query, Json};
use dal::{
     Func, FuncBackendKind, FuncId,
     QualificationPrototype, StandardModel, Visibility,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetFuncRequest {
    pub id: FuncId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetFuncResponse {
    pub id: FuncId,
    pub handler: Option<String>,
    pub kind: FuncBackendKind,
    pub name: String,
    pub description: Option<String>,
    pub code: Option<String>,
    pub is_builtin: bool,
    pub is_revertable: bool,
    pub associations: Option<FuncAssociations>,
}

pub async fn get_func(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<GetFuncRequest>,
) -> FuncResult<Json<GetFuncResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let func = Func::get_by_id(&ctx, &request.id)
        .await?
        .ok_or(FuncError::FuncNotFound)?;

    let associations = match func.backend_kind() {
        FuncBackendKind::JsQualification => {
            let protos = QualificationPrototype::find_for_func(&ctx, func.id()).await?;

            let mut schema_variant_ids = vec![];
            let mut component_ids = vec![];

            for proto in protos {
                if proto.context().schema_variant_id().is_some() {
                    schema_variant_ids.push(proto.context().schema_variant_id());
                } else if proto.context().component_id().is_some() {
                    component_ids.push(proto.context().component_id());
                }
            }

            Some(FuncAssociations::Qualification {
                schema_variant_ids,
                component_ids,
            })
        }
        FuncBackendKind::JsAttribute => None,
        _ => None,
    };

    let is_revertable = super::is_func_revertable(&ctx, &func).await?;

    Ok(Json(GetFuncResponse {
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
        is_revertable,
        associations,
    }))
}
