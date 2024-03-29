//! Create a [`SchemaVariant`](crate::SchemaVariant) with a [`Prop`](crate::Prop) tree via a
//! [`SchemaVariantDefinition`], stored in the database.
//! Do not use this struct to access or modify [`SchemaVariant`](crate::SchemaVariant) data.
//! After import, it only exists to map a [`SchemaVariant`](crate::SchemaVariant) to its
//! generating [`Func`](crate::Func). Data on this object will get outdated if the prop tree changes.

// use serde::{Deserialize, Serialize};
// use serde_json::Value;
// use si_pkg::{SchemaSpec, SchemaSpecData, SchemaVariantSpecData, SiPropFuncSpecKind};
// use std::collections::HashMap;
// use thiserror::Error;
//
// use crate::property_editor::schema::WidgetKind;
// use crate::schema::variant::root_prop::component_type::ComponentType;
// use crate::schema::variant::{SchemaVariantError, DEFAULT_SCHEMA_VARIANT_COLOR};
// use crate::{pk, PropKind, SchemaError, SchemaVariant, TransactionsError};
// use crate::{DalContext, Schema, SchemaVariantId, SocketArity, Timestamp};

// #[remain::sorted]
// #[derive(Error, Debug)]
// pub enum SchemaVariantDefinitionError {
//     //     #[error("Could not check for default variant: {0}")]
//     //     CouldNotCheckForDefaultVariant(String),
//     //     #[error("Could not get ui menu for schema: {0}")]
//     //     CouldNotGetUiMenu(SchemaId),
//     //     #[error("error decoding code_base64: {0}")]
//     //     Decode(#[from] base64::DecodeError),
//     #[error("default variant not found: {0}")]
//     DefaultVariantNotFound(String),
//     //     #[error("history event error: {0}")]
//     //     HistoryEvent(#[from] HistoryEventError),
//     //     #[error("{0} is not a valid hex color string")]
//     //     InvalidHexColor(String),
//     #[error("schema spec has more than one variant, which we do not yet support")]
//     MoreThanOneVariant,
//     //     #[error("nats txn error: {0}")]
//     //     Nats(#[from] NatsError),
//     #[error("schema spec has no variants")]
//     NoVariants,
//     //     #[error("pg error: {0}")]
//     //     Pg(#[from] PgError),
//     //     #[error("pkg error: {0}")]
//     //     Pkg(#[from] Box<PkgError>),
//     #[error(transparent)]
//     Schema(#[from] SchemaError),
//     #[error(transparent)]
//     SchemaVariant(#[from] SchemaVariantError),
//     //     #[error("error serializing/deserializing json: {0}")]
//     //     SerdeJson(#[from] serde_json::Error),
//     #[error("spec error: {0}")]
//     Spec(#[from] SpecError),
//     //     #[error("standard model error: {0}")]
//     //     StandardModelError(#[from] StandardModelError),
//     #[error("transactions error: {0}")]
//     Transactions(#[from] TransactionsError),
//     //     #[error("url parse error: {0}")]
//     //     Url(#[from] ParseError),
// }

// pub type SchemaVariantDefinitionResult<T> = Result<T, SchemaVariantDefinitionError>;
//
// /// A cache of [`PropIds`](crate::Prop) where the _key_ is a tuple corresponding to the
// /// [`Prop`](crate::Prop) name and the _parent_ [`PropId`](crate::Prop) who's child is the
// /// [`PropId`](crate::Prop) in the _value_ of the entry.
// ///
// /// It is recommended to start with the [`RootProp`](crate::RootProp) in order to descend into the
// /// cache.
// #[derive(Debug, Clone)]
// pub struct PropCache(HashMap<(String, PropId), PropId>);
//
// impl PropCache {
//     pub fn new() -> Self {
//         Self(HashMap::new())
//     }
//
//     /// Attempts to retrieve the [`PropId`](crate::Prop) value for a given [`Prop`](crate::Prop)
//     /// name and parent [`PropId`](crate::Prop) key tuple. An error is returned if nothing is found.
//     pub fn get(
//         &self,
//         prop_name: impl AsRef<str>,
//         parent_prop_id: PropId,
//     ) -> SchemaVariantResult<PropId> {
//         // NOTE(nick): the string handling could probably be better here.
//         let prop_name = prop_name.as_ref().to_string();
//         let prop_id = *self.0.get(&(prop_name.clone(), parent_prop_id)).ok_or(
//             SchemaVariantError::PropNotFoundInCache(prop_name, parent_prop_id),
//         )?;
//         Ok(prop_id)
//     }
//
//     /// Insert the [`PropId`](crate::Prop) into [`self`](Self). The returned `option` from the
//     /// underlying method is ignored.
//     pub fn insert(&mut self, key: (String, PropId), value: PropId) {
//         self.0.insert(key, value);
//     }
// }
//
// impl Default for PropCache {
//     fn default() -> Self {
//         Self::new()
//     }
// }
//
// pk!(SchemaVariantDefinitionPk);

// pk!(SchemaVariantDefinitionId);

// #[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
// pub struct SchemaVariantDefinition {
//     pk: SchemaVariantDefinitionPk,
//     id: SchemaVariantDefinitionId,
//     #[serde(flatten)]
//     tenancy: Tenancy,
//     #[serde(flatten)]
//     timestamp: Timestamp,
//     #[serde(flatten)]
//     visibility: Visibility,
//
//     /// Name for this variant. Actually, this is the name for this [`Schema`](crate::Schema), we're
//     /// punting on the issue of multiple variants for the moment.
//     name: String,
//     /// Override for the UI name for this schema
//     menu_name: Option<String>,
//     /// The category this schema variant belongs to
//     category: String,
//     /// The color for the component on the component diagram as a hex string
//     color: String,
//     component_kind: ComponentKind,
//     component_type: ComponentType,
//     link: Option<String>,
//     func_id: FuncId,
//     description: Option<String>,
//     schema_variant_id: Option<SchemaVariantId>,
// }

// impl_standard_model! {
//     model: SchemaVariantDefinition,
//     pk: SchemaVariantDefinitionPk,
//     id: SchemaVariantDefinitionId,
//     table_name: "schema_variant_definitions",
//     history_event_label_base: "schema_variant_definition",
//     history_event_message_name: "Schema Variant Definition",
// }

// impl SchemaVariantDefinition {
//     #[allow(clippy::too_many_arguments)]
//     pub async fn new(
//         ctx: &DalContext,
//         name: String,
//         menu_name: Option<String>,
//         category: String,
//         link: Option<String>,
//         color: String,
//         component_kind: ComponentKind,
//         description: Option<String>,
//         func_id: FuncId,
//     ) -> SchemaVariantDefinitionResult<SchemaVariantDefinition> {
//         let row = ctx
//             .txns()
//             .await?
//             .pg()
//             .query_one(
//                 "SELECT object FROM schema_variant_definition_create_v1(
//                     $1,
//                     $2,
//                     $3,
//                     $4,
//                     $5,
//                     $6,
//                     $7,
//                     $8,
//                     $9,
//                     $10
//                 )",
//                 &[
//                     ctx.tenancy(),
//                     ctx.visibility(),
//                     &name,
//                     &menu_name,
//                     &category,
//                     &link,
//                     &color,
//                     &component_kind.as_ref(),
//                     &func_id,
//                     &description,
//                 ],
//             )
//             .await?;

//         Ok(standard_model::finish_create_from_row(ctx, row).await?)
//     }

//     pub async fn get_by_func_id(
//         ctx: &DalContext,
//         func_id: FuncId,
//     ) -> SchemaVariantDefinitionResult<Option<Self>> {
//         let row = ctx
//             .txns()
//             .await?
//             .pg()
//             .query_opt(
//                 "SELECT row_to_json(svd.*) AS object
//                     FROM schema_variant_definitions_v1($1, $2) as svd
//                     WHERE func_id = $3",
//                 &[ctx.tenancy(), ctx.visibility(), &func_id],
//             )
//             .await?;

//         Ok(standard_model::object_option_from_row_option(row)?)
//     }

//     pub async fn get_by_schema_variant_id(
//         ctx: &DalContext,
//         schema_variant_id: &SchemaVariantId,
//     ) -> SchemaVariantDefinitionResult<Option<Self>> {
//         let row = ctx
//             .txns()
//             .await?
//             .pg()
//             .query_opt(
//                 "SELECT row_to_json(svd.*) AS object
//                     FROM schema_variant_definitions_v1($1, $2) as svd
//                     WHERE schema_variant_id = $3",
//                 &[ctx.tenancy(), ctx.visibility(), schema_variant_id],
//             )
//             .await?;

//         Ok(standard_model::object_option_from_row_option(row)?)
//     }

//     pub async fn list_components(
//         &self,
//         ctx: &DalContext,
//     ) -> SchemaVariantDefinitionResult<Vec<Component>> {
//         Ok(if let Some(variant_id) = self.schema_variant_id {
//             Component::list_for_schema_variant(ctx, variant_id)
//                 .await
//                 .map_err(Box::new)?
//         } else {
//             vec![]
//         })
//     }

//     standard_model_accessor!(name, String, SchemaVariantDefinitionResult);
//     standard_model_accessor!(menu_name, Option<String>, SchemaVariantDefinitionResult);
//     standard_model_accessor!(category, String, SchemaVariantDefinitionResult);
//     standard_model_accessor!(color, String, SchemaVariantDefinitionResult);
//     standard_model_accessor!(
//         component_kind,
//         Enum(ComponentKind),
//         SchemaVariantDefinitionResult
//     );
//     standard_model_accessor!(link, Option<String>, SchemaVariantDefinitionResult);
//     standard_model_accessor!(description, Option<String>, SchemaVariantDefinitionResult);
//     standard_model_accessor!(func_id, Pk(FuncId), SchemaVariantDefinitionResult);
//     standard_model_accessor!(
//         schema_variant_id,
//         Option<Pk(SchemaVariantId)>,
//         SchemaVariantDefinitionResult
//     );
//     standard_model_accessor!(
//         component_type,
//         Enum(ComponentType),
//         SchemaVariantDefinitionResult
//     );
// }
//

// impl From<SchemaVariantDefinition> for SchemaVariantDefinitionMetadataJson {
//     fn from(value: SchemaVariantDefinition) -> Self {
//         SchemaVariantDefinitionMetadataJson {
//             name: value.name,
//             menu_name: value.menu_name,
//             category: value.category,
//             color: value.color,
//             component_kind: value.component_kind,
//             component_type: value.component_type,
//             link: value.link,
//             description: value.description,
//         }
//     }
// }

// impl SchemaVariantDefinitionMetadataJson {
//     #[instrument(skip_all)]
//     #[allow(clippy::too_many_arguments)]
//     pub fn new(
//         name: &str,
//         menu_name: Option<&str>,
//         category: &str,
//         color: &str,
//         component_kind: ComponentKind,
//         link: Option<&str>,
//         description: Option<&str>,
//         component_type: ComponentType,
//     ) -> SchemaVariantDefinitionMetadataJson {
//         SchemaVariantDefinitionMetadataJson {
//             name: name.to_string(),
//             menu_name: menu_name.map(|s| s.to_string()),
//             category: category.to_string(),
//             color: color.to_string(),
//             component_kind,
//             component_type,
//             link: link.map(|l| l.to_string()),
//             description: description.map(|d| d.to_string()),
//         }
//     }

//     pub async fn from_schema_and_variant(
//         ctx: &DalContext,
//         schema: &Schema,
//         variant: &SchemaVariant,
//     ) -> SchemaVariantDefinitionResult<Self> {
//         let (menu_name, category) = match schema.ui_menus(ctx).await {
//             Ok(ui_menus) => match ui_menus.get(0) {
//                 Some(ui_menu) => (
//                     Some(ui_menu.name().to_string()),
//                     ui_menu.category().to_string(),
//                 ),
//                 None => (None, "".to_string()),
//             },
//             Err(_) => {
//                 return Err(SchemaVariantDefinitionError::CouldNotGetUiMenu(
//                     *schema.id(),
//                 ));
//             }
//         };

//         Ok(SchemaVariantDefinitionMetadataJson {
//             name: schema.name().to_string(),
//             menu_name,
//             category,
//             color: variant
//                 .color(ctx)
//                 .await
//                 .map_err(Box::new)?
//                 .unwrap_or_else(|| "baddad".to_string()),
//             component_kind: *schema.component_kind(),
//             link: variant.link().map(|l| l.to_string()),
//             description: None,
//             component_type: get_component_type(ctx, variant)
//                 .await
//                 .map_err(Box::new)?
//                 .into(),
//         })
//     }
// }
//

//
