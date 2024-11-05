use axum::{
    extract::{OriginalUri, Path, Query},
    Json,
};
use dal::func::binding::FuncBinding;
use dal::{ChangeSetId, DalContext, Func, SchemaId, SchemaVariant, SchemaVariantId, WorkspacePk};
use serde::{Deserialize, Serialize};
use si_frontend_types::{self as frontend_types, FuncSummary};
use std::collections::HashMap;
use telemetry::prelude::*;

use super::FuncAPIResult;
use crate::extract::{AccessBuilder, HandlerContext, PosthogClient};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListFuncRequest {
    page: Option<usize>,
    page_size: Option<usize>,
}
#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListFuncResponse {
    total: usize,
    funcs: Vec<frontend_types::FuncSummary>,
}

pub async fn list_funcs(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    PosthogClient(_posthog_client): PosthogClient,
    OriginalUri(_original_uri): OriginalUri,
    Path((_workspace_pk, change_set_id)): Path<(WorkspacePk, ChangeSetId)>,
    Query(request): Query<ListFuncRequest>,
) -> FuncAPIResult<Json<ListFuncResponse>> {
    let ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;
    let mut funcs = Vec::new();

    for func in Func::list_all(&ctx).await? {
        match treat_single_function(&ctx, &func).await {
            Ok(None) => {}
            Ok(Some(f)) => {
                funcs.push(f);
            }
            Err(err) => {
                error!(
                    ?err,
                    "could not make func with id {} into frontend type", func.id
                )
            }
        }
    }

    // sort by func id
    funcs.sort_by_key(|func| func.func_id);
    // now paginate
    let total = funcs.len();
    let chunked_funcs = paginate(funcs, request.page, request.page_size);
    Ok(Json(ListFuncResponse {
        total,
        funcs: chunked_funcs,
    }))
}

fn paginate(
    funcs: Vec<FuncSummary>,
    page: Option<usize>,
    page_size: Option<usize>,
) -> Vec<FuncSummary> {
    let page_size = page_size.unwrap_or(100);
    let target_page = page.unwrap_or(1);

    let mut current_page = 1;
    for chunk in funcs.chunks(page_size) {
        if current_page == target_page {
            return chunk.to_vec();
        }
        current_page += 1;
    }
    Vec::new()
}

async fn treat_single_function(
    ctx: &DalContext,
    func: &Func,
) -> FuncAPIResult<Option<frontend_types::FuncSummary>> {
    // compute bindings
    let bindings = FuncBinding::for_func_id(ctx, func.id).await?;

    // check if it is to be filtered away
    let mut schema_default_map: HashMap<SchemaId, SchemaVariantId> = HashMap::new();
    let mut unlocked_map: HashMap<SchemaVariantId, bool> = HashMap::new();

    // If func is unlocked or intrinsic, we always use it,
    // otherwise we return funcs that are bound to default variants
    // OR not bound to anything, OR editing variants
    // OR bound to variants with components on the canvas
    if func.is_locked && !func.is_intrinsic() && !bindings.is_empty() {
        let mut should_hide = true;
        for binding in &bindings {
            let Some(schema_variant_id) = binding.get_schema_variant() else {
                continue;
            };

            let maybe_existing_components =
                SchemaVariant::list_component_ids(ctx, schema_variant_id).await?;
            if !maybe_existing_components.is_empty() {
                should_hide = false;
            }

            let schema =
                SchemaVariant::schema_id_for_schema_variant_id(ctx, schema_variant_id).await?;

            if let Some(default_sv_id) = schema_default_map.get(&schema) {
                if schema_variant_id == *default_sv_id {
                    should_hide = false;
                }
            } else {
                let default_for_schema =
                    SchemaVariant::get_default_id_for_schema(ctx, schema).await?;

                schema_default_map.insert(schema, default_for_schema);

                if default_for_schema == schema_variant_id {
                    should_hide = false;
                }
            }

            if let Some(is_unlocked) = unlocked_map.get(&schema_variant_id) {
                if *is_unlocked {
                    should_hide = false;
                }
            } else {
                let variant = SchemaVariant::get_by_id_or_error(ctx, schema_variant_id).await?;
                if !variant.is_locked() {
                    should_hide = false;
                }
                unlocked_map.insert(schema_variant_id, !variant.is_locked());
            }
        }
        if should_hide {
            return Ok(None);
        }
    }

    // Convert to frontend type
    Ok(Some(
        func.into_frontend_type_sideload_bindings(ctx, bindings)
            .await?,
    ))
}
