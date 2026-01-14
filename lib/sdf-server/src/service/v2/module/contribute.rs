use axum::{
    Json,
    extract::{
        Host,
        OriginalUri,
        Path,
    },
    response::IntoResponse,
};
use dal::{
    ChangeSetId,
    WorkspacePk,
    module::Module,
};
use module_index_client::ModuleIndexClient;
use si_events::audit_log::AuditLogKind;
use si_frontend_types as frontend_types;

use super::ModulesAPIError;
use crate::{
    extract::{
        HandlerContext,
        PosthogClient,
        request::RawAccessToken,
    },
    service::v2::AccessBuilder,
    track,
};

#[allow(clippy::too_many_arguments)]
pub async fn contribute(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    RawAccessToken(raw_access_token): RawAccessToken,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    Path((_workspace_pk, change_set_id)): Path<(WorkspacePk, ChangeSetId)>,
    Json(request): Json<frontend_types::ModuleContributeRequest>,
) -> Result<impl IntoResponse, ModulesAPIError> {
    let ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;

    // Prepare a module index client. We'll re-use it for every request.
    let module_index_url = match ctx.module_index_url() {
        Some(url) => url,
        None => return Err(ModulesAPIError::ModuleIndexNotConfigured),
    };
    let index_client = ModuleIndexClient::new(module_index_url.try_into()?, &raw_access_token)?;

    let (
        name,
        version,
        based_on_hash,
        schema_id,
        payload,
        created_by_name,
        created_by_email,
        schema_variant_version,
    ) = Module::prepare_contribution(
        &ctx,
        request.name.as_str(),
        request.version.as_str(),
        request.schema_variant_id,
        true, // TODO make this an API argument
    )
    .await?;

    let response = index_client
        .upload_module(
            name.as_str(),
            version.as_str(),
            based_on_hash.clone(),
            schema_id.map(|id| id.to_string()),
            payload.clone(),
            Some(request.schema_variant_id.to_string()),
            Some(schema_variant_version.clone()),
            None,
        )
        .await?;

    ctx.write_audit_log(
        AuditLogKind::ContributeModule {
            version: version.clone(),
            schema_id,
            schema_variant_id: request.schema_variant_id.into(),
            schema_variant_version: Some(schema_variant_version.clone()),
        },
        name.clone(),
    )
    .await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        &host_name,
        "export_module",
        serde_json::json!({
            "pkg_name": name,
            "pkg_version": version,
            "based_on_hash": based_on_hash,
            "pkg_created_by_name": created_by_name,
            "pkg_created_by_email": created_by_email,
            "schema_variant_id": request.schema_variant_id,
            "schema_id": schema_id,
            "pkg_hash": response.latest_hash,
        }),
    );
    ctx.commit().await?;

    Ok(axum::response::Response::builder().body(axum::body::Empty::new())?)
}
