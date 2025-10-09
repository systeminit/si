use dal::{
    DalContext,
    FuncId,
    Schema,
    SchemaId,
    func::authoring::FuncAuthoringClient,
};

use crate::{
    Result,
    expected::ExpectSchema,
};

/// Test helpers for schema variants
pub mod variant;

/// Lookup a schema by name or id
pub async fn id(ctx: &DalContext, key: impl SchemaKey) -> Result<SchemaId> {
    SchemaKey::id(ctx, key).await
}

///
/// Things that you can pass as schema ids
///
#[allow(async_fn_in_trait)]
pub trait SchemaKey {
    ///
    /// Turn this into a real SchemaId
    ///
    async fn id(ctx: &DalContext, key: Self) -> Result<SchemaId>;
}
impl SchemaKey for SchemaId {
    async fn id(_: &DalContext, key: Self) -> Result<SchemaId> {
        Ok(key)
    }
}
impl SchemaKey for ExpectSchema {
    async fn id(_: &DalContext, key: Self) -> Result<SchemaId> {
        Ok(key.id())
    }
}
impl SchemaKey for Schema {
    async fn id(_: &DalContext, key: Self) -> Result<SchemaId> {
        Ok(key.id())
    }
}
impl SchemaKey for &str {
    async fn id(ctx: &DalContext, key: Self) -> Result<SchemaId> {
        Ok(Schema::get_by_name(ctx, key).await?.id())
    }
}

/// Create a management func overaly for the given schema
pub async fn create_overlay_management_func(
    ctx: &DalContext,
    schema: impl SchemaKey,
    name: impl Into<String>,
    code: impl Into<String>,
) -> Result<FuncId> {
    let schema_id = SchemaKey::id(ctx, schema).await?;
    let func =
        FuncAuthoringClient::create_new_management_func(ctx, Some(name.into()), schema_id).await?;
    FuncAuthoringClient::save_code(ctx, func.id, code.into()).await?;
    Ok(func.id)
}
