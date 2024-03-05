use dal::{
    DalContext, InputSocketId, OutputSocketId, Schema, SchemaId, SchemaVariant, SchemaVariantId,
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

            let (output_sockets, input_sockets) =
                SchemaVariant::list_all_sockets(ctx, schema_variant.id())
                    .await
                    .expect("could not list all sockets");

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
                input_sockets: input_sockets
                    .iter()
                    .map(|s| InputSocketView {
                        id: s.id(),
                        name: s.name().to_owned(),
                    })
                    .collect(),
                output_sockets: output_sockets
                    .iter()
                    .map(|s| OutputSocketView {
                        id: s.id(),
                        name: s.name().to_owned(),
                    })
                    .collect(),
            });
        }
    }

    dbg!(schema_variant_views);
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct OutputSocketView {
    id: OutputSocketId,
    name: String,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct InputSocketView {
    id: InputSocketId,
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
