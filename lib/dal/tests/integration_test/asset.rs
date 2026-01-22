use dal::{
    DalContext,
    SchemaVariant,
    SchemaVariantId,
    schema::variant::{
        DEFAULT_SCHEMA_VARIANT_COLOR,
        authoring::{
            VariantAuthoringClient,
            VariantAuthoringError,
        },
    },
};
use dal_test::{
    helpers::ChangeSetTestHelpers,
    test,
};

#[test(enable_veritech)]
async fn asset_func_execution_papercuts(ctx: &mut DalContext) {
    let (schema_name, unstable_schema_variant_id) = {
        let schema_variant = VariantAuthoringClient::create_schema_and_variant(
            ctx,
            "name",
            None::<String>,
            None::<String>,
            "category",
            DEFAULT_SCHEMA_VARIANT_COLOR,
        )
        .await
        .expect("could not create schema and variant");
        let schema = schema_variant
            .schema(ctx)
            .await
            .expect("could not get schema");
        ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
            .await
            .expect("could not commit and update snapshot to visibility");
        (schema.name, schema_variant.id)
    };

    // PASS - save with the default code and regenerate
    let code = "function main() { return new AssetBuilder().build(); }";
    let unstable_schema_variant_id =
        save_and_regenerate(ctx, code, &schema_name, unstable_schema_variant_id)
            .await
            .expect("could not save and regenerate");

    // FAIL - create a socket, but drop the build call
    let code = "function main() {
      const socket = new SocketDefinitionBuilder()
        .setName(\"socket\")
        .setArity(\"many\")
        .build();
      return new AssetBuilder()
        .addOutputSocket(socket);
    }";
    match save_and_regenerate(ctx, code, &schema_name, unstable_schema_variant_id).await {
        Ok(_) => panic!("expected failure upon regeneration"),
        Err(box_err) => match box_err.downcast_ref::<VariantAuthoringError>() {
            Some(err) => match err {
                VariantAuthoringError::AssetTypeNotReturnedForAssetFunc(_, _) => {}
                err => panic!("{err:?}"),
            },
            None => panic!("{box_err:?}"),
        },
    }

    // FAIL - mmediately return the builder and regenerate
    let code = "function main() {
      const socket = new SocketDefinitionBuilder()
        .setName(\"socket\")
        .setArity(\"many\")
        .build();
      return new AssetBuilder()
        .addOutputSocket(socket);
    }";
    match save_and_regenerate(ctx, code, &schema_name, unstable_schema_variant_id).await {
        Ok(_) => panic!("expected failure upon regeneration"),
        Err(box_err) => match box_err.downcast_ref::<VariantAuthoringError>() {
            Some(err) => match err {
                VariantAuthoringError::AssetTypeNotReturnedForAssetFunc(_, _) => {}
                err => panic!("{err:?}"),
            },
            None => panic!("{box_err:?}"),
        },
    }

    // PASS - add the build call when creating a socket
    let code = "function main() {
      const socket = new SocketDefinitionBuilder()
        .setName(\"socket\")
        .setArity(\"many\")
        .build();
      return new AssetBuilder()
        .addOutputSocket(socket)
        .build();
    }";
    let _unstable_schema_variant_id =
        save_and_regenerate(ctx, code, &schema_name, unstable_schema_variant_id)
            .await
            .expect("could not save and regenerate");
}

async fn save_and_regenerate(
    ctx: &mut DalContext,
    code: impl Into<String>,
    schema_name: impl AsRef<str>,
    schema_variant_id: SchemaVariantId,
) -> Result<SchemaVariantId, Box<dyn std::error::Error>> {
    let schema_variant = SchemaVariant::get_by_id(ctx, schema_variant_id).await?;
    let schema_name = schema_name.as_ref();

    VariantAuthoringClient::save_variant_content(
        ctx,
        schema_variant.id,
        schema_name,
        schema_variant.display_name(),
        schema_variant.category(),
        schema_variant.description(),
        schema_variant.link(),
        schema_variant.get_color(ctx).await?,
        schema_variant.component_type(),
        Some(code),
    )
    .await?;
    let updated_schema_variant_id =
        VariantAuthoringClient::regenerate_variant(ctx, schema_variant.id).await?;

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    Ok(updated_schema_variant_id)
}
