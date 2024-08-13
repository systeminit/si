use axum::{
    extract::{Host, OriginalUri, Path},
    response::IntoResponse,
    Json,
};
use dal::{
    func::binding::{
        action::ActionBinding, authentication::AuthBinding, leaf::LeafBinding, EventualParent,
    },
    ChangeSet, ChangeSetId, Func, FuncId, SchemaVariant, WorkspacePk, WsEvent,
};
use si_frontend_types as frontend_types;

use crate::{
    server::{
        extract::{AccessBuilder, HandlerContext, PosthogClient},
        tracking::track,
    },
    service::v2::func::{FuncAPIError, FuncAPIResult},
};

pub async fn delete_binding(
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
    match func.kind {
        dal::func::FuncKind::Action => {
            for binding in request.bindings {
                if let frontend_types::FuncBinding::Action {
                    action_prototype_id: Some(action_prototype_id),
                    ..
                } = binding
                {
                    let eventual_parent = ActionBinding::delete_action_binding(
                        &ctx,
                        action_prototype_id.into_raw_id().into(),
                    )
                    .await?;
                    if let EventualParent::SchemaVariant(schema_variant_id) = eventual_parent {
                        let schema =
                            SchemaVariant::schema_id_for_schema_variant_id(&ctx, schema_variant_id)
                                .await?;
                        let schema_variant =
                            SchemaVariant::get_by_id_or_error(&ctx, schema_variant_id).await?;

                        WsEvent::schema_variant_updated(&ctx, schema, schema_variant)
                            .await?
                            .publish_on_commit(&ctx)
                            .await?;
                    }
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
                            let eventual_parent = LeafBinding::delete_leaf_func_binding(
                                &ctx,
                                attribute_prototype_id.into_raw_id().into(),
                            )
                            .await?;
                            if let EventualParent::SchemaVariant(schema_variant_id) =
                                eventual_parent
                            {
                                let schema = SchemaVariant::schema_id_for_schema_variant_id(
                                    &ctx,
                                    schema_variant_id,
                                )
                                .await?;
                                let schema_variant =
                                    SchemaVariant::get_by_id_or_error(&ctx, schema_variant_id)
                                        .await?;

                                WsEvent::schema_variant_updated(&ctx, schema, schema_variant)
                                    .await?
                                    .publish_on_commit(&ctx)
                                    .await?;
                            }
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
                            let eventual_parent = LeafBinding::delete_leaf_func_binding(
                                &ctx,
                                attribute_prototype_id.into_raw_id().into(),
                            )
                            .await?;
                            if let EventualParent::SchemaVariant(schema_variant_id) =
                                eventual_parent
                            {
                                let schema = SchemaVariant::schema_id_for_schema_variant_id(
                                    &ctx,
                                    schema_variant_id,
                                )
                                .await?;
                                let schema_variant =
                                    SchemaVariant::get_by_id_or_error(&ctx, schema_variant_id)
                                        .await?;

                                WsEvent::schema_variant_updated(&ctx, schema, schema_variant)
                                    .await?
                                    .publish_on_commit(&ctx)
                                    .await?;
                            }
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
        dal::func::FuncKind::Authentication => {
            for binding in request.bindings {
                if let frontend_types::FuncBinding::Authentication {
                    schema_variant_id,
                    func_id,
                } = binding
                {
                    match func_id {
                        Some(func_id) => {
                            let eventual_parent = AuthBinding::delete_auth_binding(
                                &ctx,
                                func_id.into(),
                                schema_variant_id.into(),
                            )
                            .await?;
                            if let EventualParent::SchemaVariant(schema_variant_id) =
                                eventual_parent
                            {
                                let schema = SchemaVariant::schema_id_for_schema_variant_id(
                                    &ctx,
                                    schema_variant_id,
                                )
                                .await?;
                                let schema_variant =
                                    SchemaVariant::get_by_id_or_error(&ctx, schema_variant_id)
                                        .await?;

                                WsEvent::schema_variant_updated(&ctx, schema, schema_variant)
                                    .await?
                                    .publish_on_commit(&ctx)
                                    .await?;
                            }
                        }
                        None => return Err(FuncAPIError::MissingFuncId),
                    }
                } else {
                    return Err(FuncAPIError::WrongFunctionKindForBinding);
                }
            }
        }
        dal::func::FuncKind::Attribute
        | dal::func::FuncKind::Intrinsic
        | dal::func::FuncKind::SchemaVariantDefinition
        | dal::func::FuncKind::Unknown => return Err(FuncAPIError::CannotDeleteBindingForFunc),
    };

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
