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
    func::binding::{
        EventualParent,
        attribute::AttributeBinding,
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

pub async fn reset_attribute_binding(
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

        match eventual_parent {
            EventualParent::SchemaVariant(schema_variant_id) => {
                let schema = SchemaVariant::schema_id(&ctx, schema_variant_id).await?;
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
                WsEvent::schema_variant_updated(&ctx, schema, schema_variant)
                    .await?
                    .publish_on_commit(&ctx)
                    .await?;
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
