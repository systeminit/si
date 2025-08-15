use dal::{
    DalContext,
    cached_module::CachedModule,
};
use si_frontend_mv_types::cached_schemas::{
    CachedSchema,
    CachedSchemas as CachedSchemasMv,
    CachedSchemas,
};
use telemetry::prelude::*;

#[instrument(
    name = "dal_materialized_views.cached_schemas",
    level = "debug",
    skip_all
)]
pub async fn assemble(ctx: DalContext) -> crate::Result<CachedSchemasMv> {
    let mut schemas = vec![];

    for module in CachedModule::latest_modules(&ctx).await? {
        schemas.push(CachedSchema {
            id: module.schema_id,
            name: module.schema_name,
        });
    }

    // Try to get the order to be stable so that the checksums match for the content
    schemas.sort_by(|a, b| a.id.cmp(&b.id));

    Ok(CachedSchemas::new(schemas))
}
