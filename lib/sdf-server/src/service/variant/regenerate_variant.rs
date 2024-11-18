use std::collections::HashSet;

use axum::{
    extract::{Host, OriginalUri},
    Json,
};
use dal::{
    schema::variant::authoring::VariantAuthoringClient, AttributePrototype, ChangeSet, DalContext,
    Func, FuncId, Prop, SchemaVariant, SchemaVariantId, Visibility, WsEvent,
};
use serde::{Deserialize, Serialize};

use crate::{
    extract::{AccessBuilder, HandlerContext, PosthogClient},
    service::{force_change_set_response::ForceChangeSetResponse, variant::SchemaVariantResult},
    track,
};

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RegenerateVariantRequest {
    // We need to get the updated data here, to ensure we create the prop the user is seeing
    pub variant: si_frontend_types::SchemaVariant,
    pub code: Option<String>,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RegenerateVariantResponse {
    pub schema_variant_id: SchemaVariantId,
}

pub async fn regenerate_variant(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    Json(RegenerateVariantRequest {
        variant,
        code,
        visibility,
    }): Json<RegenerateVariantRequest>,
) -> SchemaVariantResult<ForceChangeSetResponse<RegenerateVariantResponse>> {
    let mut ctx = builder.build(request_ctx.build(visibility)).await?;

    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    let schema_variant_id = variant.schema_variant_id.into();

    VariantAuthoringClient::save_variant_content(
        &ctx,
        schema_variant_id,
        &variant.schema_name,
        &variant.display_name,
        &variant.category,
        variant.description,
        variant.link,
        &variant.color,
        variant.component_type.into(),
        code,
    )
    .await?;

    let updated_schema_variant_id =
        VariantAuthoringClient::regenerate_variant(&ctx, schema_variant_id).await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        &host_name,
        "update_variant",
        serde_json::json!({
            "old_schema_variant_id": schema_variant_id,
            "new_schema_variant_id": updated_schema_variant_id,
        }),
    );

    let schema =
        SchemaVariant::schema_id_for_schema_variant_id(&ctx, updated_schema_variant_id).await?;
    let updated_schema_variant =
        SchemaVariant::get_by_id_or_error(&ctx, updated_schema_variant_id).await?;

    if schema_variant_id == updated_schema_variant_id {
        // if old == new -> send updated for it

        WsEvent::schema_variant_updated(&ctx, schema, updated_schema_variant)
            .await?
            .publish_on_commit(&ctx)
            .await?;
    } else {
        // send that the old one is deleted and new one is created
        // (note: we auto upgrade components on regenerate now so this variant is actually eligible for GC)
        // let's pretend it was

        WsEvent::schema_variant_replaced(&ctx, schema, schema_variant_id, updated_schema_variant)
            .await?
            .publish_on_commit(&ctx)
            .await?;
    }

    // Send FuncUpdated for all functions bound to the schema variant, since there may be new props/sockets (and since
    // regenerated props/sockets now have new bindings, we need to send those too).
    // TODO most props and sockets generally stay the same during regeneration, so maybe send fewer of these
    for func_id in all_bound_functions(&ctx, updated_schema_variant_id).await? {
        let func_summary = Func::get_by_id_or_error(&ctx, func_id)
            .await?
            .into_frontend_type(&ctx)
            .await?;

        WsEvent::func_updated(&ctx, func_summary, None)
            .await?
            .publish_on_commit(&ctx)
            .await?;
    }

    ctx.commit().await?;

    Ok(ForceChangeSetResponse::new(
        force_change_set_id,
        RegenerateVariantResponse {
            schema_variant_id: updated_schema_variant_id,
        },
    ))
}

async fn all_bound_functions(
    ctx: &DalContext,
    schema_variant_id: SchemaVariantId,
) -> SchemaVariantResult<HashSet<FuncId>> {
    let mut bound_func_ids = HashSet::new();
    // Add all prop bindings
    for prop_id in SchemaVariant::all_prop_ids(ctx, schema_variant_id).await? {
        let ap_id = Prop::prototype_id(ctx, prop_id).await?;
        let func_id = AttributePrototype::func_id(ctx, ap_id).await?;
        bound_func_ids.insert(func_id);
    }

    // Add all input and output socket bindings
    let (output_socket_ids, input_socket_ids) =
        SchemaVariant::list_all_socket_ids(ctx, schema_variant_id).await?;

    for socket_id in input_socket_ids {
        if let Some(ap_id) = AttributePrototype::find_for_input_socket(ctx, socket_id).await? {
            let func_id = AttributePrototype::func_id(ctx, ap_id).await?;
            bound_func_ids.insert(func_id);
        }
    }

    for socket_id in output_socket_ids {
        if let Some(ap_id) = AttributePrototype::find_for_output_socket(ctx, socket_id).await? {
            let func_id = AttributePrototype::func_id(ctx, ap_id).await?;
            bound_func_ids.insert(func_id);
        }
    }

    Ok(bound_func_ids)
}
