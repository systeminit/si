//! This module contains the [`AttributeResolverContext`], and its corresponding builder, [`AttributeResolverContextBuilder`].
//! The context can be scoped with varying levels of specificity, using an order of precedence.
//! The builder ensures the correct order of precedence is maintained whilst setting and unsetting
//! fields of specificity.
//!
//! The order of precedence is as follows (from least to most "specificity"):
//! - [`PropId`]
//! - [`SchemaId`]
//! - [`SchemaVariantId`]
//! - [`ComponentId`]
//! - [`SystemId`]

use serde::{Deserialize, Serialize};
use std::default::Default;
use thiserror::Error;

use crate::attribute_resolver::UNSET_ID_VALUE;
use crate::{ComponentId, PropId, SchemaId, SchemaVariantId, SystemId};

#[derive(Error, Debug)]
pub enum AttributeResolverContextError {
    #[error("attribute resolver context builder error: {0}")]
    AttributeResolverContextBuilder(#[from] AttributeResolverContextBuilderError),
}

pub type AttributeResolverContextResult<T> = Result<T, AttributeResolverContextError>;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct AttributeResolverContext {
    prop_id: PropId,
    schema_id: SchemaId,
    schema_variant_id: SchemaVariantId,
    component_id: ComponentId,
    system_id: SystemId,
}

impl AttributeResolverContext {
    pub fn prop_id(&self) -> PropId {
        self.prop_id
    }

    pub fn schema_id(&self) -> SchemaId {
        self.schema_id
    }

    pub fn schema_variant_id(&self) -> SchemaVariantId {
        self.schema_variant_id
    }

    pub fn component_id(&self) -> ComponentId {
        self.component_id
    }

    pub fn system_id(&self) -> SystemId {
        self.system_id
    }

    /// Determines if the context is "least specific" or everything is unset.
    pub fn is_least_specific(&self) -> bool {
        self.system_id == UNSET_ID_VALUE.into()
            && self.component_id == UNSET_ID_VALUE.into()
            && self.schema_variant_id == UNSET_ID_VALUE.into()
            && self.schema_id == UNSET_ID_VALUE.into()
    }

    /// Return a new [`AttributeResolverContext`] with the most specific piece of the current
    /// [`AttributeResolverContext`] unset, widening the scope of the context by one step. If widening the
    /// context would result in everything being unset, it will return a new copy of the current
    /// [`AttributeResolverContext`].
    pub fn less_specific(&self) -> AttributeResolverContextResult<Self> {
        let mut builder = AttributeResolverContextBuilder::from_context(self);

        if self.system_id() != UNSET_ID_VALUE.into() {
            builder.unset_system_id();
        } else if self.component_id() != UNSET_ID_VALUE.into() {
            builder.unset_component_id();
        } else if self.schema_variant_id() != UNSET_ID_VALUE.into() {
            builder.unset_schema_variant_id();
        } else if self.schema_id() != UNSET_ID_VALUE.into() {
            builder.unset_schema_id();
        }

        Ok(builder.to_context()?)
    }
}

#[derive(Error, Debug)]
pub enum AttributeResolverContextBuilderError {
    #[error("for builder {0:?}, the following fields must be set: {1:?}")]
    PrerequisteFieldsUnset(AttributeResolverContextBuilder, Vec<&'static str>),
}

pub type AttributeResolverContextBuilderResult<T> = Result<T, AttributeResolverContextBuilderError>;

/// A builder with non-consuming "setter" and "unsetter" methods that verify the order of
/// precedence for [`AttributeResolverContext`].
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Copy)]
pub struct AttributeResolverContextBuilder {
    prop_id: PropId,
    schema_id: SchemaId,
    schema_variant_id: SchemaVariantId,
    component_id: ComponentId,
    system_id: SystemId,
}

/// Returns [`Self::new()`].
impl Default for AttributeResolverContextBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl AttributeResolverContextBuilder {
    /// Creates [`Self`] with all fields set to [`UNSET_ID_VALUE`].
    pub fn new() -> Self {
        Self {
            prop_id: UNSET_ID_VALUE.into(),
            schema_id: UNSET_ID_VALUE.into(),
            schema_variant_id: UNSET_ID_VALUE.into(),
            component_id: UNSET_ID_VALUE.into(),
            system_id: UNSET_ID_VALUE.into(),
        }
    }

    /// Converts [`AttributeResolverContext`] to [`Self`].
    pub fn from_context(context: &AttributeResolverContext) -> Self {
        Self {
            prop_id: context.prop_id(),
            schema_id: context.schema_id(),
            schema_variant_id: context.schema_variant_id(),
            component_id: context.component_id(),
            system_id: context.system_id(),
        }
    }

    /// Converts [`Self`] to [`AttributeResolverContext`]. This method will fail if the order of
    /// precedence is broken (i.e. more-specific fields are set, but one-to-all less-specific
    /// fields are unset).
    pub fn to_context(&self) -> AttributeResolverContextBuilderResult<AttributeResolverContext> {
        let mut unset_prerequisite_fields = Vec::new();

        // Start with the second highest specificty and work our way down.
        if self.component_id == UNSET_ID_VALUE.into() && self.system_id != UNSET_ID_VALUE.into() {
            unset_prerequisite_fields.push("ComponentId");
        }
        if self.schema_variant_id == UNSET_ID_VALUE.into()
            && (self.component_id != UNSET_ID_VALUE.into()
                || self.system_id != UNSET_ID_VALUE.into())
        {
            unset_prerequisite_fields.push("SchemaVariantId");
        }
        if self.schema_id == UNSET_ID_VALUE.into()
            && (self.schema_variant_id != UNSET_ID_VALUE.into()
                || self.component_id != UNSET_ID_VALUE.into()
                || self.system_id != UNSET_ID_VALUE.into())
        {
            unset_prerequisite_fields.push("SchemaId");
        }
        if self.prop_id == UNSET_ID_VALUE.into()
            && (self.schema_id != UNSET_ID_VALUE.into()
                || self.schema_variant_id != UNSET_ID_VALUE.into()
                || self.component_id != UNSET_ID_VALUE.into()
                || self.system_id != UNSET_ID_VALUE.into())
        {
            unset_prerequisite_fields.push("PropId");
        }

        if !unset_prerequisite_fields.is_empty() {
            return Err(
                AttributeResolverContextBuilderError::PrerequisteFieldsUnset(
                    *self,
                    unset_prerequisite_fields,
                ),
            );
        }

        Ok(AttributeResolverContext {
            prop_id: self.prop_id,
            schema_id: self.schema_id,
            schema_variant_id: self.schema_variant_id,
            component_id: self.component_id,
            system_id: self.system_id,
        })
    }

    /// Sets the [`PropId`] field. If [`UNSET_ID_VALUE`] is the ID passed in, then
    /// [`Self::unset_prop_id()`] is returned.
    pub fn set_prop_id(&mut self, prop_id: PropId) -> &mut Self {
        if prop_id == UNSET_ID_VALUE.into() {
            return self.unset_prop_id();
        }
        self.prop_id = prop_id;
        self
    }

    /// Sets the [`SchemaId`] field. If [`UNSET_ID_VALUE`] is the ID passed in, then
    /// [`Self::unset_schema_id()`] is returned.
    pub fn set_schema_id(&mut self, schema_id: SchemaId) -> &mut Self {
        if schema_id == UNSET_ID_VALUE.into() {
            return self.unset_schema_id();
        }
        self.schema_id = schema_id;
        self
    }

    /// Sets the [`SchemaVariantId`] field. If [`UNSET_ID_VALUE`] is the ID passed in, then
    /// [`Self::unset_schema_variant_id()`] is returned.
    pub fn set_schema_variant_id(&mut self, schema_variant_id: SchemaVariantId) -> &mut Self {
        if schema_variant_id == UNSET_ID_VALUE.into() {
            return self.unset_schema_variant_id();
        }
        self.schema_variant_id = schema_variant_id;
        self
    }

    /// Sets the [`ComponentId`] field. If [`UNSET_ID_VALUE`] is the ID passed in, then
    /// [`Self::unset_component_id()`] is returned.
    pub fn set_component_id(&mut self, component_id: ComponentId) -> &mut Self {
        if component_id == UNSET_ID_VALUE.into() {
            return self.unset_component_id();
        }
        self.component_id = component_id;
        self
    }

    /// Sets the [`SystemId`] field. If [`UNSET_ID_VALUE`] is the ID passed in, then
    /// [`Self::unset_system_id()`] is returned.
    pub fn set_system_id(&mut self, system_id: SystemId) -> &mut Self {
        if system_id == UNSET_ID_VALUE.into() {
            return self.unset_system_id();
        }
        self.system_id = system_id;
        self
    }

    /// Unsets the [`PropId`].
    pub fn unset_prop_id(&mut self) -> &mut Self {
        self.prop_id = UNSET_ID_VALUE.into();
        self
    }

    /// Unsets the [`SchemaId`].
    pub fn unset_schema_id(&mut self) -> &mut Self {
        self.schema_id = UNSET_ID_VALUE.into();
        self
    }

    /// Unsets the [`SchemaVariantId`].
    pub fn unset_schema_variant_id(&mut self) -> &mut Self {
        self.schema_variant_id = UNSET_ID_VALUE.into();
        self
    }

    /// Unsets the [`ComponentId`].
    pub fn unset_component_id(&mut self) -> &mut Self {
        self.component_id = UNSET_ID_VALUE.into();
        self
    }

    /// Unsets the [`SystemId`].
    pub fn unset_system_id(&mut self) -> &mut Self {
        self.system_id = UNSET_ID_VALUE.into();
        self
    }
}
