use dal::{
    DalContext,
    SchemaVariant,
    SchemaVariantId,
};
use si_frontend_mv_types::schema_variant::SchemaVariant as SchemaVariantMv;
use telemetry::prelude::*;

use crate::mgmt_prototype_view_list;
pub mod prop_tree;

#[instrument(
    name = "dal_materialized_views.schema_variant",
    level = "debug",
    skip_all
)]
pub async fn assemble(ctx: DalContext, id: SchemaVariantId) -> super::Result<SchemaVariantMv> {
    let schema_variant = SchemaVariant::get_by_id(&ctx, id).await?;
    let schema_id = schema_variant.schema(&ctx).await?.id();
    let sv = schema_variant.into_frontend_type(&ctx, schema_id).await?;
    let is_secret_defining = SchemaVariant::is_secret_defining(&ctx, id).await?;
    let mgmt_functions = mgmt_prototype_view_list::assemble(&ctx, id).await?;
    let prop_tree = prop_tree::assemble(ctx, id).await?;
    Ok(SchemaVariantMv {
        id: sv.schema_variant_id,
        schema_variant_id: sv.schema_variant_id,
        schema_id,
        schema_name: sv.schema_name,
        version: sv.version,
        display_name: sv.display_name,
        category: sv.category,
        description: sv.description,
        link: sv.link,
        color: sv.color,
        is_locked: sv.is_locked,
        timestamp: sv.timestamp,
        can_create_new_components: sv.can_create_new_components,
        can_contribute: sv.can_contribute,
        mgmt_functions,
        prop_tree,
        is_secret_defining,
    })
}
