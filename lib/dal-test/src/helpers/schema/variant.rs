use dal::{
    DalContext,
    FuncId,
    Schema,
    SchemaVariant,
    SchemaVariantId,
    func::authoring::FuncAuthoringClient,
    schema::variant::authoring::VariantAuthoringClient,
};

use crate::{
    Result,
    expected::ExpectSchemaVariant,
};

// NOTE Most of these should just be in VariantAuthoringClient

///
/// Things that you can pass as schema variants (schema name or variant id)
///
#[allow(async_fn_in_trait)]
pub trait SchemaVariantKey {
    ///
    /// Turn this into a real SchemaVariantId
    ///
    async fn lookup_schema_variant(self, ctx: &DalContext) -> Result<SchemaVariantId>;
}
impl SchemaVariantKey for SchemaVariantId {
    async fn lookup_schema_variant(self, _: &DalContext) -> Result<SchemaVariantId> {
        Ok(self)
    }
}
// "SchemaName" resolves to the default variant for the schema
impl SchemaVariantKey for &str {
    async fn lookup_schema_variant(self, ctx: &DalContext) -> Result<SchemaVariantId> {
        let schema = Schema::get_by_name(ctx, self).await?;
        Ok(SchemaVariant::default_id_for_schema(ctx, schema.id()).await?)
    }
}
impl SchemaVariantKey for ExpectSchemaVariant {
    async fn lookup_schema_variant(self, _: &DalContext) -> Result<SchemaVariantId> {
        Ok(self.id())
    }
}
impl SchemaVariantKey for SchemaVariant {
    async fn lookup_schema_variant(self, _: &DalContext) -> Result<SchemaVariantId> {
        Ok(self.id())
    }
}

/// Create a schema + schema variant with the given name and asset function
pub async fn create(
    ctx: &DalContext,
    name: impl Into<String>,
    asset_func: impl Into<String>,
) -> Result<SchemaVariantId> {
    // Create an asset with a corresponding asset func. After that, commit.
    let variant =
        VariantAuthoringClient::create_schema_and_variant(ctx, name, None, None, "test", "FFFFFF")
            .await?;
    update_asset_func(ctx, variant.id, asset_func).await?;
    VariantAuthoringClient::regenerate_variant(ctx, variant.id).await?;
    Ok(variant.id)
}

/// Create a management function for the given schema variant
pub async fn create_management_func(
    ctx: &DalContext,
    variant: impl SchemaVariantKey,
    code: impl Into<String>,
) -> Result<FuncId> {
    let variant_id = variant.lookup_schema_variant(ctx).await?;
    let func = FuncAuthoringClient::create_new_management_func(ctx, None, variant_id).await?;
    FuncAuthoringClient::save_code(ctx, func.id, code.into()).await?;
    Ok(func.id)
}

/// Update the asset function for the given schema variant, and regenerate the variant
pub async fn update_asset_func(
    ctx: &DalContext,
    variant: impl SchemaVariantKey,
    asset_func: impl Into<String>,
) -> Result<()> {
    let variant_id = variant.lookup_schema_variant(ctx).await?;
    let schema = SchemaVariant::schema_for_schema_variant_id(ctx, variant_id).await?;
    let variant = SchemaVariant::get_by_id(ctx, variant_id).await?;
    VariantAuthoringClient::save_variant_content(
        ctx,
        variant_id,
        schema.name,
        variant.display_name(),
        variant.category(),
        variant.description(),
        variant.link(),
        variant.color(),
        variant.component_type(),
        Some(asset_func.into()),
    )
    .await?;
    VariantAuthoringClient::regenerate_variant(ctx, variant.id).await?;
    Ok(())
}
