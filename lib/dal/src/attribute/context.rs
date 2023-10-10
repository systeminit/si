//! This module contains the [`AttributeContext`], and its corresponding builder, [`AttributeContextBuilder`].
//! The context can be scoped with varying levels of specificity, using an order of precedence.
//! The builder ensures the correct order of precedence is maintained whilst setting and unsetting
//! fields of specificity.
//!
//! ## The Order of Precedence
//!
//! The order of precedence is as follows (from least to most "specificity"):
//! - [`PropId`] / [`InternalProviderId`] / [`ExternalProviderId`]
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

use crate::{ComponentId, ExternalProviderId, InternalProviderId, PropId};

pub mod read;

use crate::attribute::context::AttributeContextLeastSpecificFieldKind::{
    ExternalProvider, InternalProvider,
};
pub use read::AttributeReadContext;

/// Indicates which least specific field for an [`AttributeContext`] is specified and contains the
/// field's value.
#[remain::sorted]
#[derive(Debug)]
pub enum AttributeContextLeastSpecificFieldKind {
    ExternalProvider(ExternalProviderId),
    InternalProvider(InternalProviderId),
    Prop(PropId),
}

#[remain::sorted]
#[derive(Error, Debug)]
pub enum AttributeContextError {
    #[error("attribute context builder error: {0}")]
    AttributeContextBuilder(#[from] AttributeContextBuilderError),
    #[error("could not find least specific field")]
    LeastSpecificFieldKindNotFound,
}

pub type AttributeContextResult<T> = Result<T, AttributeContextError>;

#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AttributeContext {
    #[serde(rename = "attribute_context_prop_id")]
    prop_id: PropId,
    #[serde(rename = "attribute_context_internal_provider_id")]
    internal_provider_id: InternalProviderId,
    #[serde(rename = "attribute_context_external_provider_id")]
    external_provider_id: ExternalProviderId,
    #[serde(rename = "attribute_context_component_id")]
    component_id: ComponentId,
}

impl From<AttributeContext> for AttributeContextBuilder {
    fn from(from_context: AttributeContext) -> AttributeContextBuilder {
        AttributeContextBuilder {
            prop_id: from_context.prop_id(),
            internal_provider_id: from_context.internal_provider_id(),
            external_provider_id: from_context.external_provider_id(),
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

        if !self.is_external_provider_unset() {
            return if !other.is_component_unset() {
                Some(Ordering::Less)
            } else if !other.is_external_provider_unset() {
                Some(Ordering::Equal)
            } else {
                None
            };
        }

        if !self.is_internal_provider_unset() {
            return if !other.is_component_unset() {
                Some(Ordering::Less)
            } else if !other.is_internal_provider_unset() {
                Some(Ordering::Equal)
            } else {
                None
            };
        }

        if !self.is_prop_unset() {
            return if !other.is_component_unset() {
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
        self.prop_id == PropId::NONE
    }

    pub fn internal_provider_id(&self) -> InternalProviderId {
        self.internal_provider_id
    }

    pub fn is_internal_provider_unset(&self) -> bool {
        self.internal_provider_id == InternalProviderId::NONE
    }

    pub fn external_provider_id(&self) -> ExternalProviderId {
        self.external_provider_id
    }

    pub fn is_external_provider_unset(&self) -> bool {
        self.external_provider_id == ExternalProviderId::NONE
    }

    pub fn component_id(&self) -> ComponentId {
        self.component_id
    }

    pub fn is_component_unset(&self) -> bool {
        self.component_id == ComponentId::NONE
    }

    pub fn is_least_specific(&self) -> bool {
        self.component_id == ComponentId::NONE
    }

    /// Return a new [`AttributeContext`] with the most specific piece
    /// of the current [`AttributeContext`] unset, widening the scope
    /// of the context by one step. If widening the context would
    /// result in everything being unset, it will return a new copy of
    /// the current [`AttributeContext`].
    pub fn less_specific(&self) -> AttributeContextResult<Self> {
        let mut builder = AttributeContextBuilder::from(*self);
        if self.component_id() != ComponentId::NONE {
            builder.unset_component_id();
        }
        Ok(builder.to_context()?)
    }

    /// Returns true if the least specific field corresponds to a [`Prop`](crate::Prop).
    pub fn is_least_specific_field_kind_prop(&self) -> AttributeContextResult<bool> {
        if let AttributeContextLeastSpecificFieldKind::Prop(_) = self.least_specific_field_kind()? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Returns true if the least specific field corresponds to an [`InternalProvider`](crate::InternalProvider).
    pub fn is_least_specific_field_kind_internal_provider(&self) -> AttributeContextResult<bool> {
        if let InternalProvider(_) = self.least_specific_field_kind()? {
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
            InternalProvider(_) | ExternalProvider(_) => Ok(true),
            _ => Ok(false),
        }
    }

    /// Returns true if the least specific field corresponds to an [`ExternalProvider`](crate::ExternalProvider).
    pub fn is_least_specific_field_kind_external_provider(&self) -> AttributeContextResult<bool> {
        if let ExternalProvider(_) = self.least_specific_field_kind()? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Returns the [`AttributeContextLeastSpecificFieldKind`] that is "set" for [`Self`].
    pub fn least_specific_field_kind(
        &self,
    ) -> AttributeContextResult<AttributeContextLeastSpecificFieldKind> {
        if self.prop_id != PropId::NONE {
            Ok(AttributeContextLeastSpecificFieldKind::Prop(self.prop_id))
        } else if self.internal_provider_id != InternalProviderId::NONE {
            Ok(InternalProvider(self.internal_provider_id))
        } else if self.external_provider_id != ExternalProviderId::NONE {
            Ok(ExternalProvider(self.external_provider_id))
        } else {
            // This should never be possible to hit, but this check exists to protect
            // against potential regressions.
            Err(AttributeContextError::LeastSpecificFieldKindNotFound)
        }
    }
}

#[remain::sorted]
#[derive(Error, Debug)]
pub enum AttributeContextBuilderError {
    #[error(
        "cannot specify more than one field at the lowest level in the order of precedence: {0:?}"
    )]
    MultipleLeastSpecificFieldsSpecified(AttributeContextBuilder),
    #[error("for builder {0:?}, the following fields must be set: {1:?}")]
    PrerequisteFieldsUnset(AttributeContextBuilder, Vec<&'static str>),
}

pub type AttributeContextBuilderResult<T> = Result<T, AttributeContextBuilderError>;

/// A builder with non-consuming "setter" and "unsetter" methods that
/// verify the order of precedence for [`AttributeContext`].
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Copy)]
pub struct AttributeContextBuilder {
    prop_id: PropId,
    internal_provider_id: InternalProviderId,
    external_provider_id: ExternalProviderId,
    component_id: ComponentId,
}

/// Returns [`Self::new()`].
impl Default for AttributeContextBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl AttributeContextBuilder {
    /// Creates [`Self`] with all fields unset.
    pub fn new() -> Self {
        Self {
            prop_id: PropId::NONE,
            internal_provider_id: InternalProviderId::NONE,
            external_provider_id: ExternalProviderId::NONE,
            component_id: ComponentId::NONE,
        }
    }

    pub fn to_context_unchecked(&self) -> AttributeContext {
        AttributeContext {
            prop_id: self.prop_id,
            internal_provider_id: self.internal_provider_id,
            external_provider_id: self.external_provider_id,
            component_id: self.component_id,
        }
    }

    /// Converts [`Self`] to [`AttributeContext`]. This method will
    /// fail if the order of precedence is broken (i.e. more-specific
    /// fields are set, but one-to-all less-specific fields are unset)
    /// or if the field of least specificity, [`PropId`], is unset.
    pub fn to_context(&self) -> AttributeContextBuilderResult<AttributeContext> {
        let mut unset_prerequisite_fields = Vec::new();

        // The lowest level in the order of precedence must always be set.
        if self.prop_id == PropId::NONE
            && self.internal_provider_id == InternalProviderId::NONE
            && self.external_provider_id == ExternalProviderId::NONE
        {
            unset_prerequisite_fields.push("PropId or InternalProviderId or ExternalProviderId");
        }

        // Only one field at the lowest level in the order of precedence can be set.
        #[allow(clippy::nonminimal_bool)]
        if (self.prop_id != PropId::NONE && self.internal_provider_id != InternalProviderId::NONE)
            || (self.prop_id != PropId::NONE
                && self.external_provider_id != ExternalProviderId::NONE)
            || (self.internal_provider_id != InternalProviderId::NONE
                && self.external_provider_id != ExternalProviderId::NONE)
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
            component_id: self.component_id,
        })
    }

    /// Sets the [`PropId`] field. If the unset value is passed in, then
    /// [`Self::unset_prop_id()`] is returned.
    pub fn set_prop_id(&mut self, prop_id: PropId) -> &mut Self {
        if prop_id == PropId::NONE {
            return self.unset_prop_id();
        }
        self.prop_id = prop_id;
        self
    }

    /// Sets the [`InternalProviderId`] field. If the unset value is passed in, then
    /// [`Self::unset_internal_provider_id()`] is returned.
    pub fn set_internal_provider_id(
        &mut self,
        internal_provider_id: InternalProviderId,
    ) -> &mut Self {
        if internal_provider_id == InternalProviderId::NONE {
            return self.unset_internal_provider_id();
        }
        self.internal_provider_id = internal_provider_id;
        self
    }

    /// Sets the [`ExternalProviderId`] field. If the unset value is passed in, then
    /// [`Self::unset_external_provider_id()`] is returned.
    pub fn set_external_provider_id(
        &mut self,
        external_provider_id: ExternalProviderId,
    ) -> &mut Self {
        if external_provider_id == ExternalProviderId::NONE {
            return self.unset_external_provider_id();
        }
        self.external_provider_id = external_provider_id;
        self
    }

    /// Sets the [`ComponentId`] field. If the unset value is passed in, then
    /// [`Self::unset_component_id()`] is returned.
    pub fn set_component_id(&mut self, component_id: ComponentId) -> &mut Self {
        if component_id == ComponentId::NONE {
            return self.unset_component_id();
        }
        self.component_id = component_id;
        self
    }

    /// Unsets the [`PropId`].
    pub fn unset_prop_id(&mut self) -> &mut Self {
        self.prop_id = PropId::NONE;
        self
    }

    /// Unsets the [`InternalProviderId`].
    pub fn unset_internal_provider_id(&mut self) -> &mut Self {
        self.internal_provider_id = InternalProviderId::NONE;
        self
    }

    /// Unsets the [`ExternalProviderId`].
    pub fn unset_external_provider_id(&mut self) -> &mut Self {
        self.external_provider_id = ExternalProviderId::NONE;
        self
    }

    /// Unsets the [`ComponentId`].
    pub fn unset_component_id(&mut self) -> &mut Self {
        self.component_id = ComponentId::NONE;
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
// helper could provide an iteration method that flips each fields value from unset to
// "Id::generate()" and vice versa. Then, the test writer could supply contraints to indicate when the helper
// should expect failure or success upon iteration.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn less_specific() {
        let prop_id = PropId::generate();
        let component_id = ComponentId::generate();
        let context = AttributeContextBuilder::new()
            .set_prop_id(prop_id)
            .set_component_id(component_id)
            .to_context()
            .expect("cannot build attribute context");
        assert!(!context.is_least_specific());

        let new_context = context
            .less_specific()
            .expect("cannot create less specific context");

        assert_eq!(
            AttributeContextBuilder::new()
                .set_prop_id(prop_id)
                .to_context()
                .expect("cannot create expected context"),
            new_context,
        );

        let new_context = new_context
            .less_specific()
            .expect("cannot create less specific context");

        // Should be the exact same.
        assert_eq!(
            AttributeContextBuilder::new()
                .set_prop_id(prop_id)
                .to_context()
                .expect("cannot create expected context"),
            new_context,
        );
        assert!(new_context.is_least_specific());
    }

    #[test]
    fn builder_new() {
        let prop_id = PropId::generate();
        let component_id = ComponentId::generate();

        let mut builder = AttributeContextBuilder::new();

        // Empty (FAIL)
        assert!(builder.to_context().is_err());

        // ComponentId without PropId (FAIL)
        builder.set_component_id(component_id);
        assert!(builder.to_context().is_err());
        builder.unset_component_id();

        // PropId (PASS)
        builder.set_prop_id(prop_id);
        assert!(builder.to_context().is_ok());

        // ComponentId with PropId (PASS)
        builder.set_component_id(component_id);
        assert!(builder.to_context().is_ok());
    }
}
