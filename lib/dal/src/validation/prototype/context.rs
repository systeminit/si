//! This module contains [`ValidationPrototypeContext`] and its corresponding builder, which
//! are used to create and access [`ValidationPrototypes`](crate::ValidationPrototype).

use serde::{Deserialize, Serialize};

use crate::validation::prototype::ValidationPrototypeResult;
use crate::{
    DalContext, Prop, PropId, PropKind, SchemaId, SchemaVariantId, StandardModel,
    ValidationPrototypeError,
};

/// The builder used to create a valid [`ValidationPrototypeContext`].
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct ValidationPrototypeContextBuilder {
    prop_id: PropId,
    schema_id: SchemaId,
    schema_variant_id: SchemaVariantId,
}

impl ValidationPrototypeContextBuilder {
    fn new() -> Self {
        Self {
            prop_id: PropId::NONE,
            schema_id: SchemaId::NONE,
            schema_variant_id: SchemaVariantId::NONE,
        }
    }

    pub fn prop_id(&self) -> PropId {
        self.prop_id
    }

    pub fn schema_id(&self) -> SchemaId {
        self.schema_id
    }

    pub fn schema_variant_id(&self) -> SchemaVariantId {
        self.schema_variant_id
    }

    pub fn set_prop_id(&mut self, prop_id: PropId) -> &mut Self {
        self.prop_id = prop_id;
        self
    }

    pub fn set_schema_id(&mut self, schema_id: SchemaId) -> &mut Self {
        self.schema_id = schema_id;
        self
    }

    pub fn set_schema_variant_id(&mut self, schema_variant_id: SchemaVariantId) -> &mut Self {
        self.schema_variant_id = schema_variant_id;
        self
    }

    /// Try to convert [`Self`] into a [`ValidationPrototypeContext`].
    pub async fn to_context(
        &self,
        ctx: &DalContext,
    ) -> ValidationPrototypeResult<ValidationPrototypeContext> {
        let mut unset_prerequisite_fields = Vec::new();

        // Start with the second highest specific and work our way down.
        if self.schema_id == SchemaId::NONE && self.schema_variant_id != SchemaVariantId::NONE {
            unset_prerequisite_fields.push("SchemaId");
        }

        // The lowest level in the order of precedence must always be set.
        if self.prop_id == PropId::NONE {
            unset_prerequisite_fields.push("PropId");
        }

        if !unset_prerequisite_fields.is_empty() {
            return Err(ValidationPrototypeError::PrerequisteFieldsUnset(
                self.clone(),
                unset_prerequisite_fields,
            ));
        }

        let prop = Prop::get_by_id(ctx, &self.prop_id)
            .await?
            .ok_or(ValidationPrototypeError::PropNotFound(self.prop_id))?;
        let prop_kind = prop.kind();
        if prop_kind != &PropKind::String
            && prop_kind != &PropKind::Integer
            && prop_kind != &PropKind::Boolean
        {
            return Err(ValidationPrototypeError::ContextPropKindIsNotPrimitive(
                *prop_kind,
            ));
        }

        Ok(ValidationPrototypeContext {
            prop_id: self.prop_id,
            schema_id: self.schema_id,
            schema_variant_id: self.schema_variant_id,
        })
    }
}

impl From<ValidationPrototypeContext> for ValidationPrototypeContextBuilder {
    fn from(context: ValidationPrototypeContext) -> Self {
        let mut builder = Self::new();
        builder
            .set_prop_id(context.prop_id)
            .set_schema_id(context.schema_id)
            .set_schema_variant_id(context.schema_variant_id);
        builder
    }
}

/// The context used to create and access [`ValidationPrototypes`](crate::ValidationPrototype).
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct ValidationPrototypeContext {
    prop_id: PropId,
    schema_id: SchemaId,
    schema_variant_id: SchemaVariantId,
}

impl ValidationPrototypeContext {
    /// Create a new [`ValidationPrototypeContextBuilder`].
    pub fn builder() -> ValidationPrototypeContextBuilder {
        ValidationPrototypeContextBuilder::new()
    }

    /// Create a new [`ValidationPrototypeContext`] from raw fields
    pub fn new_unchecked(
        prop_id: PropId,
        schema_variant_id: SchemaVariantId,
        schema_id: SchemaId,
    ) -> Self {
        Self {
            prop_id,
            schema_variant_id,
            schema_id,
        }
    }

    /// Convert [`Self`] into [`ValidationPrototypeContextBuilder`].
    pub fn to_builder(&self) -> ValidationPrototypeContextBuilder {
        ValidationPrototypeContextBuilder::from(self.clone())
    }

    pub fn prop_id(&self) -> PropId {
        self.prop_id
    }

    pub fn schema_id(&self) -> SchemaId {
        self.schema_id
    }

    pub fn schema_variant_id(&self) -> SchemaVariantId {
        self.schema_variant_id
    }
}
