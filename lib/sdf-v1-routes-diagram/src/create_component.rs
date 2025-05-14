use std::collections::HashMap;

use axum::{
    Json,
    extract::{
        Host,
        OriginalUri,
    },
};
use dal::{
    ChangeSet,
    Component,
    ComponentId,
    Func,
    Schema,
    SchemaId,
    SchemaVariant,
    SchemaVariantId,
    WsEvent,
    change_status::ChangeStatus,
    component::frame::Frame,
    diagram::view::View,
    generate_name,
};
use sdf_core::{
    force_change_set_response::ForceChangeSetResponse,
    tracking::track,
};
use sdf_extract::{
    HandlerContext,
    PosthogClient,
    v1::AccessBuilder,
};
use serde::{
    Deserialize,
    Serialize,
};
use si_db::Visibility;
use si_events::audit_log::AuditLogKind;
use si_frontend_types::SchemaVariant as FrontendVariant;

use super::{
    DiagramError,
    DiagramResult,
};

#[derive(Deserialize, Serialize, Debug, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub enum CreateComponentSchemaType {
    Installed,
    Uninstalled,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateComponentRequest {
    pub schema_type: CreateComponentSchemaType,
    pub schema_variant_id: Option<SchemaVariantId>,
    pub schema_id: Option<SchemaId>,
    pub parent_id: Option<ComponentId>,
    pub x: String,
    pub y: String,
    pub height: Option<String>,
    pub width: Option<String>,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateComponentResponse {
    pub component_id: ComponentId,
    pub installed_variant: Option<FrontendVariant>,
}

pub async fn create_component(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    Json(request): Json<CreateComponentRequest>,
) -> DiagramResult<ForceChangeSetResponse<CreateComponentResponse>> {
    let mut ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    let name = generate_name();

    let (schema_variant_id, installed_variant) = match request.schema_type {
        CreateComponentSchemaType::Installed => (
            request
                .schema_variant_id
                .ok_or(DiagramError::InvalidRequest(
                    "schemaVariantId missing on installed schema create component request".into(),
                ))?,
            None,
        ),
        // Install assets on demand when creating a component
        CreateComponentSchemaType::Uninstalled => {
            let schema_id = request.schema_id.ok_or(DiagramError::InvalidRequest(
                "schemaId missing on uninstalled schema create component request".into(),
            ))?;

            let variant_id = Schema::get_or_install_default_variant(&ctx, schema_id).await?;
            let variant = SchemaVariant::get_by_id(&ctx, variant_id).await?;

            let front_end_variant = variant.clone().into_frontend_type(&ctx, schema_id).await?;
            WsEvent::module_imported(&ctx, vec![front_end_variant.clone()])
                .await?
                .publish_on_commit(&ctx)
                .await?;
            for func_id in front_end_variant.func_ids.iter() {
                let func = Func::get_by_id(&ctx, *func_id).await?;
                let front_end_func = func.into_frontend_type(&ctx).await?;
                WsEvent::func_updated(&ctx, front_end_func, None)
                    .await?
                    .publish_on_commit(&ctx)
                    .await?;
            }

            (variant_id, Some(front_end_variant))
        }
    };

    let variant = SchemaVariant::get_by_id(&ctx, schema_variant_id).await?;
    let view_id = View::get_id_for_default(&ctx).await?;
    let mut component = Component::new(&ctx, &name, variant.id(), view_id).await?;
    let initial_geometry = component.geometry(&ctx, view_id).await?;
    ctx.write_audit_log(
        AuditLogKind::CreateComponent {
            name: name.to_string(),
            component_id: component.id(),
            schema_variant_id,
            schema_variant_name: variant.display_name().to_owned(),
        },
        name.to_string(),
    )
    .await?;

    let maybe_x = request.x.clone().parse::<isize>();
    let maybe_y = request.y.clone().parse::<isize>();
    let maybe_width = request
        .width
        .map(|w| w.clone().parse::<isize>())
        .transpose();
    let maybe_height = request
        .height
        .map(|h| h.clone().parse::<isize>())
        .transpose();

    if let (Ok(x), Ok(y), Ok(width), Ok(height)) = (maybe_x, maybe_y, maybe_width, maybe_height) {
        component
            .set_geometry(
                &ctx,
                view_id,
                x,
                y,
                width.or_else(|| initial_geometry.width()),
                height.or_else(|| initial_geometry.height()),
            )
            .await?;
    } else {
        ctx.rollback().await?;
        return Err(DiagramError::InvalidRequest(
            "geometry unable to be parsed from create component request".into(),
        ));
    }

    if let Some(frame_id) = request.parent_id {
        Frame::upsert_parent(&ctx, component.id(), frame_id).await?;

        track(
            &posthog_client,
            &ctx,
            &original_uri,
            &host_name,
            "component_attached_to_frame",
            serde_json::json!({
                "how": "/diagram/create_component",
                "component_id": component.id(),
                "parent_id": frame_id.clone(),
                "change_set_id": ctx.change_set_id(),
                "installed_on_demand": matches!(request.schema_type, CreateComponentSchemaType::Uninstalled),
            }),
        );
    } else {
        track(
            &posthog_client,
            &ctx,
            &original_uri,
            &host_name,
            "component_created",
            serde_json::json!({
                "how": "/diagram/create_component",
                "component_id": component.id(),
                "component_name": name.clone(),
                "change_set_id": ctx.change_set_id(),
                "installed_on_demand": matches!(request.schema_type, CreateComponentSchemaType::Uninstalled),
            }),
        );
    }

    let mut diagram_sockets = HashMap::new();
    let payload = component
        .into_frontend_type_for_default_view(&ctx, ChangeStatus::Added, &mut diagram_sockets)
        .await?;
    WsEvent::component_created(&ctx, payload)
        .await?
        .publish_on_commit(&ctx)
        .await?;

    ctx.commit().await?;

    Ok(ForceChangeSetResponse::new(
        force_change_set_id,
        CreateComponentResponse {
            component_id: component.id(),
            installed_variant,
        },
    ))
}
