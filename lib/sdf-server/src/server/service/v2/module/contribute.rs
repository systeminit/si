use axum::{
    extract::{OriginalUri, Path, Query},
    response::IntoResponse,
};

use dal::{module::Module, ChangeSetId, WorkspacePk};
use module_index_client::ModuleIndexClient;
use si_frontend_types as frontend_types;
use telemetry::prelude::*;

use crate::server::{
    extract::{AccessBuilder, HandlerContext, PosthogClient, RawAccessToken},
    tracking::track,
};

use super::ModulesAPIError;

pub async fn contribute(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    RawAccessToken(raw_access_token): RawAccessToken,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Path((_workspace_pk, change_set_id)): Path<(WorkspacePk, ChangeSetId)>,
    Query(request): Query<frontend_types::ModuleContributeRequest>,
) -> Result<impl IntoResponse, ModulesAPIError> {
    let ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;

    // Prepare a module index client. We'll re-use it for every request.
    let module_index_url = match ctx.module_index_url() {
        Some(url) => url,
        None => return Err(ModulesAPIError::ModuleIndexNotConfigured),
    };
    let index_client = ModuleIndexClient::new(module_index_url.try_into()?, &raw_access_token);

    // Prepare data for determining http status code.
    let item_count = request.modules.len();
    let mut errors = Vec::new();

    // NOTE(nick): right now, we contribute modules individually. Perhaps, we should do one of two things: send them in
    // bulk OR create a module index route that takes them in bulk.
    for module in &request.modules {
        let (name, version, based_on_hash, schema_id, payload) = Module::prepare_contribution(
            &ctx,
            module.name.as_str(),
            module.version.as_str(),
            module.schema_id.into(),
        )
        .await?;

        match index_client
            .upload_module(
                name.as_str(),
                version.as_str(),
                based_on_hash,
                schema_id.map(|id| id.to_string()),
                payload,
            )
            .await
        {
            Ok(response) => {
                debug!(?response, "contribution complete");
            }
            Err(err) => {
                error!(?err);
                errors.push((module.to_owned(), err));
            }
        }
    }

    // NOTE(nick): if you are reading this, you may have noticed we do not commit here... well, at least I hope so
    // until this comment is deleted. This is because the module index's database is the only persistent storage layer
    // that will be mutated when contributing modules. In other words, we can process partial failures because
    // successful contributions _will_ be successful... regardless of how the other contributions fared. There's likely
    // some noodling and tuning to do, but this should hopefully help.
    if errors.is_empty() {
        if errors.len() == item_count {
            return Err(ModulesAPIError::ContributionTotalFailure(errors));
        }
        return Err(ModulesAPIError::ContributionPartialFailure(errors));
    }

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        "contribute",
        serde_json::json!({
            "modules": request.modules,
        }),
    );

    Ok(axum::response::Response::builder().body(axum::body::Empty::new())?)
}
