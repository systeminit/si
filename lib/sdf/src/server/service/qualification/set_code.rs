use axum::Json;
use dal::{
    Component, ComponentAsyncTasks, Func, QualificationPrototype, QualificationPrototypeId, Schema,
    StandardModel, SystemId, Visibility,
};
use serde::{Deserialize, Serialize};
use telemetry::prelude::*;

use super::{QualificationError, QualificationResult};
use crate::server::extract::{AccessBuilder, HandlerContext};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SetCodeRequest {
    pub prototype_id: QualificationPrototypeId,
    pub code: String,
    pub system_id: Option<SystemId>,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SetCodeResponse {
    success: bool,
}

pub async fn set_code(
    HandlerContext(builder, mut txns): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<SetCodeRequest>,
) -> QualificationResult<Json<SetCodeResponse>> {
    let txns = txns.start().await?;
    let ctx = builder.build(request_ctx.clone().build(request.visibility), &txns);

    // TODO: actually use the system to filter qualifications
    let system_id = request.system_id.unwrap_or(SystemId::NONE);

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
    let is_in_our_visibility = func.visibility().edit_session_pk
        == ctx.visibility().edit_session_pk
        && func.visibility().change_set_pk == ctx.visibility().change_set_pk;

    // Clone the qualification into our tenancy + visibility
    if !is_in_our_tenancy || !is_in_our_visibility {
        return Err(QualificationError::NotWritable);
    }

    func.set_code_base64(&ctx, Some(base64::encode(request.code)))
        .await?;

    let mut components = Vec::new();
    if prototype.context().component_id().is_some() {
        let component_id = prototype.context().component_id();
        let component = Component::get_by_id(&ctx, &component_id)
            .await?
            .ok_or(QualificationError::ComponentNotFound(component_id))?;
        components.push(component);
    } else if prototype.context().schema_variant_id().is_some() {
        let variant_id = prototype.context().schema_variant_id();
        components.extend(Component::list_for_schema_variant(&ctx, variant_id).await?);
    } else if prototype.context().schema_id().is_some() {
        let schema_id = prototype.context().schema_id();
        let schema = Schema::get_by_id(&ctx, &schema_id)
            .await?
            .ok_or(QualificationError::SchemaNotFound(schema_id))?;
        for variant in schema.variants(&ctx).await? {
            components.extend(Component::list_for_schema_variant(&ctx, *variant.id()).await?);
        }
    }

    txns.commit().await?;

    for component in components {
        let async_tasks = ComponentAsyncTasks::new(component, system_id);
        let request_ctx = request_ctx.clone();
        let builder = builder.clone();
        tokio::task::spawn(async move {
            if let Err(err) = async_tasks
                .run(request_ctx, request.visibility, &builder)
                .await
            {
                error!("Component async qualification check failed: {err}");
            }
        });
    }

    Ok(Json(SetCodeResponse { success: true }))
}
