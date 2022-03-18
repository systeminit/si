use axum::extract::Query;
use axum::Json;
use dal::{Component, ComponentId, LabelEntry, LabelList, StandardModel, Visibility, WorkspaceId};
use serde::{Deserialize, Serialize};

use super::{ComponentError, ComponentResult};
use crate::server::extract::{AccessBuilder, HandlerContext};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListComponentNamesOnlyRequest {
    pub workspace_id: WorkspaceId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListComponentNamesOnlyItem {
    pub id: ComponentId,
    pub name: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListComponentNamesOnlyResponse {
    pub list: LabelList<ComponentId>,
}

// NOTE(nick): this name is long and cumbersome, but the hole has been dug for this dummy data
// provider. Future changes to this code should consider renaming this (and its route, TS client,
// etc.) to something more readable, such as "list_component_names".
pub async fn list_components_names_only(
    HandlerContext(builder, mut txns): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<ListComponentNamesOnlyRequest>,
) -> ComponentResult<Json<ListComponentNamesOnlyResponse>> {
    let txns = txns.start().await?;
    let ctx = builder.build(request_ctx.build(request.visibility), &txns);

    let components = Component::list(
        ctx.pg_txn(),
        &ctx.read_tenancy().into(),
        &request.visibility,
    )
    .await?;
    let mut label_entries = Vec::with_capacity(components.len());
    for component in components {
        label_entries.push(LabelEntry {
            label: component
                .find_value_by_json_pointer::<String>(
                    ctx.pg_txn(),
                    ctx.read_tenancy(),
                    ctx.visibility(),
                    "/root/si/name",
                )
                .await?
                .ok_or(ComponentError::ComponentNameNotFound)?,
            value: *component.id(),
        });
    }
    let list = LabelList::from(label_entries);
    let response = ListComponentNamesOnlyResponse { list };
    Ok(Json(response))
}
