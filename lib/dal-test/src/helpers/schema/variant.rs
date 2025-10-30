use dal::{
    DalContext,
    FuncId,
    Schema,
    SchemaVariant,
    SchemaVariantId,
    func::{
        authoring::FuncAuthoringClient,
        binding::EventualParent,
        leaf::{
            LeafInputLocation,
            LeafKind,
        },
    },
    schema::variant::authoring::VariantAuthoringClient,
};

use crate::{
    Result,
    expected::ExpectSchemaVariant,
};

/// Lookup a schema variant by name or id
pub async fn id(ctx: &DalContext, key: impl SchemaVariantKey) -> Result<SchemaVariantId> {
    SchemaVariantKey::id(ctx, key).await
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
    name: impl Into<String>,
    code: impl Into<String>,
) -> Result<FuncId> {
    let variant_id = id(ctx, variant).await?;
    let func =
        FuncAuthoringClient::create_new_management_func(ctx, Some(name.into()), variant_id).await?;
    FuncAuthoringClient::save_code(ctx, func.id, code.into()).await?;
    Ok(func.id)
}

/// Update the asset function for the given schema variant, and regenerate the variant
pub async fn update_asset_func(
    ctx: &DalContext,
    variant: impl SchemaVariantKey,
    asset_func: impl Into<String>,
) -> Result<()> {
    let variant_id = id(ctx, variant).await?;
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

/// Create a qualification function for the given schema variant
pub async fn create_qualification_func(
    ctx: &DalContext,
    variant: impl SchemaVariantKey,
    name: impl Into<String>,
    code: impl Into<String>,
    inputs: &[LeafInputLocation],
) -> Result<FuncId> {
    let variant_id = id(ctx, variant).await?;
    let func = FuncAuthoringClient::create_new_leaf_func(
        ctx,
        Some(name.into()),
        LeafKind::Qualification,
        EventualParent::SchemaVariant(variant_id),
        inputs,
    )
    .await?;
    FuncAuthoringClient::save_code(ctx, func.id, code.into()).await?;
    Ok(func.id)
}

///
/// Things that you can pass as schema variants (schema name or variant id)
///
#[allow(async_fn_in_trait)]
pub trait SchemaVariantKey {
    ///
    /// Turn this into a real SchemaVariantId
    ///
    async fn id(ctx: &DalContext, key: Self) -> Result<SchemaVariantId>;
}
impl SchemaVariantKey for SchemaVariantId {
    async fn id(_: &DalContext, key: Self) -> Result<SchemaVariantId> {
        Ok(key)
    }
}
// "SchemaName" resolves to the default variant for the schema
impl SchemaVariantKey for &str {
    async fn id(ctx: &DalContext, key: Self) -> Result<SchemaVariantId> {
        let schema = Schema::get_by_name(ctx, key).await?;
        Ok(SchemaVariant::default_id_for_schema(ctx, schema.id()).await?)
    }
}
impl SchemaVariantKey for ExpectSchemaVariant {
    async fn id(_: &DalContext, key: Self) -> Result<SchemaVariantId> {
        Ok(key.id())
    }
}
impl SchemaVariantKey for SchemaVariant {
    async fn id(_: &DalContext, key: Self) -> Result<SchemaVariantId> {
        Ok(key.id())
    }
}
