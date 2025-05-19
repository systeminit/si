use dal::{
    DalContext,
    Prop,
    SchemaVariant,
    prop::PropPath,
};
use si_frontend_mv_types::secret::SecretDefinitionList as SecretDefinitionListMv;
use telemetry::prelude::*;

#[instrument(
    name = "dal_materialized_views.secret_definition_list.assemble",
    level = "debug",
    skip_all
)]
pub async fn assemble(ctx: DalContext) -> crate::Result<SecretDefinitionListMv> {
    let id = ctx.change_set_id();
    let ctx = &ctx;
    //todo: we probably need to also find schema variants with active secrets even if they're not the default
    let schema_variant_ids = SchemaVariant::list_default_secret_defining_ids(ctx).await?;
    let secret_definition_path = PropPath::new(["root", "secret_definition"]);
    let mut definitions = Vec::new();

    for schema_variant_id in schema_variant_ids {
        let maybe_secret_definition_prop_id =
            Prop::find_prop_id_by_path_opt(ctx, schema_variant_id, &secret_definition_path).await?;

        // We have found a schema variant with a secret definition!
        if let Some(secret_definition_prop_id) = maybe_secret_definition_prop_id {
            definitions.push(secret_definition_prop_id);
        }
    }
    definitions.sort();
    Ok(SecretDefinitionListMv {
        secret_definitions: definitions.iter().map(|&prop_id| prop_id.into()).collect(),
        id,
    })
}
