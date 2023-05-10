use super::FuncResult;
use crate::server::extract::{AccessBuilder, HandlerContext};
use axum::{extract::Query, Json};
use dal::{
    prop_tree::PropTree, ExternalProvider, ExternalProviderId, InternalProvider,
    InternalProviderId, PropId, PropKind, SchemaVariantId, StandardModel, Visibility,
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
pub struct OutputSocket {
    pub schema_variant_id: SchemaVariantId,
    pub external_provider_id: ExternalProviderId,
    pub name: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InputSourceProp {
    pub schema_variant_id: SchemaVariantId,
    pub internal_provider_id: Option<InternalProviderId>,
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
    pub input_sockets: Vec<InputSourceSocket>,
    pub output_sockets: Vec<OutputSocket>,
    pub props: Vec<InputSourceProp>,
}

// We take the tree and recompose it as a list using depth-first search, filtering out props that
// do not have an internal provider id (and thus cannot be used as function input sources)
// we have to recompose it as a list to ensure props are listed in the correct order, since
// the SQL query is limited in some respects.
fn prop_tree_to_list(prop_tree: &PropTree) -> Vec<InputSourceProp> {
    let mut prop_sources = vec![];

    for root in &prop_tree.root_props {
        let mut work_queue = VecDeque::from([root]);

        while let Some(cur) = work_queue.pop_front() {
            // Don't add the children of arrays or maps (yet!)
            match cur.kind {
                PropKind::Array | PropKind::Map => {}
                _ => {
                    for child in &cur.children {
                        work_queue.push_front(child);
                    }
                }
            }

            prop_sources.push(InputSourceProp {
                schema_variant_id: cur.schema_variant_id,
                internal_provider_id: cur.internal_provider_id,
                prop_id: cur.prop_id,
                kind: cur.kind,
                name: cur.name.clone(),
                path: cur.path.clone(),
            });
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

    let input_sockets = InternalProvider::list_for_input_sockets(&ctx)
        .await?
        .iter()
        .map(|ip| InputSourceSocket {
            internal_provider_id: *ip.id(),
            schema_variant_id: *ip.schema_variant_id(),
            name: ip.name().to_owned(),
        })
        .collect();

    let output_sockets = ExternalProvider::list(&ctx)
        .await?
        .iter()
        .map(|ep| OutputSocket {
            external_provider_id: *ep.id(),
            schema_variant_id: *ep.schema_variant_id(),
            name: ep.name().to_owned(),
        })
        .collect();

    let props = prop_tree_to_list(&PropTree::new(&ctx, true, None).await?);

    Ok(Json(ListInputSourcesResponse {
        input_sockets,
        output_sockets,
        props,
    }))
}
