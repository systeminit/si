use dal::{
    DalContext, ExternalProviderId, InternalProviderId, Schema, SchemaId, SchemaVariant,
    SchemaVariantId,
};
use dal_test::test;

#[test]
async fn list_schema_variant_views(ctx: &DalContext) {
    let mut schema_variant_views: Vec<SchemaVariantView> = Vec::new();

    let schemas = Schema::list(ctx).await.expect("could not list schemas");
    for schema in schemas {
        if schema.ui_hidden {
            continue;
        }

        let schema_variants = SchemaVariant::list_for_schema(ctx, schema.id())
            .await
            .expect("could not list schema variants for schema");
        for schema_variant in schema_variants {
            if schema_variant.ui_hidden() {
                continue;
            }

            let mut input_sockets = Vec::new();
            let mut output_sockets = Vec::new();

            let (external_providers, explicit_internal_providers) =
                SchemaVariant::list_external_providers_and_explicit_internal_providers(
                    ctx,
                    schema_variant.id(),
                )
                .await
                .expect("could not list external providers and explicit internal providers");

            for explicit_internal_provider in explicit_internal_providers {
                input_sockets.push(InputSocketView {
                    id: explicit_internal_provider.id(),
                    name: explicit_internal_provider.name().to_owned(),
                })
            }

            for external_provider in external_providers {
                output_sockets.push(OutputSocketView {
                    id: external_provider.id(),
                    name: external_provider.name().to_owned(),
                })
            }

            schema_variant_views.push(SchemaVariantView {
                id: schema_variant.id(),
                // FIXME(nick): use the real value here
                builtin: true,
                // builtin: schema_variant.is_builtin(ctx).await?,
                name: schema_variant.name().to_owned(),
                schema_id: schema.id(),
                schema_name: schema.name.to_owned(),
                color: schema_variant
                    .get_color(ctx)
                    .await
                    .expect("could not get color")
                    .unwrap_or("#0F0F0F".into()),
                category: schema_variant.category().to_owned(),
                input_sockets,
                output_sockets,
            });
        }
    }

    dbg!(schema_variant_views);
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct OutputSocketView {
    id: ExternalProviderId,
    name: String,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct InputSocketView {
    id: InternalProviderId,
    name: String,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct SchemaVariantView {
    id: SchemaVariantId,
    builtin: bool,
    name: String,
    schema_name: String,
    schema_id: SchemaId,
    color: String,
    category: String,
    input_sockets: Vec<InputSocketView>,
    output_sockets: Vec<OutputSocketView>,
}
