use serde::{Deserialize, Serialize};
use telemetry::prelude::*;

use crate::{DalContext, FuncId, SchemaVariant, SchemaVariantId};

use super::{FuncBinding, FuncBindingResult};

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct AuthBinding {
    // unique ids
    pub schema_variant_id: SchemaVariantId,
    pub func_id: FuncId,
}

impl AuthBinding {
    pub(crate) async fn assemble_auth_bindings(
        ctx: &DalContext,
        func_id: FuncId,
    ) -> FuncBindingResult<Vec<FuncBinding>> {
        let schema_variant_ids = SchemaVariant::list_for_auth_func(ctx, func_id).await?;
        let mut bindings = vec![];
        for schema_variant_id in schema_variant_ids {
            bindings.push(FuncBinding::Authentication(AuthBinding {
                schema_variant_id,
                func_id,
            }));
        }
        Ok(bindings)
    }

    #[instrument(
        level = "info",
        skip(ctx),
        name = "func.binding.authentication.create_auth_binding"
    )]
    /// Create an Auth Binding for a Schema Variant
    pub async fn create_auth_binding(
        ctx: &DalContext,
        func_id: FuncId,
        schema_variant_id: SchemaVariantId,
    ) -> FuncBindingResult<Vec<FuncBinding>> {
        // don't add binding if parent is locked
        SchemaVariant::error_if_locked(ctx, schema_variant_id).await?;

        SchemaVariant::new_authentication_prototype(ctx, func_id, schema_variant_id).await?;
        FuncBinding::for_func_id(ctx, func_id).await
    }

    #[instrument(
        level = "info",
        skip(ctx),
        name = "func.binding.authentication.create_auth_binding"
    )]
    /// Deletes an Auth Binding for a Schema Variant
    pub async fn delete_auth_binding(
        ctx: &DalContext,
        func_id: FuncId,
        schema_variant_id: SchemaVariantId,
    ) -> FuncBindingResult<Vec<FuncBinding>> {
        // don't delete binding if parent is locked
        SchemaVariant::error_if_locked(ctx, schema_variant_id).await?;
        SchemaVariant::remove_authentication_prototype(ctx, func_id, schema_variant_id).await?;
        FuncBinding::for_func_id(ctx, func_id).await
    }

    pub(crate) async fn port_binding_to_new_func(
        &self,
        ctx: &DalContext,
        new_func_id: FuncId,
    ) -> FuncBindingResult<Vec<FuncBinding>> {
        let schema_variant_id = self.schema_variant_id;

        // don't add binding if parent is locked
        // this shouldn't happen?
        SchemaVariant::error_if_locked(ctx, schema_variant_id).await?;

        Self::delete_auth_binding(ctx, self.func_id, self.schema_variant_id).await?;
        Self::create_auth_binding(ctx, new_func_id, schema_variant_id).await?;
        FuncBinding::for_func_id(ctx, new_func_id).await
    }
}
