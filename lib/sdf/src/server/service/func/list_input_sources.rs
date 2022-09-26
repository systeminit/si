use super::FuncResult;
use crate::server::extract::{AccessBuilder, HandlerContext};
use axum::{extract::Query, Json};
use dal::{
    prop_tree::PropTree, InternalProvider, InternalProviderId, PropId, PropKind, SchemaVariantId,
    StandardModel, Visibility,
};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InputSourceSocket {
    pub schema_variant_id: SchemaVariantId,
    pub internal_provider_id: InternalProviderId,
    pub name: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InputSourceProp {
    pub schema_variant_id: SchemaVariantId,
    pub internal_provider_id: InternalProviderId,
    pub prop_id: PropId,
    pub kind: PropKind,
    pub name: String,
    pub path: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListInputSourcesRequest {
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListInputSourcesResponse {
    pub sockets: Vec<InputSourceSocket>,
    pub props: Vec<InputSourceProp>,
}

// We take the tree and recompose it as a list using depth-first search, filtering out props that
// do not have an internal provider id (and thus cannot be used as function input sources)
fn prop_tree_to_list(prop_tree: &PropTree) -> Vec<InputSourceProp> {
    let mut prop_sources = vec![];

    for root in &prop_tree.root_props {
        let mut work_queue = VecDeque::from([root]);

        while let Some(cur) = work_queue.pop_front() {
            for child in &cur.children {
                work_queue.push_front(&child);
            }

            // No internal provider id? Not a valid source
            if let Some(internal_provider_id) = cur.internal_provider_id {
                prop_sources.push(InputSourceProp {
                    schema_variant_id: cur.schema_variant_id.clone(),
                    internal_provider_id: internal_provider_id.clone(),
                    prop_id: cur.prop_id.clone(),
                    kind: cur.kind.clone(),
                    name: cur.name.clone(),
                    path: cur.path.clone(),
                });
            }
        }
    }

    prop_sources
}

pub async fn list_input_sources(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<ListInputSourcesRequest>,
) -> FuncResult<Json<ListInputSourcesResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let sockets = dbg!(InternalProvider::list_for_input_sockets(&ctx)
        .await?
        .iter()
        .map(|ip| InputSourceSocket {
            internal_provider_id: ip.id().to_owned(),
            schema_variant_id: ip.schema_variant_id().to_owned(),
            name: ip.name().to_owned(),
        })
        .collect());

    let props = prop_tree_to_list(&PropTree::new(&ctx).await?);

    Ok(Json(ListInputSourcesResponse { sockets, props }))
}
