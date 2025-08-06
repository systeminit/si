use dal::{
    DalContext,
};
use dal::cached_module::CachedModule;
use si_frontend_mv_types::cached_schemas::{CachedSchema, CachedSchemas as CachedSchemasMv, CachedSchemas};
use telemetry::prelude::*;

#[instrument(
    name = "dal_materialized_views.cached_schemas",
    level = "debug",
    skip_all
)]
pub async fn assemble(ctx: DalContext) -> super::Result<CachedSchemasMv> {
    let ctx = &ctx;

    let mut schemas = vec![];

    for module in CachedModule::latest_modules(ctx).await? {
        schemas.push(CachedSchema {
            id: module.schema_id,
            name: module.schema_name,
        });
    };

    Ok(CachedSchemas::new(schemas))
}
