use axum::{
    extract::{OriginalUri, Path},
    Json,
};
use dal::func::binding::FuncBinding;
use dal::{ChangeSetId, DalContext, Func, SchemaId, SchemaVariant, SchemaVariantId, WorkspacePk};
use si_frontend_types as frontend_types;
use std::collections::HashMap;
use telemetry::prelude::*;

use super::FuncAPIResult;
use crate::extract::{AccessBuilder, HandlerContext, PosthogClient};

pub async fn list_funcs(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    PosthogClient(_posthog_client): PosthogClient,
    OriginalUri(_original_uri): OriginalUri,
    Path((_workspace_pk, change_set_id)): Path<(WorkspacePk, ChangeSetId)>,
) -> FuncAPIResult<Json<Vec<frontend_types::FuncSummary>>> {
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
    Ok(Json(funcs))
}

async fn treat_single_function(
    ctx: &DalContext,
    func: &Func,
) -> FuncAPIResult<Option<frontend_types::FuncSummary>> {
    // compute bindings
    let bindings = FuncBinding::for_func_id(ctx, func.id).await?;

    // check if it is to be filtered away
    let mut schema_default_map: HashMap<SchemaId, SchemaVariantId> = HashMap::new();

    // If func is unlocked or intrinsic, we always use it,
    // otherwise we return funcs that are bound to default variants
    // OR not bound anything
    if func.is_locked && !func.is_intrinsic() && !bindings.is_empty() {
        let mut bindings_to_default_svs = vec![];
        for binding in &bindings {
            let Some(schema_variant_id) = binding.get_schema_variant() else {
                continue;
            };

            let schema =
                SchemaVariant::schema_id_for_schema_variant_id(ctx, schema_variant_id).await?;

            if let Some(default_sv_id) = schema_default_map.get(&schema) {
                if schema_variant_id == *default_sv_id {
                    bindings_to_default_svs.push(binding);
                }
            } else {
                let default_for_schema =
                    SchemaVariant::get_default_id_for_schema(ctx, schema).await?;

                schema_default_map.insert(schema, default_for_schema);

                if default_for_schema == schema_variant_id {
                    bindings_to_default_svs.push(binding);
                }
            }
        }
        if bindings_to_default_svs.is_empty() {
            return Ok(None);
        }
    }

    // Convert to frontend type
    Ok(Some(
        func.into_frontend_type_sideload_bindings(ctx, bindings)
            .await?,
    ))
}
