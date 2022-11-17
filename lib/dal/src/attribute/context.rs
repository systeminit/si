//! This module contains the [`AttributeContext`], and its corresponding builder, [`AttributeContextBuilder`].
//! The context can be scoped with varying levels of specificity, using an order of precedence.
//! The builder ensures the correct order of precedence is maintained whilst setting and unsetting
//! fields of specificity.
//!
//! ## The Order of Precedence
//!
//! The order of precedence is as follows (from least to most "specificity"):
//! - [`PropId`] / [`InternalProviderId`] / [`ExternalProviderId`]
//! - [`SchemaId`]
//! - [`SchemaVariantId`]
//! - [`ComponentId`]
//!
//! At the level of least "specificity", you can provider have a [`PropId`], an
//! [`InternalProviderId`], or an [`ExternalProviderId`]. However, you can only provide one and only
//! one for an [`AttributeContext`] since they are at the same "level" in the order of precedence.
//!
//! ## `AttributeContext` vs. `AttributeReadContext`
//!
//! While the [`AttributeContext`] can be used for both read and write queries, the
//! [`AttributeReadContext`](crate::AttributeReadContext) is useful for read-only queries and for
//! flexibility when searching for objects of varying levels of specificity.

use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::default::Default;
use thiserror::Error;

use crate::{
    ComponentId, ExternalProviderId, InternalProviderId, PropId, SchemaId, SchemaVariantId,
};

pub mod read;

pub use read::AttributeReadContext;

pub const UNSET_ID_VALUE: i64 = -1;

/// Indicates which least specific field for an [`AttributeContext`] is specified.
#[derive(Debug)]
pub enum AttributeContextLeastSpecificFieldKind {
    ExternalProvider,
    InternalProvider,
    Prop,
}

#[derive(Error, Debug)]
pub enum AttributeContextError {
    #[error("attribute context builder error: {0}")]
    AttributeContextBuilder(#[from] AttributeContextBuilderError),
    #[error("could not find least specific field")]
    LeastSpecificFieldKindNotFound,
}

pub type AttributeContextResult<T> = Result<T, AttributeContextError>;

#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct AttributeContext {
    #[serde(rename = "attribute_context_prop_id")]
    prop_id: PropId,
    #[serde(rename = "attribute_context_internal_provider_id")]
    internal_provider_id: InternalProviderId,
    #[serde(rename = "attribute_context_external_provider_id")]
    external_provider_id: ExternalProviderId,
    #[serde(rename = "attribute_context_schema_id")]
    schema_id: SchemaId,
    #[serde(rename = "attribute_context_schema_variant_id")]
    schema_variant_id: SchemaVariantId,
    #[serde(rename = "attribute_context_component_id")]
    component_id: ComponentId,
}

impl From<AttributeContext> for AttributeContextBuilder {
    fn from(from_context: AttributeContext) -> AttributeContextBuilder {
        AttributeContextBuilder {
            prop_id: from_context.prop_id(),
            internal_provider_id: from_context.internal_provider_id(),
            external_provider_id: from_context.external_provider_id(),
            schema_id: from_context.schema_id(),
            schema_variant_id: from_context.schema_variant_id(),
            component_id: from_context.component_id(),
        }
    }
}

impl From<AttributeReadContext> for AttributeContextBuilder {
    fn from(from_read_context: AttributeReadContext) -> AttributeContextBuilder {
        let mut builder = AttributeContextBuilder::new();
        if let Some(prop_id) = from_read_context.prop_id {
            builder.set_prop_id(prop_id);
        }
        if let Some(internal_provider_id) = from_read_context.internal_provider_id {
            builder.set_internal_provider_id(internal_provider_id);
        }
        if let Some(external_provider_id) = from_read_context.external_provider_id {
            builder.set_external_provider_id(external_provider_id);
        }
        if let Some(schema_id) = from_read_context.schema_id {
            builder.set_schema_id(schema_id);
        }
        if let Some(schema_variant_id) = from_read_context.schema_variant_id {
            builder.set_schema_variant_id(schema_variant_id);
        }
        if let Some(component_id) = from_read_context.component_id {
            builder.set_component_id(component_id);
        }
        builder
    }
}

impl PartialOrd for AttributeContext {
    /// How to compare two [`AttributeContexts`](crate::AttributeContext):
    ///
    /// - [`Ordering::Equal`]: same level of specificity between two contexts
    /// - [`Ordering::Greater`]: "self" is "more-specific" than "other"
    /// - [`Ordering::Less`]: "self" is "less-specific" than "other"
    /// - [`None`]: "self" and "other" have different "least-specific" fields (e.g. "self" is
    ///   [`Prop`](crate::Prop)-specific and "other" is [`InternalProvider`](crate::InternalProvider)-specific.
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if !self.is_component_unset() {
            return if !other.is_component_unset() {
                Some(Ordering::Equal)
            } else {
                Some(Ordering::Greater)
            };
        }

        if !self.is_schema_variant_unset() {
            return if !other.is_component_unset() {
                Some(Ordering::Less)
            } else if !other.is_schema_variant_unset() {
                Some(Ordering::Equal)
            } else {
                Some(Ordering::Greater)
            };
        }

        if !self.is_schema_unset() {
            return if !other.is_schema_variant_unset() {
                Some(Ordering::Less)
            } else if !other.is_schema_unset() {
                Some(Ordering::Equal)
            } else {
                Some(Ordering::Greater)
            };
        }

        if !self.is_external_provider_unset() {
            return if !other.is_schema_unset() {
                Some(Ordering::Less)
            } else if !other.is_external_provider_unset() {
                Some(Ordering::Equal)
            } else {
                None
            };
        }

        if !self.is_internal_provider_unset() {
            return if !other.is_schema_unset() {
                Some(Ordering::Less)
            } else if !other.is_internal_provider_unset() {
                Some(Ordering::Equal)
            } else {
                None
            };
        }

        if !self.is_prop_unset() {
            return if !other.is_schema_unset() {
                Some(Ordering::Less)
            } else if !other.is_prop_unset() {
                Some(Ordering::Equal)
            } else {
                None
            };
        }

        None
    }
}

impl AttributeContext {
    pub fn builder() -> AttributeContextBuilder {
        AttributeContextBuilder::new()
    }

    pub fn prop_id(&self) -> PropId {
        self.prop_id
    }

    pub fn is_prop_unset(&self) -> bool {
        self.prop_id == UNSET_ID_VALUE.into()
    }

    pub fn internal_provider_id(&self) -> InternalProviderId {
        self.internal_provider_id
    }

    pub fn is_internal_provider_unset(&self) -> bool {
        self.internal_provider_id == UNSET_ID_VALUE.into()
    }

    pub fn external_provider_id(&self) -> ExternalProviderId {
        self.external_provider_id
    }

    pub fn is_external_provider_unset(&self) -> bool {
        self.external_provider_id == UNSET_ID_VALUE.into()
    }

    pub fn schema_id(&self) -> SchemaId {
        self.schema_id
    }

    pub fn is_schema_unset(&self) -> bool {
        self.schema_id == UNSET_ID_VALUE.into()
    }

    pub fn schema_variant_id(&self) -> SchemaVariantId {
        self.schema_variant_id
    }

    pub fn is_schema_variant_unset(&self) -> bool {
        self.schema_variant_id == UNSET_ID_VALUE.into()
    }

    pub fn component_id(&self) -> ComponentId {
        self.component_id
    }

    pub fn is_component_unset(&self) -> bool {
        self.component_id == UNSET_ID_VALUE.into()
    }

    pub fn is_least_specific(&self) -> bool {
        self.component_id == UNSET_ID_VALUE.into()
            && self.schema_variant_id == UNSET_ID_VALUE.into()
            && self.schema_id == UNSET_ID_VALUE.into()
    }

    /// Return a new [`AttributeContext`] with the most specific piece
    /// of the current [`AttributeContext`] unset, widening the scope
    /// of the context by one step. If widening the context would
    /// result in everything being unset, it will return a new copy of
    /// the current [`AttributeContext`].
    pub fn less_specific(&self) -> AttributeContextResult<Self> {
        let mut builder = AttributeContextBuilder::from(*self);
        if self.component_id() != UNSET_ID_VALUE.into() {
            builder.unset_component_id();
        } else if self.schema_variant_id() != UNSET_ID_VALUE.into() {
            builder.unset_schema_variant_id();
        } else if self.schema_id() != UNSET_ID_VALUE.into() {
            builder.unset_schema_id();
        }

        Ok(builder.to_context()?)
    }

    /// Returns true if the least specific field corresponds to a [`Prop`](crate::Prop).
    pub fn is_least_specific_field_kind_prop(&self) -> AttributeContextResult<bool> {
        if let AttributeContextLeastSpecificFieldKind::Prop = self.least_specific_field_kind()? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Returns true if the least specific field corresponds to an [`InternalProvider`](crate::InternalProvider).
    pub fn is_least_specific_field_kind_internal_provider(&self) -> AttributeContextResult<bool> {
        if let AttributeContextLeastSpecificFieldKind::InternalProvider =
            self.least_specific_field_kind()?
        {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Returns true if the least specific field corresponds to an [`InternalProvider`](crate::InternalProvider)
    /// _or_ an [`ExternalProvider`](crate::ExternalProvider).
    pub fn is_least_specific_field_kind_internal_or_external_provider(
        &self,
    ) -> AttributeContextResult<bool> {
        match self.least_specific_field_kind()? {
            AttributeContextLeastSpecificFieldKind::InternalProvider
            | AttributeContextLeastSpecificFieldKind::ExternalProvider => Ok(true),
            _ => Ok(false),
        }
    }

    /// Returns true if the least specific field corresponds to an [`ExternalProvider`](crate::ExternalProvider).
    pub fn is_least_specific_field_kind_external_provider(&self) -> AttributeContextResult<bool> {
        if let AttributeContextLeastSpecificFieldKind::ExternalProvider =
            self.least_specific_field_kind()?
        {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Returns the [`AttributeContextLeastSpecificFieldKind`] that is "set" for [`Self`].
    pub fn least_specific_field_kind(
        &self,
    ) -> AttributeContextResult<AttributeContextLeastSpecificFieldKind> {
        if self.prop_id != UNSET_ID_VALUE.into() {
            Ok(AttributeContextLeastSpecificFieldKind::Prop)
        } else if self.internal_provider_id != UNSET_ID_VALUE.into() {
            Ok(AttributeContextLeastSpecificFieldKind::InternalProvider)
        } else if self.external_provider_id != UNSET_ID_VALUE.into() {
            Ok(AttributeContextLeastSpecificFieldKind::ExternalProvider)
        } else {
            // This should never be possible to hit, but this check exists to protect
            // against potential regressions.
            Err(AttributeContextError::LeastSpecificFieldKindNotFound)
        }
    }
}

#[derive(Error, Debug)]
pub enum AttributeContextBuilderError {
    #[error("for builder {0:?}, the following fields must be set: {1:?}")]
    PrerequisteFieldsUnset(AttributeContextBuilder, Vec<&'static str>),
    #[error(
        "cannot specify more than one field at the lowest level in the order of precedence: {0:?}"
    )]
    MultipleLeastSpecificFieldsSpecified(AttributeContextBuilder),
}

pub type AttributeContextBuilderResult<T> = Result<T, AttributeContextBuilderError>;

/// A builder with non-consuming "setter" and "unsetter" methods that
/// verify the order of precedence for [`AttributeContext`].
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Copy)]
pub struct AttributeContextBuilder {
    prop_id: PropId,
    internal_provider_id: InternalProviderId,
    external_provider_id: ExternalProviderId,
    schema_id: SchemaId,
    schema_variant_id: SchemaVariantId,
    component_id: ComponentId,
}

/// Returns [`Self::new()`].
impl Default for AttributeContextBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl AttributeContextBuilder {
    /// Creates [`Self`] with all fields set to [`UNSET_ID_VALUE`].
    pub fn new() -> Self {
        Self {
            prop_id: UNSET_ID_VALUE.into(),
            internal_provider_id: UNSET_ID_VALUE.into(),
            external_provider_id: UNSET_ID_VALUE.into(),
            schema_id: UNSET_ID_VALUE.into(),
            schema_variant_id: UNSET_ID_VALUE.into(),
            component_id: UNSET_ID_VALUE.into(),
        }
    }

    /// Converts [`Self`] to [`AttributeContext`]. This method will
    /// fail if the order of precedence is broken (i.e. more-specific
    /// fields are set, but one-to-all less-specific fields are unset)
    /// or if the field of least specificity, [`PropId`], is unset.
    pub fn to_context(&self) -> AttributeContextBuilderResult<AttributeContext> {
        let mut unset_prerequisite_fields = Vec::new();

        // Start with the second highest specificty and work our way down.
        if self.schema_variant_id == UNSET_ID_VALUE.into()
            && self.component_id != UNSET_ID_VALUE.into()
        {
            unset_prerequisite_fields.push("SchemaVariantId");
        }
        if self.schema_id == UNSET_ID_VALUE.into()
            && (self.schema_variant_id != UNSET_ID_VALUE.into()
                || self.component_id != UNSET_ID_VALUE.into())
        {
            unset_prerequisite_fields.push("SchemaId");
        }

        // The lowest level in the order of precedence must always be set.
        if self.prop_id == UNSET_ID_VALUE.into()
            && self.internal_provider_id == UNSET_ID_VALUE.into()
            && self.external_provider_id == UNSET_ID_VALUE.into()
        {
            unset_prerequisite_fields.push("PropId or InternalProviderId or ExternalProviderId");
        }

        // Only one field at the lowest level in the order of precedence can be set.
        if (self.prop_id != UNSET_ID_VALUE.into()
            && self.internal_provider_id != UNSET_ID_VALUE.into())
            || (self.prop_id != UNSET_ID_VALUE.into()
                && self.external_provider_id != UNSET_ID_VALUE.into())
            || (self.internal_provider_id != UNSET_ID_VALUE.into()
                && self.external_provider_id != UNSET_ID_VALUE.into())
        {
            return Err(AttributeContextBuilderError::MultipleLeastSpecificFieldsSpecified(*self));
        }

        if !unset_prerequisite_fields.is_empty() {
            return Err(AttributeContextBuilderError::PrerequisteFieldsUnset(
                *self,
                unset_prerequisite_fields,
            ));
        }

        Ok(AttributeContext {
            prop_id: self.prop_id,
            internal_provider_id: self.internal_provider_id,
            external_provider_id: self.external_provider_id,
            schema_id: self.schema_id,
            schema_variant_id: self.schema_variant_id,
            component_id: self.component_id,
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

    /// Sets the [`InternalProviderId`] field. If [`UNSET_ID_VALUE`] is the ID passed in, then
    /// [`Self::unset_internal_provider_id()`] is returned.
    pub fn set_internal_provider_id(
        &mut self,
        internal_provider_id: InternalProviderId,
    ) -> &mut Self {
        if internal_provider_id == UNSET_ID_VALUE.into() {
            return self.unset_internal_provider_id();
        }
        self.internal_provider_id = internal_provider_id;
        self
    }

    /// Sets the [`ExternalProviderId`] field. If [`UNSET_ID_VALUE`] is the ID passed in, then
    /// [`Self::unset_external_provider_id()`] is returned.
    pub fn set_external_provider_id(
        &mut self,
        external_provider_id: ExternalProviderId,
    ) -> &mut Self {
        if external_provider_id == UNSET_ID_VALUE.into() {
            return self.unset_external_provider_id();
        }
        self.external_provider_id = external_provider_id;
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

    /// Unsets the [`PropId`].
    pub fn unset_prop_id(&mut self) -> &mut Self {
        self.prop_id = UNSET_ID_VALUE.into();
        self
    }

    /// Unsets the [`InternalProviderId`].
    pub fn unset_internal_provider_id(&mut self) -> &mut Self {
        self.internal_provider_id = UNSET_ID_VALUE.into();
        self
    }

    /// Unsets the [`ExternalProviderId`].
    pub fn unset_external_provider_id(&mut self) -> &mut Self {
        self.external_provider_id = UNSET_ID_VALUE.into();
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
}

impl postgres_types::ToSql for AttributeContext {
    fn to_sql(
        &self,
        ty: &postgres_types::Type,
        out: &mut postgres_types::private::BytesMut,
    ) -> Result<postgres_types::IsNull, Box<dyn std::error::Error + Sync + Send>>
    where
        Self: Sized,
    {
        let json = serde_json::to_value(self)?;
        postgres_types::ToSql::to_sql(&json, ty, out)
    }

    fn accepts(ty: &postgres_types::Type) -> bool
    where
        Self: Sized,
    {
        ty == &postgres_types::Type::JSONB
    }

    fn to_sql_checked(
        &self,
        ty: &postgres_types::Type,
        out: &mut postgres_types::private::BytesMut,
    ) -> Result<postgres_types::IsNull, Box<dyn std::error::Error + Sync + Send>> {
        let json = serde_json::to_value(self)?;
        postgres_types::ToSql::to_sql(&json, ty, out)
    }
}

// NOTE(nick): there are only error permutations tests for fields that have at least two prerequisite
// fields. Thus ComponentId, and SchemaVariantId have error permutations tests and SchemaId
// and PropId do not.

// TODO(nick): for the aforementioned error permutations tests, when/if more "layers" are added, we will likely
// need a helper to "flip" values from set to unset (and vice versa) to automatically test every condition.
// Currently, all error permutations are manually written. In an example using an automatic setup, the
// helper could provide an iteration method that flips each fields value from [`UNSET_ID_VALUE`] to
// "1.into()" and vice versa. Then, the test writer could supply contraints to indicate when the helper
// should expect failure or success upon iteration.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn less_specific() {
        let context = AttributeContextBuilder::new()
            .set_prop_id(1.into())
            .set_schema_id(2.into())
            .set_schema_variant_id(3.into())
            .set_component_id(4.into())
            .to_context()
            .expect("cannot build attribute context");

        let new_context = context
            .less_specific()
            .expect("cannot create less specific context");

        assert_eq!(
            AttributeContextBuilder::new()
                .set_prop_id(1.into())
                .set_schema_id(2.into())
                .set_schema_variant_id(3.into())
                .to_context()
                .expect("cannot create expected context"),
            new_context,
        );

        let new_context = new_context
            .less_specific()
            .expect("cannot create less specific context");

        assert_eq!(
            AttributeContextBuilder::new()
                .set_prop_id(1.into())
                .set_schema_id(2.into())
                .to_context()
                .expect("cannot create expected context"),
            new_context,
        );

        let new_context = new_context
            .less_specific()
            .expect("cannot create less specific context");

        assert_eq!(
            AttributeContextBuilder::new()
                .set_prop_id(1.into())
                .to_context()
                .expect("cannot create expected context"),
            new_context,
        );

        let new_context = new_context
            .less_specific()
            .expect("cannot create less specific context");

        assert_eq!(
            AttributeContextBuilder::new()
                .set_prop_id(1.into())
                .to_context()
                .expect("cannot create expected context"),
            new_context,
        );

        let least_specific_context = AttributeContextBuilder::new()
            .set_prop_id(1.into())
            .to_context()
            .expect("cannot create expected context");
        assert!(least_specific_context.is_least_specific());
    }

    #[test]
    fn builder_new() {
        let prop_id: PropId = 1.into();
        let schema_id: SchemaId = 1.into();
        let schema_variant_id: SchemaVariantId = 1.into();
        let component_id: ComponentId = 1.into();

        let mut builder = AttributeContextBuilder::new();

        // Empty (FAIL)
        assert!(builder.to_context().is_err());

        // SchemaId (FAIL) --> PropId (PASS)
        builder.set_schema_id(schema_id);
        assert!(builder.to_context().is_err());
        builder.unset_schema_id();
        builder.set_prop_id(prop_id);
        assert!(builder.to_context().is_ok(),);

        // SchemaVariantId (FAIL) --> SchemaId (PASS)
        builder.set_schema_variant_id(schema_variant_id);
        assert!(builder.to_context().is_err());
        builder.unset_schema_variant_id();
        builder.set_schema_id(schema_id);
        assert!(builder.to_context().is_ok(),);

        // ComponentId (FAIL) --> SchemaVariantId (PASS)
        builder.set_component_id(component_id);
        assert!(builder.to_context().is_err());
        builder.unset_component_id();
        builder.set_schema_variant_id(schema_variant_id);
        assert!(builder.to_context().is_ok(),);

        builder.set_component_id(component_id);
        assert!(builder.to_context().is_ok());
    }

    #[test]
    fn builder_component_id_error_permutations() {
        let prop_id: PropId = 1.into();
        let schema_id: SchemaId = 1.into();
        let schema_variant_id: SchemaVariantId = 1.into();
        let component_id: ComponentId = 1.into();

        // ----------------
        // Prerequisites: 0
        // ----------------

        // SchemaVariantId [ ] --> SchemaId [ ] --> PropId [ ]
        let mut builder = AttributeContextBuilder::new();
        builder.set_component_id(component_id);
        assert!(builder.to_context().is_err());

        // ----------------
        // Prerequisites: 1
        // ----------------

        // SchemaVariantId [x] --> SchemaId [ ] --> PropId [ ]
        let mut builder = AttributeContextBuilder::new();
        builder.set_component_id(component_id);
        builder.set_schema_variant_id(schema_variant_id);
        assert!(builder.to_context().is_err());

        // SchemaVariantId [ ] --> SchemaId [x] --> PropId [ ]
        let mut builder = AttributeContextBuilder::new();
        builder.set_component_id(component_id);
        builder.set_schema_id(schema_id);
        assert!(builder.to_context().is_err());

        // SchemaVariantId [ ] --> SchemaId [ ] --> PropId [x]
        let mut builder = AttributeContextBuilder::new();
        builder.set_component_id(component_id);
        builder.set_prop_id(prop_id);
        assert!(builder.to_context().is_err());

        // ----------------
        // Prerequisites: 2
        // ----------------

        // SchemaVariantId [x] --> SchemaId [x] --> PropId [ ]
        let mut builder = AttributeContextBuilder::new();
        builder.set_component_id(component_id);
        builder.set_schema_variant_id(schema_variant_id);
        builder.set_schema_id(schema_id);
        assert!(builder.to_context().is_err());

        // SchemaVariantId [x] --> SchemaId [ ] --> PropId [x]
        let mut builder = AttributeContextBuilder::new();
        builder.set_component_id(component_id);
        builder.set_schema_variant_id(schema_variant_id);
        builder.set_prop_id(prop_id);
        assert!(builder.to_context().is_err());

        // SchemaVariantId [ ] --> SchemaId [x] --> PropId [x]
        let mut builder = AttributeContextBuilder::new();
        builder.set_component_id(component_id);
        builder.set_schema_id(schema_id);
        builder.set_prop_id(prop_id);
        assert!(builder.to_context().is_err());
    }

    #[test]
    fn builder_schema_variant_id_error_permutations() {
        let prop_id: PropId = 1.into();
        let schema_id: SchemaId = 1.into();
        let schema_variant_id: SchemaVariantId = 1.into();

        // ----------------
        // Prerequisites: 0
        // ----------------

        // SchemaId [ ] --> PropId [ ]
        let mut builder = AttributeContextBuilder::new();
        builder.set_schema_variant_id(schema_variant_id);
        assert!(builder.to_context().is_err());

        // ----------------
        // Prerequisites: 1
        // ----------------

        // SchemaId [x] --> PropId [ ]
        let mut builder = AttributeContextBuilder::new();
        builder.set_schema_variant_id(schema_variant_id);
        builder.set_schema_id(schema_id);
        assert!(builder.to_context().is_err());

        // SchemaId [ ] --> PropId [x]
        let mut builder = AttributeContextBuilder::new();
        builder.set_schema_variant_id(schema_variant_id);
        builder.set_prop_id(prop_id);
        assert!(builder.to_context().is_err());
    }
}
