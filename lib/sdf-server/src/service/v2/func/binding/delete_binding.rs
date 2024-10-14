use crate::{
    extract::{AccessBuilder, HandlerContext, PosthogClient},
    service::{
        force_change_set_response::ForceChangeSetResponse,
        v2::func::{FuncAPIError, FuncAPIResult},
    },
    track,
};
use axum::{
    extract::{Host, OriginalUri, Path},
    Json,
};
use dal::func::FuncKind;
use dal::{
    func::binding::{
        action::ActionBinding, authentication::AuthBinding, leaf::LeafBinding, EventualParent,
    },
    ChangeSet, ChangeSetId, Func, FuncId, SchemaVariant, WorkspacePk, WsEvent,
};
use si_frontend_types as frontend_types;
use std::collections::HashSet;

pub async fn delete_binding(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    Path((_workspace_pk, change_set_id, func_id)): Path<(WorkspacePk, ChangeSetId, FuncId)>,
    Json(request): Json<frontend_types::FuncBindings>,
) -> FuncAPIResult<ForceChangeSetResponse<Vec<frontend_types::FuncBinding>>> {
    let mut ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;
    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;
    let func = Func::get_by_id_or_error(&ctx, func_id).await?;

    let mut modified_sv_ids = HashSet::new();

    // Note(victor): Matching inside the loop for a static variable may look weird, but
    // branch prediction will mitigate any performance penalty, and there's no elegant way to implement
    // these branches as closures as of today
    for binding in request.bindings {
        let eventual_parent = match func.kind {
            FuncKind::Action => {
                let frontend_types::FuncBinding::Action {
                    action_prototype_id: Some(action_prototype_id),
                    ..
                } = binding
                else {
                    return Err(FuncAPIError::MissingActionPrototype);
                };

                ActionBinding::delete_action_binding(&ctx, action_prototype_id.into_raw_id().into())
                    .await?
            }
            FuncKind::Authentication => {
                let frontend_types::FuncBinding::Authentication {
                    schema_variant_id,
                    func_id,
                } = binding
                else {
                    return Err(FuncAPIError::WrongFunctionKindForBinding);
                };

                let Some(func_id) = func_id else {
                    return Err(FuncAPIError::MissingFuncId);
                };

                AuthBinding::delete_auth_binding(&ctx, func_id.into(), schema_variant_id.into())
                    .await?
            }
            FuncKind::CodeGeneration | FuncKind::Qualification => {
                let (frontend_types::FuncBinding::Qualification {
                    attribute_prototype_id,
                    ..
                }
                | frontend_types::FuncBinding::CodeGeneration {
                    attribute_prototype_id,
                    ..
                }) = binding
                else {
                    return Err(FuncAPIError::WrongFunctionKindForBinding);
                };

                let Some(attribute_prototype_id) = attribute_prototype_id else {
                    return Err(FuncAPIError::MissingPrototypeId);
                };

                LeafBinding::delete_leaf_func_binding(
                    &ctx,
                    attribute_prototype_id.into_raw_id().into(),
                )
                .await?
            }
            FuncKind::Attribute
            | FuncKind::Management
            | FuncKind::Intrinsic
            | FuncKind::SchemaVariantDefinition
            | FuncKind::Unknown => return Err(FuncAPIError::CannotDeleteBindingForFunc),
        };

        if let EventualParent::SchemaVariant(schema_variant_id) = eventual_parent {
            modified_sv_ids.insert(schema_variant_id);
        }
    }

    for schema_variant_id in modified_sv_ids {
        let schema =
            SchemaVariant::schema_id_for_schema_variant_id(&ctx, schema_variant_id).await?;
        let schema_variant = SchemaVariant::get_by_id_or_error(&ctx, schema_variant_id).await?;

        WsEvent::schema_variant_updated(&ctx, schema, schema_variant)
            .await?
            .publish_on_commit(&ctx)
            .await?;
    }

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        &host_name,
        "delete_binding",
        serde_json::json!({
            "how": "/func/delete_binding",
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
    WsEvent::func_updated(&ctx, func_summary.clone(), None)
        .await?
        .publish_on_commit(&ctx)
        .await?;
    ctx.commit().await?;

    Ok(ForceChangeSetResponse::new(force_change_set_id, binding))
}
