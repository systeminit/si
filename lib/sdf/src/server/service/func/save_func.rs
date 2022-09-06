use super::{FuncError, FuncResult};
use crate::server::extract::{AccessBuilder, HandlerContext};
use axum::Json;
use dal::{
    qualification_prototype::QualificationPrototypeContextField, AttributePrototype, ComponentId,
    DalContext, Func, FuncBackendKind, FuncId, QualificationPrototype, SchemaVariantId,
    StandardModel, Visibility, WsEvent,
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

async fn update_values_for_func(ctx: &DalContext<'_, '_, '_>, func: &Func) -> FuncResult<()> {
    let prototypes = AttributePrototype::find_for_func(ctx, func.id()).await?;
    for proto in prototypes {
        for value in proto.attribute_values(ctx).await? {
            let maybe_parent_value_id = value
                .parent_attribute_value(ctx)
                .await?
                .map(|pav| *pav.id());

            super::update_attribute_value_by_func_for_context(
                ctx,
                *value.id(),
                maybe_parent_value_id,
                func,
                value.context,
                false,
            )
            .await?;
        }
    }

    Ok(())
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

    func.set_display_name(&ctx, Some(request.name)).await?;
    func.set_description(&ctx, request.description).await?;
    func.set_handler(&ctx, request.handler).await?;
    func.set_code_plaintext(&ctx, request.code.as_deref())
        .await?;

    match func.backend_kind() {
        FuncBackendKind::JsQualification => {
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
        }
        // Rexecute the function for every prototype it is defined on
        FuncBackendKind::JsAttribute => {
            update_values_for_func(&ctx, &func).await?;
        }
        _ => {}
    }

    WsEvent::change_set_written(&ctx).publish(&ctx).await?;

    txns.commit().await?;

    Ok(Json(SaveFuncResponse { success: true }))
}
