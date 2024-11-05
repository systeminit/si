use asset_sprayer::prompt::Prompt;
use axum::extract::{Host, OriginalUri, Path, Query};
use dal::{
    schema::variant::authoring::VariantAuthoringClient, ChangeSet, ChangeSetId, SchemaVariant,
    SchemaVariantId, WorkspacePk, WsEvent,
};
use serde::{Deserialize, Serialize};

use crate::{
    extract::{AccessBuilder, AssetSprayer, HandlerContext, PosthogClient},
    service::force_change_set_response::ForceChangeSetResponse,
    track,
};

use super::SchemaVariantsAPIResult;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AwsCommand {
    pub command: String,
    pub subcommand: String,
}

#[allow(clippy::too_many_arguments)]
pub async fn generate_aws_asset_schema(
    HandlerContext(builder): HandlerContext,
    AssetSprayer(asset_sprayer): AssetSprayer,
    AccessBuilder(access_builder): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    Path((_workspace_pk, change_set_id, schema_variant_id)): Path<(
        WorkspacePk,
        ChangeSetId,
        SchemaVariantId,
    )>,
    Query(aws_command): Query<AwsCommand>,
) -> SchemaVariantsAPIResult<ForceChangeSetResponse<()>> {
    let mut ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;
    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    // Generate the code
    let prompt = Prompt::AwsAssetSchema {
        command: aws_command.command.clone(),
        subcommand: aws_command.subcommand.clone(),
    };
    let code = asset_sprayer.run(&prompt).await?;

    // Update the function
    let schema_variant = SchemaVariant::get_by_id_or_error(&ctx, schema_variant_id).await?;
    let schema_id = SchemaVariant::schema_id_for_schema_variant_id(&ctx, schema_variant_id).await?;
    let variant = schema_variant.into_frontend_type(&ctx, schema_id).await?;

    VariantAuthoringClient::save_variant_content(
        &ctx,
        schema_variant_id,
        &variant.schema_name,
        variant.display_name.clone(),
        variant.category.clone(),
        variant.description.clone(),
        variant.link.clone(),
        variant.color.clone(),
        variant.component_type.into(),
        Some(code),
    )
    .await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        &host_name,
        "generate_aws_asset_schema",
        serde_json::json!({
            "variant_id": schema_variant_id,
            "variant_category": variant.category.clone(),
            "variant_name": variant.schema_name.clone(),
            "variant_display_name": variant.display_name.clone(),
            "asset_func_id": variant.asset_func_id,
        }),
    );

    WsEvent::schema_variant_updated(
        &ctx,
        schema_id,
        SchemaVariant::get_by_id_or_error(&ctx, schema_variant_id).await?,
    )
    .await?
    .publish_on_commit(&ctx)
    .await?;

    ctx.commit().await?;

    Ok(ForceChangeSetResponse::empty(force_change_set_id))
}
