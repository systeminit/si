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
    ComponentId,
    Func,
    Schema,
    SchemaId,
    SchemaVariant,
    SchemaVariantId,
    WorkspacePk,
    WsEvent,
    diagram::view::ViewId,
    generate_name,
};
use serde::{
    Deserialize,
    Serialize,
};
use si_events::audit_log::AuditLogKind;
use si_frontend_mv_types::{
    component::{
        Component as ComponentMv,
        attribute_tree::AttributeTree as AttributeTreeMv,
    },
    schema_variant::SchemaVariant as SchemaVariantMv,
};
use si_frontend_types::SchemaVariant as FrontendVariant;

use super::{
    ViewError,
    ViewResult,
};
use crate::{
    extract::{
        HandlerContext,
        PosthogClient,
    },
    service::{
        force_change_set_response::ForceChangeSetResponse,
        v2::AccessBuilder,
    },
    track,
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
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateComponentResponse {
    pub materialized_view: ComponentMv,
    pub attribute_tree_materialized_view: AttributeTreeMv,
    pub schema_variant_materialized_view: SchemaVariantMv,
    pub component_id: ComponentId,
    pub installed_variant: Option<FrontendVariant>,
}

pub async fn create_component(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    Path((_workspace_pk, change_set_id, view_id)): Path<(WorkspacePk, ChangeSetId, ViewId)>,
    Json(request): Json<CreateComponentRequest>,
) -> ViewResult<ForceChangeSetResponse<CreateComponentResponse>> {
    let mut ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;

    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    let name = generate_name();

    let (schema_variant_id, installed_variant) = match request.schema_type {
        CreateComponentSchemaType::Installed => (
            request.schema_variant_id.ok_or(ViewError::InvalidRequest(
                "schemaVariantId missing on installed schema create component request".into(),
            ))?,
            None,
        ),
        // Install assets on demand when creating a component
        CreateComponentSchemaType::Uninstalled => {
            let schema_id = request.schema_id.ok_or(ViewError::InvalidRequest(
                "schemaId missing on uninstalled schema create component request".into(),
            ))?;

            // We want to be sure that we don't have stale frontend data,
            // since this module might have just been installed, or
            // installed by another user
            let variant_id = Schema::get_or_install_default_variant(&ctx, schema_id).await?;
            let variant = SchemaVariant::get_by_id(&ctx, variant_id).await?;

            let front_end_variant = variant.into_frontend_type(&ctx, schema_id).await?;
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

    component
        .set_geometry(
            &ctx,
            view_id,
            0,
            0,
            initial_geometry.width(),
            initial_geometry.height(),
        )
        .await?;

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

    ctx.commit().await?;

    // Construct the materialized views in parallel
    let (component_mv_result, attribute_tree_mv_result, schema_variant_mv_result) = tokio::join!(
        tokio::spawn(dal_materialized_views::component::assemble(
            ctx.clone(),
            component.id(),
        )),
        tokio::spawn(dal_materialized_views::component::attribute_tree::assemble(
            ctx.clone(),
            component.id()
        )),
        tokio::spawn(dal_materialized_views::schema_variant::assemble(
            ctx.clone(),
            variant.id(),
        ))
    );

    Ok(ForceChangeSetResponse::new(
        force_change_set_id,
        CreateComponentResponse {
            materialized_view: component_mv_result??,
            attribute_tree_materialized_view: attribute_tree_mv_result??,
            schema_variant_materialized_view: schema_variant_mv_result??,
            component_id: component.id(),
            installed_variant,
        },
    ))
}
