use asset_sprayer::prompt::{AwsCliCommand, AwsCliCommandPromptKind, Prompt};
use axum::extract::{Host, OriginalUri, Path, Query};
use dal::{
    action::prototype::{ActionKind, ActionPrototype},
    func::authoring::FuncAuthoringClient,
    prompt_override::PromptOverride,
    ChangeSet, ChangeSetId, DalContext, Func, FuncId, SchemaVariant, SchemaVariantId, WorkspacePk,
    WsEvent,
};
use serde::{Deserialize, Serialize};
use si_frontend_types::FuncCode;

use crate::{
    extract::{AccessBuilder, AssetSprayer, HandlerContext, PosthogClient},
    service::force_change_set_response::ForceChangeSetResponse,
    track,
};

use super::{FuncAPIError, FuncAPIResult};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GenerateAwsFunctionQuery {
    command: String,
    subcommand: String,
    schema_variant_id: SchemaVariantId,
}

#[allow(clippy::too_many_arguments)]
pub async fn generate_aws_function(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    AssetSprayer(asset_sprayer): AssetSprayer,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    Path((_workspace_pk, change_set_id, func_id)): Path<(WorkspacePk, ChangeSetId, FuncId)>,
    Query(query): Query<GenerateAwsFunctionQuery>,
) -> FuncAPIResult<ForceChangeSetResponse<()>> {
    let mut ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;
    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    let schema_variant_id = query.schema_variant_id;
    let schema_variant = SchemaVariant::get_by_id_or_error(&ctx, schema_variant_id).await?;

    // Figure out which prompt to use based on function type
    let prompt_kind = get_prompt_kind(&ctx, func_id, &schema_variant).await?;
    let aws_command = AwsCliCommand(query.command.clone(), query.subcommand.clone());
    let prompt = Prompt::AwsCliCommandPrompt(prompt_kind, aws_command);

    // Tell everyone this function is generating so they can lock it
    println!("Sending generate event ...");
    WsEvent::func_generating(&ctx, func_id, query.command, query.subcommand)
        .await?
        .publish_immediately(&ctx)
        .await?;

    // Generate the code.
    let result = generate_and_save_code(&ctx, &asset_sprayer, &prompt, func_id).await;

    // Always send the code saved event, even if there was an error generating.
    let code_save_result = send_func_code_saved_event(&ctx, func_id).await;
    let func = result.and(code_save_result)?;

    // WsEvent::schema_variant_updated(
    //     &ctx,
    //     schema_id,
    //     SchemaVariant::get_by_id_or_error(&ctx, schema_variant_id).await?,
    // )
    // .await?
    // .publish_on_commit(&ctx)
    // .await?;
    // None

    let schema_id = schema_variant.schema_id(&ctx).await?;
    let variant = schema_variant.into_frontend_type(&ctx, schema_id).await?;
    track(
        &posthog_client,
        &ctx,
        &original_uri,
        &host_name,
        "generate_aws_function",
        serde_json::json!({
            "func_id": func_id,
            "func_name": func.name.clone(),
            "func_display_name": func.display_name.clone(),
            "variant_id": schema_variant_id,
            "variant_category": variant.category.clone(),
            "variant_name": variant.schema_name.clone(),
            "variant_display_name": variant.display_name.clone(),
            "prompt": prompt,
        }),
    );

    Ok(ForceChangeSetResponse::empty(force_change_set_id))
}

async fn generate_and_save_code(
    ctx: &DalContext,
    asset_sprayer: &asset_sprayer::AssetSprayer,
    prompt: &Prompt,
    func_id: FuncId,
) -> FuncAPIResult<()> {
    let raw_prompt = match PromptOverride::get_opt(ctx, prompt.kind().as_ref()).await? {
        Some(prompt_override) => prompt_override.into(),
        None => asset_sprayer.raw_prompt(prompt.kind()).await?,
    };
    let code = asset_sprayer.run(prompt, &raw_prompt).await?;
    FuncAuthoringClient::save_code(ctx, func_id, code).await?;
    ctx.commit().await?;
    Ok(())
}

async fn send_func_code_saved_event(ctx: &DalContext, func_id: FuncId) -> FuncAPIResult<Func> {
    let func = Func::get_by_id_or_error(ctx, func_id).await?;
    let func_code = FuncCode {
        func_id,
        code: func.code_plaintext()?.unwrap_or("".to_string()),
    };
    WsEvent::func_code_saved(ctx, func_code, true)
        .await?
        .publish_immediately(ctx)
        .await?;
    Ok(func)
}

async fn get_prompt_kind(
    ctx: &DalContext,
    func_id: FuncId,
    schema_variant: &SchemaVariant,
) -> FuncAPIResult<AwsCliCommandPromptKind> {
    let mut prompt_kind = None;
    if schema_variant.asset_func_id() == Some(func_id) {
        prompt_kind = Some(AwsCliCommandPromptKind::AssetSchema);
    }
    for ap_id in ActionPrototype::list_for_func_id(ctx, func_id).await? {
        let ap = ActionPrototype::get_by_id(ctx, ap_id).await?;
        let new_prompt_kind = match ap.kind {
            ActionKind::Create => AwsCliCommandPromptKind::CreateAction,
            ActionKind::Destroy => AwsCliCommandPromptKind::DestroyAction,
            ActionKind::Manual => AwsCliCommandPromptKind::UpdateAction,
            ActionKind::Refresh => AwsCliCommandPromptKind::RefreshAction,
            ActionKind::Update => AwsCliCommandPromptKind::UpdateAction,
        };
        if prompt_kind.is_some_and(|prompt_kind| prompt_kind != new_prompt_kind) {
            return Err(FuncAPIError::MultipleActionTypes(func_id));
        }
        prompt_kind = Some(new_prompt_kind);
    }
    // prompt_kind.ok_or(FuncAPIError::NoActionTypes(func_id))
    Ok(prompt_kind.unwrap_or(AwsCliCommandPromptKind::Import))
}
