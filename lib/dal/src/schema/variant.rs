//! This module contains [`SchemaVariant`](crate::SchemaVariant), which is t/he "class" of a
//! [`Component`](crate::Component).

use content_store::ContentHash;
use serde::{Deserialize, Serialize};

use strum::EnumDiscriminants;
use telemetry::prelude::*;
use thiserror::Error;

use crate::workspace_snapshot::content_address::ContentAddress;
use crate::workspace_snapshot::node_weight::category_node_weight::CategoryNodeKind;
use crate::{pk, DalContext, PropId, StandardModel, Timestamp, WorkspaceSnapshot};

// use self::leaves::{LeafInput, LeafInputLocation, LeafKind};

// pub mod definition;
pub mod leaves;
pub mod root_prop;

// const ALL_FUNCS: &str = include_str!("../queries/schema_variant/all_related_funcs.sql");
// const ALL_PROPS: &str = include_str!("../queries/schema_variant/all_props.sql");
// const FIND_ROOT_PROP: &str = include_str!("../queries/schema_variant/find_root_prop.sql");
// const FIND_LEAF_ITEM_PROP: &str = include_str!("../queries/schema_variant/find_leaf_item_prop.sql");
// const FIND_ROOT_CHILD_IMPLICIT_INTERNAL_PROVIDER: &str =
//     include_str!("../queries/schema_variant/find_root_child_implicit_internal_provider.sql");
// const LIST_ROOT_SI_CHILD_PROPS: &str =
//     include_str!("../queries/schema_variant/list_root_si_child_props.sql");

pub const SCHEMA_VARIANT_VERSION: SchemaVariantContentDiscriminants =
    SchemaVariantContentDiscriminants::V1;

pk!(SchemaVariantId);

#[derive(Debug, PartialEq)]
pub struct SchemaVariantGraphNode {
    id: SchemaVariantId,
    content_address: ContentAddress,
    content: SchemaVariantContentV1,
}

#[derive(EnumDiscriminants, Serialize, Deserialize, PartialEq)]
#[serde(tag = "version")]
pub enum SchemaVariantContent {
    V1(SchemaVariantContentV1),
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct SchemaVariantContentV1 {
    #[serde(flatten)]
    pub timestamp: Timestamp,
    pub ui_hidden: bool,
    pub name: String,
    /// The [`RootProp`](crate::RootProp) for [`self`](Self).
    pub root_prop_id: Option<PropId>,
    // pub schema_variant_definition_id: Option<SchemaVariantDefinitionId>,
    pub link: Option<String>,
    pub finalized_once: bool,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct SchemaVariant {
    id: SchemaVariantId,
    #[serde(flatten)]
    timestamp: Timestamp,
    ui_hidden: bool,
    name: String,
    /// The [`RootProp`](crate::RootProp) for [`self`](Self).
    root_prop_id: Option<PropId>,
    // schema_variant_definition_id: Option<SchemaVariantDefinitionId>,
    link: Option<String>,
    finalized_once: bool,
}

impl SchemaVariantGraphNode {
    pub fn assemble(
        id: impl Into<SchemaVariantId>,
        content_hash: ContentHash,
        content: SchemaVariantContentV1,
    ) -> Self {
        Self {
            id: id.into(),
            content_address: ContentAddress::Schema(content_hash),
            content,
        }
    }
}

// impl SchemaVariant {
//     pub async fn is_builtin(&self, ctx: &DalContext) -> SchemaVariantResult<bool> {
//         let row = ctx
//             .txns()
//             .await?
//             .pg()
//             .query_opt(
//                 "SELECT id FROM schema_variants WHERE id = $1 and tenancy_workspace_pk = $2 LIMIT 1",
//                 &[self.id(), &WorkspacePk::NONE],
//             )
//             .await?;

//         Ok(row.is_some())
//     }

//     /// This _idempotent_ function "finalizes" a [`SchemaVariant`].
//     ///
//     /// Once a [`SchemaVariant`] has had all of its [`Props`](crate::Prop) created, there are a few
//     /// things that need to happen before it is usable:
//     ///
//     /// * Create the default [`AttributePrototypes`](crate::AttributePrototype) and
//     ///   [`AttributeValues`](crate::AttributeValue).
//     /// * Create the _internally consuming_ [`InternalProviders`](crate::InternalProvider)
//     ///   corresponding to every [`Prop`](crate::Prop) in the [`SchemaVariant`] that is not a
//     ///   descendant of an Array or a Map.
//     ///
//     /// This method **MUST** be called once all the [`Props`](Prop) have been created for the
//     /// [`SchemaVariant`]. It can be called multiple times while [`Props`](Prop) are being created,
//     /// but it must be called once after all [`Props`](Prop) have been created.
//     pub async fn finalize(
//         &mut self,
//         ctx: &DalContext,
//         component_type: Option<ComponentType>,
//     ) -> SchemaVariantResult<()> {
//         let total_start = std::time::Instant::now();

//         Self::create_default_prototypes_and_values(ctx, self.id).await?;
//         Self::create_implicit_internal_providers(ctx, self.id).await?;
//         if !self.finalized_once() {
//             self.set_finalized_once(ctx, true).await?;
//         }

//         // Default to the standard "component" component type.
//         let component_type = match component_type {
//             Some(component_type) => component_type,
//             None => ComponentType::Component,
//         };

//         // Find props that we need to set defaults on for _all_ schema variants.
//         // FIXME(nick): use the enum and create an appropriate query.
//         let mut maybe_type_prop_id = None;
//         let mut maybe_protected_prop_id = None;
//         for root_si_child_prop in Self::list_root_si_child_props(ctx, self.id).await? {
//             if root_si_child_prop.name() == "type" {
//                 maybe_type_prop_id = Some(*root_si_child_prop.id())
//             } else if root_si_child_prop.name() == "protected" {
//                 maybe_protected_prop_id = Some(*root_si_child_prop.id())
//             }
//         }
//         let type_prop_id =
//             maybe_type_prop_id.ok_or(SchemaVariantError::PropNotFound("/root/si/type"))?;
//         let protected_prop_id = maybe_protected_prop_id
//             .ok_or(SchemaVariantError::PropNotFound("/root/si/protected"))?;

//         // Set the default type of the schema variant.
//         let attribute_read_context = AttributeReadContext::default_with_prop(type_prop_id);
//         let attribute_value = AttributeValue::find_for_context(ctx, attribute_read_context)
//             .await?
//             .ok_or(SchemaVariantError::AttributeValueNotFoundForContext(
//                 attribute_read_context,
//             ))?;
//         let parent_attribute_value = attribute_value
//             .parent_attribute_value(ctx)
//             .await?
//             .ok_or_else(|| {
//                 SchemaVariantError::AttributeValueDoesNotHaveParent(*attribute_value.id())
//             })?;
//         let context = AttributeContextBuilder::from(attribute_read_context).to_context()?;
//         AttributeValue::update_for_context(
//             ctx,
//             *attribute_value.id(),
//             Some(*parent_attribute_value.id()),
//             context,
//             Some(serde_json::to_value(component_type)?),
//             None,
//         )
//         .await?;

//         // Ensure _all_ schema variants are not protected by default.
//         let attribute_read_context = AttributeReadContext::default_with_prop(protected_prop_id);
//         let attribute_value = AttributeValue::find_for_context(ctx, attribute_read_context)
//             .await?
//             .ok_or(SchemaVariantError::AttributeValueNotFoundForContext(
//                 attribute_read_context,
//             ))?;
//         let parent_attribute_value = attribute_value
//             .parent_attribute_value(ctx)
//             .await?
//             .ok_or_else(|| {
//                 SchemaVariantError::AttributeValueDoesNotHaveParent(*attribute_value.id())
//             })?;
//         let context = AttributeContextBuilder::from(attribute_read_context).to_context()?;
//         AttributeValue::update_for_context(
//             ctx,
//             *attribute_value.id(),
//             Some(*parent_attribute_value.id()),
//             context,
//             Some(serde_json::json![false]),
//             None,
//         )
//         .await?;

//         debug!("finalizing {:?} took {:?}", self.id, total_start.elapsed());
//         Ok(())
//     }

//     standard_model_accessor!(ui_hidden, bool, SchemaVariantResult);
//     standard_model_accessor!(name, String, SchemaVariantResult);
//     standard_model_accessor!(root_prop_id, Option<Pk(PropId)>, SchemaVariantResult);
//     standard_model_accessor!(link, Option<String>, SchemaVariantResult);
//     standard_model_accessor!(finalized_once, bool, SchemaVariantResult);
//     standard_model_accessor!(
//         schema_variant_definition_id,
//         Option<Pk(SchemaVariantDefinitionId)>,
//         SchemaVariantResult
//     );

//     pub async fn color(&self, ctx: &DalContext) -> SchemaVariantResult<Option<String>> {
//         let attribute_value = Component::find_si_child_attribute_value(
//             ctx,
//             ComponentId::NONE,
//             self.id,
//             SiPropChild::Color,
//         )
//         .await
//         .map_err(Box::new)?;
//         let func_binding_return_value =
//             FuncBindingReturnValue::get_by_id(ctx, &attribute_value.func_binding_return_value_id())
//                 .await?
//                 .ok_or_else(|| {
//                     SchemaVariantError::FuncBindingReturnValueNotFound(
//                         attribute_value.func_binding_return_value_id(),
//                     )
//                 })?;

//         let color = func_binding_return_value
//             .value()
//             .cloned()
//             .map(serde_json::from_value)
//             .transpose()?;
//         Ok(color)
//     }

//     pub async fn set_color(&self, ctx: &DalContext, color: String) -> SchemaVariantResult<()> {
//         let attribute_value = Component::find_si_child_attribute_value(
//             ctx,
//             ComponentId::NONE,
//             self.id,
//             SiPropChild::Color,
//         )
//         .await
//         .map_err(Box::new)?;
//         let prop = Prop::get_by_id(ctx, &attribute_value.context.prop_id())
//             .await?
//             .ok_or(PropError::NotFound(
//                 attribute_value.context.prop_id(),
//                 *ctx.visibility(),
//             ))?;
//         prop.set_default_value(ctx, color).await?;
//         Ok(())
//     }

//     standard_model_belongs_to!(
//         lookup_fn: schema,
//         set_fn: set_schema,
//         unset_fn: unset_schema,
//         table: "schema_variant_belongs_to_schema",
//         model_table: "schemas",
//         belongs_to_id: SchemaId,
//         returns: Schema,
//         result: SchemaVariantResult,
//     );

//     standard_model_many_to_many!(
//         lookup_fn: sockets,
//         associate_fn: add_socket,
//         disassociate_fn: remove_socket,
//         table_name: "socket_many_to_many_schema_variants",
//         left_table: "sockets",
//         left_id: SocketId,
//         right_table: "schema_variants",
//         right_id: SchemaId,
//         which_table_is_this: "right",
//         returns: Socket,
//         result: SchemaVariantResult,
//     );

//     /// List all direct child [`Props`](crate::Prop) of the [`Prop`](crate::Prop) corresponding
//     /// to "/root/si".
//     #[instrument(skip_all)]
//     pub async fn list_root_si_child_props(
//         ctx: &DalContext,
//         schema_variant_id: SchemaVariantId,
//     ) -> SchemaVariantResult<Vec<Prop>> {
//         let rows = ctx
//             .txns()
//             .await?
//             .pg()
//             .query(
//                 LIST_ROOT_SI_CHILD_PROPS,
//                 &[ctx.tenancy(), ctx.visibility(), &schema_variant_id],
//             )
//             .await?;
//         Ok(objects_from_rows(rows)?)
//     }

//     /// Find all [`Props`](crate::Prop) for a given [`SchemaVariantId`](SchemaVariant).
//     #[instrument(skip_all)]
//     pub async fn all_props(
//         ctx: &DalContext,
//         schema_variant_id: SchemaVariantId,
//     ) -> SchemaVariantResult<Vec<Prop>> {
//         let rows = ctx
//             .txns()
//             .await?
//             .pg()
//             .query(
//                 ALL_PROPS,
//                 &[ctx.tenancy(), ctx.visibility(), &schema_variant_id],
//             )
//             .await?;
//         Ok(objects_from_rows(rows)?)
//     }

//     /// Find all [`Func`](crate::Func) objects connected to this schema variant in any way. Only
//     /// finds funcs connected at the schema variant context, ignoring any funcs connected to
//     /// directly to components. Ignores any functions that have no code (these are typically
//     /// intrinsics)
//     #[instrument(skip_all)]
//     pub async fn all_funcs(
//         ctx: &DalContext,
//         schema_variant_id: SchemaVariantId,
//     ) -> SchemaVariantResult<Vec<Func>> {
//         let rows = ctx
//             .txns()
//             .await?
//             .pg()
//             .query(
//                 ALL_FUNCS,
//                 &[ctx.tenancy(), ctx.visibility(), &schema_variant_id],
//             )
//             .await?;

//         Ok(objects_from_rows(rows)?)
//     }

//     pub async fn upsert_leaf_function(
//         ctx: &DalContext,
//         schema_variant_id: SchemaVariantId,
//         component_id: Option<ComponentId>,
//         leaf_kind: LeafKind,
//         input_locations: &[LeafInputLocation],
//         func: &Func,
//     ) -> SchemaVariantResult<AttributePrototype> {
//         let leaf_prop =
//             SchemaVariant::find_leaf_item_prop(ctx, schema_variant_id, leaf_kind).await?;

//         let context = match component_id {
//             Some(component_id) => AttributeContextBuilder::new()
//                 .set_prop_id(*leaf_prop.id())
//                 .set_component_id(component_id)
//                 .to_context()?,
//             None => AttributeContextBuilder::new()
//                 .set_prop_id(*leaf_prop.id())
//                 .to_context()?,
//         };

//         let key = Some(func.name().to_string());
//         let mut existing_args = FuncArgument::list_for_func(ctx, *func.id()).await?;
//         let mut inputs = vec![];
//         for location in input_locations {
//             let arg_name = location.arg_name();
//             let arg = match existing_args.iter().find(|arg| arg.name() == arg_name) {
//                 Some(existing_arg) => existing_arg.clone(),
//                 None => {
//                     FuncArgument::new(ctx, arg_name, location.arg_kind(), None, *func.id()).await?
//                 }
//             };

//             inputs.push(LeafInput {
//                 location: *location,
//                 func_argument_id: *arg.id(),
//             });
//         }

//         for mut existing_arg in existing_args.drain(..) {
//             if !inputs.iter().any(
//                 |&LeafInput {
//                      func_argument_id, ..
//                  }| func_argument_id == *existing_arg.id(),
//             ) {
//                 existing_arg.delete_by_id(ctx).await?;
//             }
//         }

//         Ok(
//             match AttributePrototype::find_for_context_and_key(ctx, context, &key)
//                 .await?
//                 .pop()
//             {
//                 Some(existing_proto) => {
//                     let mut apas = AttributePrototypeArgument::list_for_attribute_prototype(
//                         ctx,
//                         *existing_proto.id(),
//                     )
//                     .await?;

//                     for input in &inputs {
//                         if !apas
//                             .iter()
//                             .any(|apa| apa.func_argument_id() == input.func_argument_id)
//                         {
//                             let input_internal_provider =
//                                 Self::find_root_child_implicit_internal_provider(
//                                     ctx,
//                                     schema_variant_id,
//                                     input.location.into(),
//                                 )
//                                 .await?;

//                             AttributePrototypeArgument::new_for_intra_component(
//                                 ctx,
//                                 *existing_proto.id(),
//                                 input.func_argument_id,
//                                 *input_internal_provider.id(),
//                             )
//                             .await?;
//                         }
//                     }

//                     for mut apa in apas.drain(..) {
//                         if !inputs.iter().any(
//                             |&LeafInput {
//                                  func_argument_id, ..
//                              }| {
//                                 func_argument_id == apa.func_argument_id()
//                             },
//                         ) {
//                             apa.delete_by_id(ctx).await?;
//                         }
//                     }

//                     existing_proto
//                 }
//                 None => {
//                     let (_, new_proto) = SchemaVariant::add_leaf(
//                         ctx,
//                         *func.id(),
//                         schema_variant_id,
//                         component_id,
//                         leaf_kind,
//                         inputs,
//                     )
//                     .await?;

//                     new_proto
//                 }
//             },
//         )
//     }

//     /// This method finds all the functions for a particular
//     /// ['LeafKind'](crate::schema::variant::leaves::LeafKind) for this SchemaVariant. For example,
//     /// it can find all Qualification functions for the variant.
//     pub async fn find_leaf_item_functions(
//         ctx: &DalContext,
//         schema_variant_id: SchemaVariantId,
//         leaf_kind: LeafKind,
//     ) -> SchemaVariantResult<Vec<(AttributePrototype, Func)>> {
//         let leaf_item_prop = Self::find_leaf_item_prop(ctx, schema_variant_id, leaf_kind).await?;
//         let backend_response_type: FuncBackendResponseType = leaf_kind.into();

//         let context = AttributeContextBuilder::new()
//             .set_prop_id(*leaf_item_prop.id())
//             .to_context()?;

//         Ok(
//             AttributePrototype::list_prototype_funcs_by_context_and_backend_response_type(
//                 ctx,
//                 context,
//                 backend_response_type,
//             )
//             .await?,
//         )
//     }

//     /// This method finds a [`leaf`](crate::schema::variant::leaves)'s entry
//     /// [`Prop`](crate::Prop) given a [`LeafKind`](crate::schema::variant::leaves::LeafKind).
//     pub async fn find_leaf_item_prop(
//         ctx: &DalContext,
//         schema_variant_id: SchemaVariantId,
//         leaf_kind: LeafKind,
//     ) -> SchemaVariantResult<Prop> {
//         let (leaf_map_prop_name, leaf_item_prop_name) = leaf_kind.prop_names();
//         let row = ctx
//             .txns()
//             .await?
//             .pg()
//             .query_one(
//                 FIND_LEAF_ITEM_PROP,
//                 &[
//                     ctx.tenancy(),
//                     ctx.visibility(),
//                     &schema_variant_id,
//                     &leaf_map_prop_name,
//                     &leaf_item_prop_name,
//                 ],
//             )
//             .await?;
//         Ok(object_from_row(row)?)
//     }

//     /// Find the implicit [`InternalProvider`](crate::InternalProvider) corresponding to a provided,
//     /// [`direct child`](crate::RootPropChild) of [`RootProp`](crate::RootProp).
//     pub async fn find_root_child_implicit_internal_provider(
//         ctx: &DalContext,
//         schema_variant_id: SchemaVariantId,
//         root_prop_child: RootPropChild,
//     ) -> SchemaVariantResult<InternalProvider> {
//         let row = ctx
//             .txns()
//             .await?
//             .pg()
//             .query_one(
//                 FIND_ROOT_CHILD_IMPLICIT_INTERNAL_PROVIDER,
//                 &[
//                     ctx.tenancy(),
//                     ctx.visibility(),
//                     &schema_variant_id,
//                     &root_prop_child.as_str(),
//                 ],
//             )
//             .await?;
//         Ok(object_from_row(row)?)
//     }

//     /// Call [`Self::find_root_prop`] with the [`SchemaVariantId`](SchemaVariant) off
//     /// [`self`](SchemaVariant).
//     pub async fn root_prop(&self, ctx: &DalContext) -> SchemaVariantResult<Option<Prop>> {
//         Self::find_root_prop(ctx, self.id).await
//     }

//     /// Find the [`Prop`](crate::Prop) corresponding to "/root" for a given
//     /// [`SchemaVariantId`](SchemaVariant).
//     pub async fn find_root_prop(
//         ctx: &DalContext,
//         schema_variant_id: SchemaVariantId,
//     ) -> SchemaVariantResult<Option<Prop>> {
//         let maybe_row = ctx
//             .txns()
//             .await?
//             .pg()
//             .query_opt(
//                 FIND_ROOT_PROP,
//                 &[ctx.tenancy(), ctx.visibility(), &schema_variant_id],
//             )
//             .await?;
//         Ok(option_object_from_row(maybe_row)?)
//     }

//     /// Find the [`SchemaVariant`] for a given [`PropId`](crate::Prop) that resides _anywhere_ in a
//     /// [`Prop`](crate::Prop) tree.
//     ///
//     /// For instance, if you have a [`PropId`](crate::Prop) corresponding to "/root/domain/poop"
//     /// and want to know what [`SchemaVariant`]'s [`Prop`](crate::Prop) tree it resides in, use this
//     /// method to find out.
//     pub async fn find_for_prop(
//         ctx: &DalContext,
//         prop_id: PropId,
//     ) -> SchemaVariantResult<Option<Self>> {
//         // FIXME(nick): this is expensive and should be one query. Please WON'T SOMEBODY THINK OF
//         // THE CPU AND THE DATABASE??? OHHHHHHH THE HUMANITY!!!!!!! Oh well, anyway.
//         if let Some(root_prop) = Prop::find_root_prop_for_prop(ctx, prop_id).await? {
//             for schema_variant in Self::list(ctx).await? {
//                 if let Some(populated_root_prop_id) = schema_variant.root_prop_id {
//                     if *root_prop.id() == populated_root_prop_id {
//                         return Ok(Some(schema_variant));
//                     }
//                 }
//             }
//         }
//         Ok(None)
//     }

//     /// Calls [`Self::find_prop_in_tree`] using the ID off of [`self`](SchemaVariant).
//     pub async fn find_prop(&self, ctx: &DalContext, path: &[&str]) -> SchemaVariantResult<Prop> {
//         Self::find_prop_in_tree(ctx, self.id, path).await
//     }

//     /// Find the [`Prop`] in a tree underneath our [`RootProp`] with a given path.
//     pub async fn find_prop_in_tree(
//         ctx: &DalContext,
//         schema_variant_id: SchemaVariantId,
//         path: &[&str],
//     ) -> SchemaVariantResult<Prop> {
//         match Prop::find_prop_by_path(ctx, schema_variant_id, &PropPath::new(path)).await {
//             Ok(prop) => Ok(prop),
//             Err(PropError::NotFoundAtPath(path, visiblity)) => Err(
//                 SchemaVariantError::PropNotFoundAtPath(schema_variant_id, path, visiblity),
//             ),
//             Err(err) => Err(err)?,
//         }
//     }
// }
