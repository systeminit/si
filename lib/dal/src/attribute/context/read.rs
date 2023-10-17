// use serde::{Deserialize, Serialize};

// use crate::{AttributeContext, ComponentId, ExternalProviderId, InternalProviderId, PropId};

// /// An `AttributeReadContext` allows for saying "do not use this field
// /// to filter results" by providing [`None`] for the field's value.
// /// It also allows for saying "explicitly filter out results for that
// /// have this field set" by providing the unset value for the field's
// /// value.
// ///
// /// For example:
// ///
// /// ```rust
// /// # use dal::attribute::context::read::AttributeReadContext;
// /// # use dal::{ExternalProviderId, InternalProviderId, ComponentId};
// /// let read_context = AttributeReadContext {
// ///     prop_id: None,
// ///     internal_provider_id: Some(InternalProviderId::NONE),
// ///     external_provider_id: Some(ExternalProviderId::NONE),
// ///     component_id: Some(ComponentId::generate())
// /// };
// /// ```
// ///
// /// The above `AttributeReadContext` would be used for finding all
// /// attributes, across all [`Props`](crate::Prop) that have been set
// /// for a given [`ComponentId`].
// #[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq, Eq)]
// pub struct AttributeReadContext {
//     #[serde(rename = "attribute_context_prop_id")]
//     pub prop_id: Option<PropId>,
//     #[serde(rename = "attribute_context_internal_provider_id")]
//     pub internal_provider_id: Option<InternalProviderId>,
//     #[serde(rename = "attribute_context_external_provider_id")]
//     pub external_provider_id: Option<ExternalProviderId>,
//     #[serde(rename = "attribute_context_component_id")]
//     pub component_id: Option<ComponentId>,
// }

// impl Default for AttributeReadContext {
//     fn default() -> Self {
//         Self {
//             prop_id: Some(PropId::NONE),
//             internal_provider_id: Some(InternalProviderId::NONE),
//             external_provider_id: Some(ExternalProviderId::NONE),
//             component_id: Some(ComponentId::NONE),
//         }
//     }
// }

// impl From<AttributeContext> for AttributeReadContext {
//     fn from(from_context: AttributeContext) -> Self {
//         Self {
//             prop_id: Some(from_context.prop_id()),
//             internal_provider_id: Some(from_context.internal_provider_id()),
//             external_provider_id: Some(from_context.external_provider_id()),
//             component_id: Some(from_context.component_id()),
//         }
//     }
// }

// impl AttributeReadContext {
//     /// Creates a [`read context`](Self) with a given [`PropId`](crate::Prop)
//     /// and all other fields set to their defaults.
//     pub fn default_with_prop(prop_id: PropId) -> Self {
//         Self {
//             prop_id: Some(prop_id),
//             ..Self::default()
//         }
//     }

//     pub fn default_with_prop_and_component_id(
//         prop_id: PropId,
//         component_id: Option<ComponentId>,
//     ) -> Self {
//         Self {
//             prop_id: Some(prop_id),
//             component_id: match component_id {
//                 Some(component_id) => Some(component_id),
//                 None => Some(ComponentId::NONE),
//             },
//             ..Self::default()
//         }
//     }

//     /// Creates a [`read context`](Self) with a given [`InternalProviderId`](crate::InternalProvider)
//     /// and all other fields set to their defaults.
//     pub fn default_with_internal_provider(internal_provider_id: InternalProviderId) -> Self {
//         Self {
//             internal_provider_id: Some(internal_provider_id),
//             ..Self::default()
//         }
//     }

//     /// Creates a [`read context`](Self) with a given [`ExternalProviderId`](crate::ExternalProvider)
//     /// and all other fields set to their defaults.
//     pub fn default_with_external_provider(external_provider_id: ExternalProviderId) -> Self {
//         Self {
//             external_provider_id: Some(external_provider_id),
//             ..Self::default()
//         }
//     }

//     pub fn prop_id(&self) -> Option<PropId> {
//         self.prop_id
//     }

//     pub fn has_prop_id(&self) -> bool {
//         self.prop_id.is_some()
//     }

//     pub fn has_set_prop_id(&self) -> bool {
//         if let Some(prop_id) = self.prop_id {
//             prop_id != PropId::NONE
//         } else {
//             false
//         }
//     }

//     pub fn has_unset_prop_id(&self) -> bool {
//         if let Some(prop_id) = self.prop_id {
//             prop_id == PropId::NONE
//         } else {
//             false
//         }
//     }

//     pub fn internal_provider_id(&self) -> Option<InternalProviderId> {
//         self.internal_provider_id
//     }

//     pub fn has_internal_provider_id(&self) -> bool {
//         self.internal_provider_id.is_some()
//     }

//     pub fn has_set_internal_provider(&self) -> bool {
//         if let Some(internal_provider) = self.internal_provider_id {
//             internal_provider != InternalProviderId::NONE
//         } else {
//             false
//         }
//     }

//     pub fn has_unset_internal_provider(&self) -> bool {
//         if let Some(internal_provider) = self.internal_provider_id {
//             internal_provider == InternalProviderId::NONE
//         } else {
//             false
//         }
//     }

//     pub fn external_provider_id(&self) -> Option<ExternalProviderId> {
//         self.external_provider_id
//     }

//     pub fn has_external_provider_id(&self) -> bool {
//         self.external_provider_id.is_some()
//     }

//     pub fn has_set_external_provider(&self) -> bool {
//         if let Some(external_provider) = self.external_provider_id {
//             external_provider != ExternalProviderId::NONE
//         } else {
//             false
//         }
//     }

//     pub fn has_unset_external_provider(&self) -> bool {
//         if let Some(external_provider) = self.external_provider_id {
//             external_provider == ExternalProviderId::NONE
//         } else {
//             false
//         }
//     }

//     pub fn component_id(&self) -> Option<ComponentId> {
//         self.component_id
//     }

//     pub fn has_component_id(&self) -> bool {
//         self.component_id.is_some()
//     }

//     pub fn has_set_component_id(&self) -> bool {
//         if let Some(component_id) = self.component_id {
//             component_id != ComponentId::NONE
//         } else {
//             false
//         }
//     }

//     pub fn has_unset_component_id(&self) -> bool {
//         if let Some(component_id) = self.component_id {
//             component_id == ComponentId::NONE
//         } else {
//             false
//         }
//     }

//     pub fn any() -> Self {
//         Self {
//             prop_id: None,
//             internal_provider_id: None,
//             external_provider_id: None,
//             component_id: None,
//         }
//     }
// }

// impl postgres_types::ToSql for AttributeReadContext {
//     fn to_sql(
//         &self,
//         ty: &postgres_types::Type,
//         out: &mut postgres_types::private::BytesMut,
//     ) -> Result<postgres_types::IsNull, Box<dyn std::error::Error + Sync + Send>>
//     where
//         Self: Sized,
//     {
//         let json = serde_json::to_value(self)?;
//         postgres_types::ToSql::to_sql(&json, ty, out)
//     }

//     fn accepts(ty: &postgres_types::Type) -> bool
//     where
//         Self: Sized,
//     {
//         ty == &postgres_types::Type::JSONB
//     }

//     fn to_sql_checked(
//         &self,
//         ty: &postgres_types::Type,
//         out: &mut postgres_types::private::BytesMut,
//     ) -> Result<postgres_types::IsNull, Box<dyn std::error::Error + Sync + Send>> {
//         let json = serde_json::to_value(self)?;
//         postgres_types::ToSql::to_sql(&json, ty, out)
//     }
// }
