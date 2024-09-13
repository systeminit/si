use axum::{
    extract::{Host, OriginalUri, Path},
    response::IntoResponse,
    Json,
};
use dal::{
    func::binding::{attribute::AttributeBinding, EventualParent},
    ChangeSet, ChangeSetId, Func, FuncId, SchemaVariant, WorkspacePk, WsEvent,
};
use si_frontend_types as frontend_types;

use crate::{
    extract::{AccessBuilder, HandlerContext, PosthogClient},
    service::v2::func::{FuncAPIError, FuncAPIResult},
    track,
};

pub async fn reset_attribute_binding(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    Path((_workspace_pk, change_set_id, func_id)): Path<(WorkspacePk, ChangeSetId, FuncId)>,
    Json(request): Json<frontend_types::FuncBindings>,
) -> FuncAPIResult<impl IntoResponse> {
    let mut ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;
    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;
    let func = Func::get_by_id_or_error(&ctx, func_id).await?;

    if func.kind != dal::func::FuncKind::Attribute {
        return Err(FuncAPIError::WrongFunctionKindForBinding);
    }

    for binding in request.bindings {
        let frontend_types::FuncBinding::Attribute {
            attribute_prototype_id: Some(attribute_prototype_id),
            ..
        } = binding
        else {
            return Err(FuncAPIError::MissingPrototypeId);
        };

        let eventual_parent = AttributeBinding::reset_attribute_binding(
            &ctx,
            attribute_prototype_id.into_raw_id().into(),
        )
        .await?;

        if let EventualParent::SchemaVariant(schema_variant_id) = eventual_parent {
            let schema =
                SchemaVariant::schema_id_for_schema_variant_id(&ctx, schema_variant_id).await?;
            let schema_variant = SchemaVariant::get_by_id_or_error(&ctx, schema_variant_id).await?;

            WsEvent::schema_variant_updated(&ctx, schema, schema_variant)
                .await?
                .publish_on_commit(&ctx)
                .await?;
        }
    }

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        &host_name,
        "reset_attribute_binding",
        serde_json::json!({
            "how": "/func/reset_attribute_binding",
            "func_id": func_id,
            "func_name": func.name.clone(),
            "func_kind": func.kind.clone(),
        }),
    );
    let binding = Func::get_by_id_or_error(&ctx, func_id)
        .await?
        .into_frontend_type(&ctx)
        .await?
        .bindings;
    let func_summary = Func::get_by_id_or_error(&ctx, func_id)
        .await?
        .into_frontend_type(&ctx)
        .await?;
    WsEvent::func_updated(&ctx, func_summary.clone())
        .await?
        .publish_on_commit(&ctx)
        .await?;

    ctx.commit().await?;

    let mut response = axum::response::Response::builder();
    response = response.header("Content-Type", "application/json");
    if let Some(force_change_set_id) = force_change_set_id {
        response = response.header("force_change_set_id", force_change_set_id.to_string());
    }
    Ok(response.body(serde_json::to_string(&binding)?)?)
}
