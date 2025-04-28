#![allow(async_fn_in_trait)]

use dal::{
    DalContext,
    Schema,
    SchemaId,
};

use crate::{
    Result,
    expected::ExpectSchema,
};

/// Test helpers for schema variants
pub mod variant;

///
/// Things that you can pass as schema ids
///
pub trait SchemaKey {
    ///
    /// Turn this into a real SchemaId
    ///
    async fn lookup_schema(self, ctx: &DalContext) -> Result<SchemaId>;
}
impl SchemaKey for SchemaId {
    async fn lookup_schema(self, _: &DalContext) -> Result<SchemaId> {
        Ok(self)
    }
}
impl SchemaKey for ExpectSchema {
    async fn lookup_schema(self, _: &DalContext) -> Result<SchemaId> {
        Ok(self.id())
    }
}
impl SchemaKey for Schema {
    async fn lookup_schema(self, _: &DalContext) -> Result<SchemaId> {
        Ok(self.id())
    }
}
impl SchemaKey for &str {
    async fn lookup_schema(self, ctx: &DalContext) -> Result<SchemaId> {
        Ok(Schema::get_by_name(ctx, self).await?.id())
    }
}
