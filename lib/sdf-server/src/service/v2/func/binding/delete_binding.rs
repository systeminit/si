use std::collections::HashSet;

use axum::{
    Json,
    extract::{
        Host,
        OriginalUri,
        Path,
    },
};
use dal::{
    ChangeSet,
    ChangeSetId,
    Component,
    Func,
    FuncId,
    SchemaVariant,
    WorkspacePk,
    WsEvent,
    func::{
        FuncKind,
        binding::{
            EventualParent,
            action::ActionBinding,
            authentication::AuthBinding,
            leaf::LeafBinding,
            management::ManagementBinding,
        },
    },
};
use si_events::audit_log::AuditLogKind;
use si_frontend_types as frontend_types;

use crate::{
    extract::{
        HandlerContext,
        PosthogClient,
    },
    service::{
        force_change_set_response::ForceChangeSetResponse,
        v2::{
            AccessBuilder,
            func::{
                FuncAPIError,
                FuncAPIResult,
            },
        },
    },
    track,
};

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
    let func = Func::get_by_id(&ctx, func_id).await?;

    if func.is_transformation {
        return Err(FuncAPIError::WrongFunctionKindForBinding);
    }

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

                AuthBinding::delete_auth_binding(&ctx, func_id, schema_variant_id).await?
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
            FuncKind::Management => {
                let frontend_types::FuncBinding::Management {
                    management_prototype_id: Some(management_prototype_id),
                    ..
                } = binding
                else {
                    return Err(FuncAPIError::MissingActionPrototype);
                };

                ManagementBinding::delete_management_binding(
                    &ctx,
                    management_prototype_id.into_raw_id().into(),
                )
                .await?
            }
            FuncKind::Attribute
            | FuncKind::Intrinsic
            | FuncKind::SchemaVariantDefinition
            | FuncKind::Unknown => return Err(FuncAPIError::CannotDeleteBindingForFunc),
        };
        match eventual_parent {
            EventualParent::SchemaVariant(schema_variant_id) => {
                let schema_variant = SchemaVariant::get_by_id(&ctx, schema_variant_id).await?;
                ctx.write_audit_log(
                    AuditLogKind::DetachFunc {
                        func_id,
                        func_display_name: func.display_name.clone(),
                        schema_variant_id: Some(schema_variant_id),
                        component_id: None,
                        subject_name: schema_variant.display_name().to_owned(),
                    },
                    func.name.clone(),
                )
                .await?;
                modified_sv_ids.insert(schema_variant_id);
            }
            EventualParent::Component(component_id) => {
                let component = Component::get_by_id(&ctx, component_id).await?;
                ctx.write_audit_log(
                    AuditLogKind::DetachFunc {
                        func_id,
                        func_display_name: func.display_name.clone(),
                        schema_variant_id: None,
                        component_id: Some(component_id),
                        subject_name: component.name(&ctx).await?.to_owned(),
                    },
                    func.name.clone(),
                )
                .await?;
            }
            EventualParent::Schema(_) => {}
        }
    }

    for schema_variant_id in modified_sv_ids {
        let schema = SchemaVariant::schema_id(&ctx, schema_variant_id).await?;
        let schema_variant = SchemaVariant::get_by_id(&ctx, schema_variant_id).await?;

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
    let binding = Func::get_by_id(&ctx, func_id)
        .await?
        .into_frontend_type(&ctx)
        .await?
        .bindings;
    let func_summary = Func::get_by_id(&ctx, func_id)
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
