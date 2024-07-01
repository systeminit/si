use axum::{
    extract::{OriginalUri, Path},
    response::IntoResponse,
    Json,
};
use dal::{
    func::binding::{action::ActionBinding, leaf::LeafBinding, FuncBindings},
    ChangeSet, ChangeSetId, Func, FuncId, WorkspacePk, WsEvent,
};
use si_frontend_types as frontend_types;

use crate::{
    server::{
        extract::{AccessBuilder, HandlerContext, PosthogClient},
        tracking::track,
    },
    service::v2::func::{get_types, FuncAPIError, FuncAPIResult},
};

pub async fn delete_binding(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Path((_workspace_pk, change_set_id, func_id)): Path<(WorkspacePk, ChangeSetId, FuncId)>,
    Json(request): Json<frontend_types::FuncBindings>,
) -> FuncAPIResult<impl IntoResponse> {
    let mut ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;
    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;
    let func = Func::get_by_id_or_error(&ctx, func_id).await?;
    match func.kind {
        dal::func::FuncKind::Action => {
            for binding in request.bindings {
                if let frontend_types::FuncBinding::Action {
                    action_prototype_id: Some(action_prototype_id),
                    ..
                } = binding
                {
                    ActionBinding::delete_action_binding(
                        &ctx,
                        action_prototype_id.into_raw_id().into(),
                    )
                    .await?;
                } else {
                    return Err(FuncAPIError::MissingActionPrototype);
                }
            }
        }
        dal::func::FuncKind::CodeGeneration | dal::func::FuncKind::Qualification => {
            for binding in request.bindings {
                if let frontend_types::FuncBinding::CodeGeneration {
                    attribute_prototype_id,
                    ..
                } = binding
                {
                    match attribute_prototype_id {
                        Some(attribute_prototype_id) => {
                            LeafBinding::delete_leaf_func_binding(
                                &ctx,
                                attribute_prototype_id.into_raw_id().into(),
                            )
                            .await?;
                        }
                        None => {
                            return Err(FuncAPIError::MissingPrototypeId);
                        }
                    }
                } else if let frontend_types::FuncBinding::Qualification {
                    attribute_prototype_id,
                    ..
                } = binding
                {
                    match attribute_prototype_id {
                        Some(attribute_prototype_id) => {
                            LeafBinding::delete_leaf_func_binding(
                                &ctx,
                                attribute_prototype_id.into_raw_id().into(),
                            )
                            .await?;
                        }
                        None => {
                            return Err(FuncAPIError::MissingPrototypeId);
                        }
                    }
                } else {
                    return Err(FuncAPIError::WrongFunctionKindForBinding);
                }
            }
        }
        _ => {
            return Err(FuncAPIError::WrongFunctionKindForBinding);
        }
    };
    let binding = FuncBindings::from_func_id(&ctx, func_id)
        .await?
        .into_frontend_type();
    track(
        &posthog_client,
        &ctx,
        &original_uri,
        "delete_binding",
        serde_json::json!({
            "how": "/func/delete_binding",
            "func_id": func_id,
            "func_name": func.name.clone(),
            "func_kind": func.kind.clone(),
        }),
    );
    let types = get_types(&ctx, func_id).await?;
    WsEvent::func_bindings_updated(&ctx, binding.clone(), types)
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
