use axum::extract::Query;
use axum::Json;
use chrono::{DateTime, Utc};
use dal::{
    node::Node, ComponentId, DiagramKind, LabelEntry, LabelList, ResourceView, SchemaId,
    SchemaVariantId, StandardModel, Visibility, WorkspacePk,
};
use dal::{ActorView, ComponentStatus, DalContext, HistoryActorTimestamp};
use serde::{Deserialize, Serialize};

use super::{ComponentError, ComponentResult};
use crate::server::extract::{AccessBuilder, HandlerContext};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListComponentsIdentificationRequest {
    pub workspace_pk: WorkspacePk,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListComponentsIdentificationItemTimestamp {
    actor: ActorView,
    timestamp: DateTime<Utc>,
}

impl ListComponentsIdentificationItemTimestamp {
    async fn from_history_actor_timestamp(
        ctx: &DalContext,
        value: HistoryActorTimestamp,
    ) -> ComponentResult<Self> {
        let actor = ActorView::from_history_actor(ctx, value.actor).await?;

        Ok(Self {
            actor,
            timestamp: value.timestamp,
        })
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListComponentsIdentificationItem {
    pub component_id: ComponentId,
    pub schema_variant_id: SchemaVariantId,
    pub schema_id: SchemaId,
    pub schema_name: String,
    pub diagram_kind: DiagramKind,
    pub schema_variant_name: String,
    pub resource: ResourceView,
    pub created_at: ListComponentsIdentificationItemTimestamp,
    pub updated_at: ListComponentsIdentificationItemTimestamp,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListComponentsIdentificationResponse {
    pub list: LabelList<ListComponentsIdentificationItem>,
}

pub async fn list_components_identification(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<ListComponentsIdentificationRequest>,
) -> ComponentResult<Json<ListComponentsIdentificationResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let nodes = Node::list(&ctx).await?;

    let mut label_entries = Vec::with_capacity(nodes.len());
    for node in &nodes {
        let component = match node.component(&ctx).await? {
            Some(component) => component,
            None => continue,
        };
        let component_status = ComponentStatus::get_by_id(&ctx, component.id())
            .await?
            .ok_or(ComponentError::ComponentNotFound)?;
        let resource = ResourceView::new(component.resource(&ctx).await?);

        let schema_variant = component
            .schema_variant(&ctx)
            .await?
            .ok_or(ComponentError::SchemaVariantNotFound)?;
        let schema = component
            .schema(&ctx)
            .await?
            .ok_or(ComponentError::SchemaNotFound)?;

        let created_at = ListComponentsIdentificationItemTimestamp::from_history_actor_timestamp(
            &ctx,
            component_status.creation(),
        )
        .await?;
        let updated_at = ListComponentsIdentificationItemTimestamp::from_history_actor_timestamp(
            &ctx,
            component_status.update(),
        )
        .await?;

        let value = ListComponentsIdentificationItem {
            component_id: *component.id(),
            schema_variant_id: *schema_variant.id(),
            schema_variant_name: schema_variant.name().to_owned(),
            schema_id: *schema.id(),
            schema_name: schema.name().to_owned(),
            diagram_kind: DiagramKind::Configuration,
            resource,
            created_at,
            updated_at,
        };
        label_entries.push(LabelEntry {
            label: component.name(&ctx).await?,
            value,
        });
    }
    let list = LabelList::from(label_entries);
    let response = ListComponentsIdentificationResponse { list };
    Ok(Json(response))
}
