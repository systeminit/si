//! This module contains [`Component`], which is an instance of a
//! [`SchemaVariant`](crate::SchemaVariant) and a _model_ of a "real world resource".

use content_store::{Store, StoreError};
use serde::{Deserialize, Serialize};
use strum::EnumDiscriminants;
use strum::{AsRefStr, Display, EnumIter, EnumString};
use telemetry::prelude::*;
use thiserror::Error;
use tokio::sync::TryLockError;

use crate::attribute::value::AttributeValueError;
use crate::change_set_pointer::ChangeSetPointerError;
use crate::workspace_snapshot::content_address::ContentAddress;
use crate::workspace_snapshot::edge_weight::{
    EdgeWeight, EdgeWeightError, EdgeWeightKind, EdgeWeightKindDiscriminants,
};
use crate::workspace_snapshot::node_weight::category_node_weight::CategoryNodeKind;
use crate::workspace_snapshot::node_weight::{NodeWeight, NodeWeightError};
use crate::workspace_snapshot::WorkspaceSnapshotError;
use crate::{
    pk, AttributeValue, DalContext, SchemaVariantId, StandardModel, Timestamp, TransactionsError,
};

// pub mod code;
// pub mod diff;
// pub mod qualification;
// pub mod resource;
// pub mod status;
// pub mod validation;
// pub mod view;

// pub use view::{ComponentView, ComponentViewError, ComponentViewProperties};

#[derive(Debug, Error)]
pub enum ComponentError {
    #[error("attribute value error: {0}")]
    AttributeValue(#[from] AttributeValueError),
    #[error("change set error: {0}")]
    ChangeSet(#[from] ChangeSetPointerError),
    #[error("edge weight error: {0}")]
    EdgeWeight(#[from] EdgeWeightError),
    #[error("node weight error: {0}")]
    NodeWeight(#[from] NodeWeightError),
    #[error("store error: {0}")]
    Store(#[from] StoreError),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error("try lock error: {0}")]
    TryLock(#[from] TryLockError),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] WorkspaceSnapshotError),
}

pub type ComponentResult<T> = Result<T, ComponentError>;

pk!(ComponentId);

#[remain::sorted]
#[derive(
    AsRefStr,
    Clone,
    Copy,
    Debug,
    Deserialize,
    Display,
    EnumIter,
    EnumString,
    Eq,
    PartialEq,
    Serialize,
)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum ComponentKind {
    Credential,
    Standard,
}

impl Default for ComponentKind {
    fn default() -> Self {
        Self::Standard
    }
}

/// A [`Component`] is an instantiation of a [`SchemaVariant`](crate::SchemaVariant).
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct Component {
    id: ComponentId,
    #[serde(flatten)]
    timestamp: Timestamp,
    name: String,
    kind: ComponentKind,
    needs_destroy: bool,
}

#[derive(EnumDiscriminants, Serialize, Deserialize, PartialEq)]
pub enum ComponentContent {
    V1(ComponentContentV1),
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ComponentContentV1 {
    pub timestamp: Timestamp,
    pub name: String,
    pub kind: ComponentKind,
    pub needs_destroy: bool,
}

impl Component {
    pub fn assemble(id: ComponentId, inner: ComponentContentV1) -> Self {
        Self {
            id,
            timestamp: inner.timestamp,
            name: inner.name,
            kind: inner.kind,
            needs_destroy: inner.needs_destroy,
        }
    }

    pub async fn new(
        ctx: &DalContext,
        name: impl Into<String>,
        schema_variant_id: SchemaVariantId,
        component_kind: Option<ComponentKind>,
    ) -> ComponentResult<Self> {
        let content = ComponentContentV1 {
            timestamp: Timestamp::now(),
            name: name.into(),
            kind: match component_kind {
                Some(provided_kind) => provided_kind,
                None => ComponentKind::Standard,
            },
            needs_destroy: false,
        };
        let hash = ctx
            .content_store()
            .lock()
            .await
            .add(&ComponentContent::V1(content.clone()))?;

        let change_set = ctx.change_set_pointer()?;
        let id = change_set.generate_ulid()?;
        let node_weight =
            NodeWeight::new_content(&change_set, id, ContentAddress::Component(hash))?;
        let provider_indices = {
            let mut workspace_snapshot = ctx.workspace_snapshot()?.try_lock()?;
            workspace_snapshot.add_node(node_weight)?;

            // Root --> Component Category --> Component (this)
            let component_category_id =
                workspace_snapshot.get_category(CategoryNodeKind::Component)?;
            workspace_snapshot.add_edge(
                component_category_id,
                EdgeWeight::new(change_set, EdgeWeightKind::Use)?,
                id,
            )?;

            // Component (this) --> Schema Variant
            workspace_snapshot.add_edge(
                id,
                EdgeWeight::new(change_set, EdgeWeightKind::Use)?,
                schema_variant_id.into(),
            )?;

            // Collect all providers corresponding to input and output sockets for the schema
            // variant.
            workspace_snapshot.outgoing_targets_for_edge_weight_kind(
                schema_variant_id,
                EdgeWeightKindDiscriminants::Provider,
            )?
        };

        // Create attribute values for all providers corresponding to input and output sockets.
        for provider_index in provider_indices {
            let attribute_value = AttributeValue::new(ctx, false).await?;
            {
                let mut workspace_snapshot = ctx.workspace_snapshot()?.try_lock()?;

                // Component (this) --> AttributeValue (new)
                workspace_snapshot.add_edge(
                    id,
                    EdgeWeight::new(change_set, EdgeWeightKind::Use)?,
                    attribute_value.id().into(),
                )?;

                // AttributeValue (new) --> Provider (corresponding to an input or an output Socket)
                let attribute_value_index =
                    workspace_snapshot.get_node_index_by_id(attribute_value.id())?;
                workspace_snapshot.add_edge_unchecked(
                    attribute_value_index,
                    EdgeWeight::new(change_set, EdgeWeightKind::Provider)?,
                    provider_index,
                )?;
            }
        }

        Ok(Self::assemble(id.into(), content))
    }
}

// impl Component {
//     /// The primary constructor method for creating [`Components`](Self). It returns a new
//     /// [`Component`] with a corresponding [`Node`](crate::Node).
//     ///
//     /// If you would like to use the default [`SchemaVariant`](crate::SchemaVariant) for
//     /// a [`Schema`](crate::Schema) rather than
//     /// a specific [`SchemaVariantId`](crate::SchemaVariant), use
//     /// [`Self::new_for_default_variant_from_schema()`].
//     #[instrument(skip_all)]
//     pub async fn new(
//         ctx: &DalContext,
//         name: impl AsRef<str>,
//         schema_variant_id: SchemaVariantId,
//     ) -> ComponentResult<(Self, Node)> {
//         let schema_variant = SchemaVariant::get_by_id(ctx, &schema_variant_id)
//             .await?
//             .ok_or(SchemaVariantError::NotFound(schema_variant_id))?;

//         // Ensure components are not created unless the variant has been finalized at least once.
//         if !schema_variant.finalized_once() {
//             return Err(ComponentError::SchemaVariantNotFinalized(schema_variant_id));
//         }

//         let schema = schema_variant
//             .schema(ctx)
//             .await?
//             .ok_or(SchemaVariantError::MissingSchema(schema_variant_id))?;
//         let actor_user_pk = match ctx.history_actor() {
//             HistoryActor::User(user_pk) => Some(*user_pk),
//             _ => None,
//         };

//         let row = ctx
//             .txns()
//             .await?
//             .pg()
//             .query_one(
//                 "SELECT object FROM component_create_v1($1, $2, $3, $4)",
//                 &[
//                     ctx.tenancy(),
//                     ctx.visibility(),
//                     &actor_user_pk,
//                     &schema.component_kind().as_ref(),
//                 ],
//             )
//             .await?;

//         let component: Component = standard_model::finish_create_from_row(ctx, row).await?;
//         component.set_schema(ctx, schema.id()).await?;
//         component
//             .set_schema_variant(ctx, &schema_variant_id)
//             .await?;

//         // Need to flesh out node so that the template data is also included in the node we
//         // persist. But it isn't, - our node is anemic.
//         let node = Node::new(ctx, &NodeKind::Configuration).await?;
//         node.set_component(ctx, component.id()).await?;
//         component.set_name(ctx, Some(name.as_ref())).await?;

//         Ok((component, node))
//     }

//     /// A secondary constructor method that finds the default
//     /// [`SchemaVariant`](crate::SchemaVariant) for a given [`SchemaId`](crate::Schema). Once found,
//     /// the [`primary constructor method`](Self::new) is called.
//     pub async fn new_for_default_variant_from_schema(
//         ctx: &DalContext,
//         name: impl AsRef<str>,
//         schema_id: SchemaId,
//     ) -> ComponentResult<(Self, Node)> {
//         let schema = Schema::get_by_id(ctx, &schema_id)
//             .await?
//             .ok_or(SchemaError::NotFound(schema_id))?;

//         let schema_variant_id = schema
//             .default_schema_variant_id()
//             .ok_or(SchemaError::NoDefaultVariant(schema_id))?;

//         Self::new(ctx, name, *schema_variant_id).await
//     }

//     standard_model_accessor!(kind, Enum(ComponentKind), ComponentResult);
//     standard_model_accessor!(needs_destroy, bool, ComponentResult);
//     standard_model_accessor!(deletion_user_pk, Option<Pk(UserPk)>, ComponentResult);

//     standard_model_belongs_to!(
//         lookup_fn: schema,
//         set_fn: set_schema,
//         unset_fn: unset_schema,
//         table: "component_belongs_to_schema",
//         model_table: "schemas",
//         belongs_to_id: SchemaId,
//         returns: Schema,
//         result: ComponentResult,
//     );

//     standard_model_belongs_to!(
//         lookup_fn: schema_variant,
//         set_fn: set_schema_variant,
//         unset_fn: unset_schema_variant,
//         table: "component_belongs_to_schema_variant",
//         model_table: "schema_variants",
//         belongs_to_id: SchemaVariantId,
//         returns: SchemaVariant,
//         result: ComponentResult,
//     );

//     standard_model_has_many!(
//         lookup_fn: node,
//         table: "node_belongs_to_component",
//         model_table: "nodes",
//         returns: Node,
//         result: ComponentResult,
//     );

//     pub fn tenancy(&self) -> &Tenancy {
//         &self.tenancy
//     }

//     /// List [`Sockets`](crate::Socket) with a given
//     /// [`SocketEdgeKind`](crate::socket::SocketEdgeKind).
//     #[instrument(skip_all)]
//     pub async fn list_sockets_for_kind(
//         ctx: &DalContext,
//         component_id: ComponentId,
//         socket_edge_kind: SocketEdgeKind,
//     ) -> ComponentResult<Vec<Socket>> {
//         let rows = ctx
//             .txns()
//             .await?
//             .pg()
//             .query(
//                 LIST_SOCKETS_FOR_SOCKET_EDGE_KIND,
//                 &[
//                     ctx.tenancy(),
//                     ctx.visibility(),
//                     &component_id,
//                     &(socket_edge_kind.to_string()),
//                 ],
//             )
//             .await?;
//         Ok(standard_model::objects_from_rows(rows)?)
//     }

//     /// Find [`Self`] with a provided [`NodeId`](crate::Node).
//     #[instrument(skip_all)]
//     pub async fn find_for_node(ctx: &DalContext, node_id: NodeId) -> ComponentResult<Option<Self>> {
//         let row = ctx
//             .txns()
//             .await?
//             .pg()
//             .query_opt(FIND_FOR_NODE, &[ctx.tenancy(), ctx.visibility(), &node_id])
//             .await?;
//         Ok(standard_model::object_option_from_row_option(row)?)
//     }

//     /// Find the [`AttributeValue`](crate::AttributeValue) whose
//     /// [`context`](crate::AttributeContext) corresponds to the following:
//     ///
//     /// - The [`PropId`](crate::Prop) corresponding to the child [`Prop`](crate::Prop) of "/root/si"
//     ///   whose name matches the provided
//     ///   [`SiPropChild`](crate::schema::variant::root_prop::SiPropChild)
//     /// - The [`ComponentId`](Self) matching the provided [`ComponentId`](Self).
//     ///
//     /// _Note:_ if the type has never been updated, this will find the _default_
//     /// [`AttributeValue`](crate::AttributeValue) where the [`ComponentId`](Self) is unset.
//     #[instrument(skip_all)]
//     pub async fn find_si_child_attribute_value(
//         ctx: &DalContext,
//         component_id: ComponentId,
//         schema_variant_id: SchemaVariantId,
//         si_prop_child: SiPropChild,
//     ) -> ComponentResult<AttributeValue> {
//         let si_child_prop_name = si_prop_child.prop_name();
//         let row = ctx
//             .txns()
//             .await?
//             .pg()
//             .query_one(
//                 FIND_SI_CHILD_PROP_ATTRIBUTE_VALUE,
//                 &[
//                     ctx.tenancy(),
//                     ctx.visibility(),
//                     &component_id,
//                     &schema_variant_id,
//                     &si_child_prop_name,
//                 ],
//             )
//             .await?;
//         Ok(object_from_row(row)?)
//     }

//     #[instrument(skip_all)]
//     pub async fn is_in_tenancy(ctx: &DalContext, id: ComponentId) -> ComponentResult<bool> {
//         let row = ctx
//             .txns()
//             .await?
//             .pg()
//             .query_opt(
//                 "SELECT id FROM components WHERE id = $1 AND in_tenancy_v1($2, components.tenancy_workspace_pk) LIMIT 1",
//                 &[
//                     &id,
//                     ctx.tenancy(),
//                 ],
//             )
//             .await?;
//         Ok(row.is_some())
//     }

//     #[instrument(skip_all)]
//     pub async fn list_for_schema_variant(
//         ctx: &DalContext,
//         schema_variant_id: SchemaVariantId,
//     ) -> ComponentResult<Vec<Component>> {
//         let rows = ctx
//             .txns()
//             .await?
//             .pg()
//             .query(
//                 LIST_FOR_SCHEMA_VARIANT,
//                 &[ctx.tenancy(), ctx.visibility(), &schema_variant_id],
//             )
//             .await?;

//         let mut results = Vec::new();
//         for row in rows.into_iter() {
//             let json: serde_json::Value = row.try_get("object")?;
//             let object: Self = serde_json::from_value(json)?;
//             results.push(object);
//         }

//         Ok(results)
//     }

//     /// Sets the "/root/si/name" for [`self`](Self).
//     #[instrument(skip_all)]
//     pub async fn set_name<T: Serialize + std::fmt::Debug + std::clone::Clone>(
//         &self,
//         ctx: &DalContext,
//         value: Option<T>,
//     ) -> ComponentResult<()> {
//         let schema_variant_id = Self::schema_variant_id(ctx, self.id).await?;
//         let attribute_value =
//             Self::find_si_child_attribute_value(ctx, self.id, schema_variant_id, SiPropChild::Name)
//                 .await?;

//         // Before we set the name, ensure that another function is not setting the name (e.g.
//         // something different than "unset" or "setString").
//         let attribute_prototype = attribute_value
//             .attribute_prototype(ctx)
//             .await?
//             .ok_or_else(|| ComponentError::MissingAttributePrototype(*attribute_value.id()))?;
//         let prototype_func = Func::get_by_id(ctx, &attribute_prototype.func_id())
//             .await?
//             .ok_or_else(|| {
//                 ComponentError::MissingAttributePrototypeFunction(*attribute_prototype.id())
//             })?;
//         let name = prototype_func.name();
//         if name != "si:unset" && name != "si:setString" {
//             return Ok(());
//         }

//         let attribute_context = AttributeContext::builder()
//             .set_component_id(self.id)
//             .set_prop_id(attribute_value.context.prop_id())
//             .to_context()?;

//         let json_value = match value.clone() {
//             Some(v) => Some(serde_json::to_value(v)?),
//             None => None,
//         };

//         let parent_attribute_value = attribute_value
//             .parent_attribute_value(ctx)
//             .await?
//             .ok_or_else(|| ComponentError::ParentAttributeValueNotFound(*attribute_value.id()))?;
//         let (_, _) = AttributeValue::update_for_context(
//             ctx,
//             *attribute_value.id(),
//             Some(*parent_attribute_value.id()),
//             attribute_context,
//             json_value,
//             None,
//         )
//         .await?;

//         Ok(())
//     }

//     #[instrument(skip_all)]
//     pub async fn set_deleted_at(
//         &self,
//         ctx: &DalContext,
//         value: Option<DateTime<Utc>>,
//     ) -> ComponentResult<Option<DateTime<Utc>>> {
//         let json_value = match value {
//             Some(v) => Some(serde_json::to_value(v)?),
//             None => None,
//         };

//         let attribute_value = Self::root_prop_child_attribute_value_for_component(
//             ctx,
//             self.id,
//             RootPropChild::DeletedAt,
//         )
//         .await?;
//         let parent_attribute_value = attribute_value
//             .parent_attribute_value(ctx)
//             .await?
//             .ok_or_else(|| ComponentError::ParentAttributeValueNotFound(*attribute_value.id()))?;
//         let attribute_context = AttributeContext::builder()
//             .set_component_id(self.id)
//             .set_prop_id(attribute_value.context.prop_id())
//             .to_context()?;
//         let (_, _) = AttributeValue::update_for_context(
//             ctx,
//             *attribute_value.id(),
//             Some(*parent_attribute_value.id()),
//             attribute_context,
//             json_value,
//             None,
//         )
//         .await?;

//         Ok(value)
//     }

//     /// Return the name of the [`Component`](Self) for the provided [`ComponentId`](Self).
//     #[instrument(skip_all)]
//     pub async fn find_name(ctx: &DalContext, component_id: ComponentId) -> ComponentResult<String> {
//         let row = ctx
//             .txns()
//             .await?
//             .pg()
//             .query_one(FIND_NAME, &[ctx.tenancy(), ctx.visibility(), &component_id])
//             .await?;
//         let component_name: Value = row.try_get("component_name")?;
//         let component_name: Option<String> = serde_json::from_value(component_name)?;
//         let component_name = component_name.ok_or(ComponentError::NameIsUnset(component_id))?;
//         Ok(component_name)
//     }

//     /// Calls [`Self::find_name()`] and provides the "id" off [`self`](Self).
//     pub async fn name(&self, ctx: &DalContext) -> ComponentResult<String> {
//         Self::find_name(ctx, self.id).await
//     }

//     /// Grabs the [`AttributeValue`](crate::AttributeValue) corresponding to the
//     /// [`RootPropChild`](crate::RootPropChild) [`Prop`](crate::Prop) for the given
//     /// [`Component`](Self).
//     #[instrument(skip_all)]
//     pub async fn root_prop_child_attribute_value_for_component(
//         ctx: &DalContext,
//         component_id: ComponentId,
//         root_prop_child: RootPropChild,
//     ) -> ComponentResult<AttributeValue> {
//         let row = ctx
//             .txns()
//             .await?
//             .pg()
//             .query_one(
//                 ROOT_CHILD_ATTRIBUTE_VALUE_FOR_COMPONENT,
//                 &[
//                     ctx.tenancy(),
//                     ctx.visibility(),
//                     &root_prop_child.as_str(),
//                     &component_id,
//                 ],
//             )
//             .await?;
//         Ok(object_from_row(row)?)
//     }

//     /// List the connected input [`Sockets`](crate::Socket) for a given [`ComponentId`](Self) and
//     /// [`AttributeValueId`](crate::AttributeValue) whose [`context`](crate::AttributeContext)'s
//     /// least specific field corresponding to a [`PropId`](crate::Prop). In other words, this is
//     /// the list of input [`Sockets`](crate::Socket) with incoming connections from other
//     /// [`Component(s)`](Self) that the given [`AttributeValue`](crate::AttributeValue) depends on.
//     ///
//     /// ```raw
//     ///                      ┌────────────────────────────┐
//     ///                      │ This                       │
//     ///                      │ Component                  │
//     /// ┌───────────┐        │         ┌────────────────┐ │
//     /// │ Another   │        │    ┌───►│ AttributeValue │ │
//     /// │ Component │        │    │    │ for Prop       │ │
//     /// │           │        │    │    └────────────────┘ │
//     /// │  ┌────────┤        ├────┴─────────┐             │
//     /// │  │ Output ├───────►│ Input        │             │
//     /// │  │ Socket │        │ Socket       │             │
//     /// │  │        │        │ (list these) │             │
//     /// └──┴────────┘        └──────────────┴─────────────┘
//     /// ```
//     ///
//     /// _Warning: users of this query must ensure that the
//     /// [`AttributeValueId`](crate::AttributeValue) provided has a
//     /// [`context`](crate::AttributeContext) whose least specific field corresponds to a
//     /// [`PropId`](crate::Prop)._
//     #[instrument(skip_all)]
//     pub async fn list_connected_input_sockets_for_attribute_value(
//         ctx: &DalContext,
//         attribute_value_id: AttributeValueId,
//         component_id: ComponentId,
//     ) -> ComponentResult<Vec<Socket>> {
//         let rows = ctx
//             .txns()
//             .await?
//             .pg()
//             .query(
//                 LIST_CONNECTED_INPUT_SOCKETS_FOR_ATTRIBUTE_VALUE,
//                 &[
//                     ctx.tenancy(),
//                     ctx.visibility(),
//                     &attribute_value_id,
//                     &component_id,
//                 ],
//             )
//             .await?;
//         Ok(standard_model::objects_from_rows(rows)?)
//     }

//     /// Find the [`SchemaVariantId`](crate::SchemaVariantId) that belongs to the provided
//     /// [`Component`](crate::Component).
//     pub async fn schema_variant_id(
//         ctx: &DalContext,
//         component_id: ComponentId,
//     ) -> ComponentResult<SchemaVariantId> {
//         let row = ctx
//             .txns()
//             .await?
//             .pg()
//             .query_one(
//                 "select belongs_to_id as schema_variant_id from
//                     component_belongs_to_schema_variant_v1($1, $2)
//                     where object_id = $3
//                 ",
//                 &[ctx.tenancy(), ctx.visibility(), &component_id],
//             )
//             .await?;

//         Ok(row.try_get("schema_variant_id")?)
//     }

//     /// Find the [`SchemaId`](crate::SchemaId) that belongs to the provided
//     /// [`Component`](crate::Component).
//     pub async fn schema_id(
//         ctx: &DalContext,
//         component_id: ComponentId,
//     ) -> ComponentResult<SchemaId> {
//         let row = ctx
//             .txns()
//             .await?
//             .pg()
//             .query_one(
//                 "select belongs_to_id as schema_id from
//                     component_belongs_to_schema_v1($1, $2)
//                     where object_id = $3
//                 ",
//                 &[ctx.tenancy(), ctx.visibility(), &component_id],
//             )
//             .await?;

//         Ok(row.try_get("schema_id")?)
//     }

//     /// Gets the [`ComponentType`](crate::ComponentType) of [`self`](Self).
//     ///
//     /// Mutate this with [`Self::set_type()`].
//     pub async fn get_type(&self, ctx: &DalContext) -> ComponentResult<ComponentType> {
//         let schema_variant_id = Self::schema_variant_id(ctx, self.id).await?;
//         let type_attribute_value =
//             Self::find_si_child_attribute_value(ctx, self.id, schema_variant_id, SiPropChild::Type)
//                 .await?;
//         let raw_value = type_attribute_value.get_value(ctx).await?.ok_or_else(|| {
//             ComponentError::ComponentTypeIsNone(self.id, *type_attribute_value.id())
//         })?;
//         let component_type: ComponentType = serde_json::from_value(raw_value)?;
//         Ok(component_type)
//     }

//     /// Gets the protected attribute value of [`self`](Self).
//     pub async fn get_protected(&self, ctx: &DalContext) -> ComponentResult<bool> {
//         let schema_variant_id = Self::schema_variant_id(ctx, self.id).await?;
//         let protected_attribute_value = Self::find_si_child_attribute_value(
//             ctx,
//             self.id,
//             schema_variant_id,
//             SiPropChild::Protected,
//         )
//         .await?;
//         let raw_value = protected_attribute_value
//             .get_value(ctx)
//             .await?
//             .ok_or_else(|| {
//                 ComponentError::ComponentProtectionIsNone(self.id, *protected_attribute_value.id())
//             })?;
//         let protected: bool = serde_json::from_value(raw_value)?;
//         Ok(protected)
//     }

//     /// Sets the field corresponding to "/root/si/type" for the [`Component`]. Possible values
//     /// are limited to variants of [`ComponentType`](crate::ComponentType).
//     pub async fn set_type(
//         &self,
//         ctx: &DalContext,
//         component_type: ComponentType,
//     ) -> ComponentResult<()> {
//         let schema_variant_id = Self::schema_variant_id(ctx, self.id).await?;
//         let type_attribute_value =
//             Self::find_si_child_attribute_value(ctx, self.id, schema_variant_id, SiPropChild::Type)
//                 .await?;

//         // If we are setting the type for the first time, we will need to mutate the context to
//         // be component-specific. This is because the attribute value will have an unset component
//         // id and we will need to deviate from the schema variant default component type.
//         let attribute_context = if type_attribute_value.context.is_component_unset() {
//             AttributeContextBuilder::from(type_attribute_value.context)
//                 .set_component_id(self.id)
//                 .to_context()?
//         } else {
//             type_attribute_value.context
//         };

//         let si_attribute_value = type_attribute_value
//             .parent_attribute_value(ctx)
//             .await?
//             .ok_or_else(|| {
//                 ComponentError::ParentAttributeValueNotFound(*type_attribute_value.id())
//             })?;
//         AttributeValue::update_for_context(
//             ctx,
//             *type_attribute_value.id(),
//             Some(*si_attribute_value.id()),
//             attribute_context,
//             Some(serde_json::to_value(component_type)?),
//             None,
//         )
//         .await?;

//         // Now that we've updated the field, we need to see if we need to do additional work.
//         let schema_variant = self
//             .schema_variant(ctx)
//             .await?
//             .ok_or(ComponentError::NoSchemaVariant(self.id))?;
//         let external_providers =
//             ExternalProvider::list_for_schema_variant(ctx, *schema_variant.id()).await?;
//         let internal_providers =
//             InternalProvider::list_explicit_for_schema_variant(ctx, *schema_variant.id()).await?;

//         // We have some work to do for all component types, but the aggregation frames need a
//         // special look.
//         if let ComponentType::AggregationFrame = component_type {
//             let (func, func_binding, func_binding_return_value) =
//                 Func::identity_with_binding_and_return_value(ctx).await?;
//             let func_id = *func.id();

//             for external_provider in external_providers {
//                 let attribute_read_context = AttributeReadContext {
//                     prop_id: Some(PropId::NONE),
//                     internal_provider_id: Some(InternalProviderId::NONE),
//                     external_provider_id: Some(*external_provider.id()),
//                     component_id: Some(self.id),
//                 };

//                 let attribute_context =
//                     AttributeContextBuilder::from(attribute_read_context).to_context()?;

//                 let attribute_value = AttributeValue::find_for_context(ctx, attribute_read_context)
//                     .await?
//                     .ok_or(ComponentError::AttributeValueNotFoundForContext(
//                         attribute_read_context,
//                     ))?;

//                 if attribute_value.context.is_component_unset() {
//                     AttributePrototype::new(
//                         ctx,
//                         func_id,
//                         *func_binding.id(),
//                         *func_binding_return_value.id(),
//                         attribute_context,
//                         None,
//                         None,
//                     )
//                     .await?;
//                 } else {
//                     AttributePrototype::new_with_existing_value(
//                         ctx,
//                         func_id,
//                         attribute_context,
//                         None,
//                         None,
//                         *attribute_value.id(),
//                     )
//                     .await?;
//                 };
//             }

//             for internal_provider in internal_providers {
//                 let attribute_read_context = AttributeReadContext {
//                     prop_id: Some(PropId::NONE),
//                     internal_provider_id: Some(*internal_provider.id()),
//                     external_provider_id: Some(ExternalProviderId::NONE),
//                     component_id: Some(self.id),
//                 };

//                 let attr_write_context =
//                     AttributeContextBuilder::from(attribute_read_context).to_context()?;

//                 let attribute_value = AttributeValue::find_for_context(ctx, attribute_read_context)
//                     .await?
//                     .ok_or(ComponentError::AttributeValueNotFoundForContext(
//                         attribute_read_context,
//                     ))?;

//                 let prototype =
//                     attribute_value
//                         .attribute_prototype(ctx)
//                         .await?
//                         .ok_or_else(|| {
//                             ComponentError::MissingAttributePrototype(*attribute_value.id())
//                         })?;

//                 let arguments = AttributePrototypeArgument::find_by_attr(
//                     ctx,
//                     "attribute_prototype_id",
//                     prototype.id(),
//                 )
//                 .await?;

//                 let new_prototype = if attribute_value.context.is_component_unset() {
//                     AttributePrototype::new(
//                         ctx,
//                         func_id,
//                         *func_binding.id(),
//                         *func_binding_return_value.id(),
//                         attr_write_context,
//                         None,
//                         None,
//                     )
//                     .await?
//                 } else {
//                     AttributePrototype::new_with_existing_value(
//                         ctx,
//                         func_id,
//                         attr_write_context,
//                         None,
//                         None,
//                         *attribute_value.id(),
//                     )
//                     .await?
//                 };

//                 for argument in arguments {
//                     AttributePrototypeArgument::new_for_inter_component(
//                         ctx,
//                         *new_prototype.id(),
//                         argument.func_argument_id(),
//                         argument.head_component_id(),
//                         argument.tail_component_id(),
//                         argument.external_provider_id(),
//                     )
//                     .await?;
//                 }
//             }
//         } else {
//             for external_provider in external_providers {
//                 let attribute_read_context = AttributeReadContext {
//                     prop_id: Some(PropId::NONE),
//                     internal_provider_id: Some(InternalProviderId::NONE),
//                     external_provider_id: Some(*external_provider.id()),
//                     component_id: Some(self.id),
//                 };

//                 let mut attribute_value =
//                     AttributeValue::find_for_context(ctx, attribute_read_context)
//                         .await?
//                         .ok_or(ComponentError::AttributeValueNotFoundForContext(
//                             attribute_read_context,
//                         ))?;

//                 if !attribute_value.context.is_component_unset() {
//                     attribute_value.unset_attribute_prototype(ctx).await?;
//                     attribute_value.delete_by_id(ctx).await?;
//                 }
//             }

//             for internal_provider in internal_providers {
//                 let attribute_read_context = AttributeReadContext {
//                     prop_id: Some(PropId::NONE),
//                     internal_provider_id: Some(*internal_provider.id()),
//                     external_provider_id: Some(ExternalProviderId::NONE),
//                     component_id: Some(self.id),
//                 };

//                 let mut attribute_value =
//                     AttributeValue::find_for_context(ctx, attribute_read_context)
//                         .await?
//                         .ok_or(ComponentError::AttributeValueNotFoundForContext(
//                             attribute_read_context,
//                         ))?;

//                 if !attribute_value.context.is_component_unset() {
//                     attribute_value.unset_attribute_prototype(ctx).await?;
//                     attribute_value.delete_by_id(ctx).await?;
//                 }
//             }
//         }

//         Ok(())
//     }

//     pub async fn delete_and_propagate(&mut self, ctx: &DalContext) -> ComponentResult<()> {
//         // Block deletion of frames with children
//         if self.get_type(ctx).await? != ComponentType::Component {
//             let frame_edges = Edge::list_for_component(ctx, self.id).await?;
//             let frame_node = self
//                 .node(ctx)
//                 .await?
//                 .pop()
//                 .ok_or(ComponentError::NodeNotFoundForComponent(self.id))?;
//             let frame_socket = Socket::find_frame_socket_for_node(
//                 ctx,
//                 *frame_node.id(),
//                 SocketEdgeKind::ConfigurationInput,
//             )
//             .await?;
//             let connected_children = frame_edges
//                 .into_iter()
//                 .filter(|edge| edge.head_socket_id() == *frame_socket.id())
//                 .count();
//             if connected_children > 0 {
//                 return Err(ComponentError::FrameHasAttachedComponents);
//             }
//         }

//         self.set_deleted_at(ctx, Some(Utc::now())).await?;

//         if self.get_protected(ctx).await? {
//             return Err(ComponentError::ComponentProtected(self.id));
//         }

//         let actor_user_pk = match ctx.history_actor() {
//             HistoryActor::User(user_pk) => Some(*user_pk),
//             _ => None,
//         };

//         let has_resource = self.resource(ctx).await?.payload.is_some();
//         let rows = ctx
//             .txns()
//             .await?
//             .pg()
//             .query(
//                 "SELECT * FROM component_delete_and_propagate_v1($1, $2, $3, $4, $5)",
//                 &[
//                     ctx.tenancy(),
//                     ctx.visibility(),
//                     self.id(),
//                     &actor_user_pk,
//                     &has_resource,
//                 ],
//             )
//             .await?;
//         let mut attr_values: Vec<AttributeValue> = standard_model::objects_from_rows(rows)?;

//         for attr_value in attr_values.iter_mut() {
//             attr_value.update_from_prototype_function(ctx).await?;
//         }

//         let ids = attr_values.iter().map(|av| *av.id()).collect();

//         ctx.enqueue_job(DependentValuesUpdate::new(
//             ctx.access_builder(),
//             *ctx.visibility(),
//             ids,
//         ))
//         .await?;

//         Ok(())
//     }

//     pub async fn restore_and_propagate(
//         ctx: &DalContext,
//         component_id: ComponentId,
//     ) -> ComponentResult<Option<Self>> {
//         // Check if component has deleted frame before restoring
//         let component = {
//             let ctx_with_deleted = &ctx.clone_with_delete_visibility();

//             let component = Self::get_by_id(ctx_with_deleted, &component_id)
//                 .await?
//                 .ok_or_else(|| ComponentError::NotFound(component_id))?;

//             let sockets = Socket::list_for_component(ctx_with_deleted, component_id).await?;

//             let maybe_socket_to_parent = sockets.iter().find(|socket| {
//                 socket.name() == "Frame"
//                     && *socket.edge_kind() == SocketEdgeKind::ConfigurationOutput
//             });

//             let edges_with_deleted = Edge::list(ctx_with_deleted).await?;

//             let mut maybe_deleted_parent_id = None;

//             if let Some(socket_to_parent) = maybe_socket_to_parent {
//                 for edge in &edges_with_deleted {
//                     if edge.tail_object_id() == (*component.id()).into()
//                         && edge.tail_socket_id() == *socket_to_parent.id()
//                         && (edge.visibility().deleted_at.is_some() && edge.deleted_implicitly())
//                     {
//                         maybe_deleted_parent_id = Some(edge.head_object_id().into());
//                         break;
//                     }
//                 }
//             };

//             if let Some(parent_id) = maybe_deleted_parent_id {
//                 let parent_comp = Self::get_by_id(ctx_with_deleted, &parent_id)
//                     .await?
//                     .ok_or_else(|| ComponentError::NotFound(parent_id))?;

//                 if parent_comp.visibility().deleted_at.is_some() {
//                     return Err(ComponentError::InsideDeletedFrame(component_id, parent_id));
//                 }
//             }

//             component
//         };

//         component.set_deleted_at(ctx, None).await?;

//         let rows = ctx
//             .txns()
//             .await?
//             .pg()
//             .query(
//                 "SELECT * FROM component_restore_and_propagate_v1($1, $2, $3)",
//                 &[ctx.tenancy(), ctx.visibility(), &component_id],
//             )
//             .await?;
//         let mut attr_values: Vec<AttributeValue> = standard_model::objects_from_rows(rows)?;

//         for attr_value in &mut attr_values {
//             attr_value.update_from_prototype_function(ctx).await?;
//         }

//         let ids = attr_values.iter().map(|av| *av.id()).collect();

//         ctx.enqueue_job(DependentValuesUpdate::new(
//             ctx.access_builder(),
//             *ctx.visibility(),
//             ids,
//         ))
//         .await?;

//         Ok(Component::get_by_id(ctx, &component_id).await?)
//     }

//     /// Finds the "color" that the [`Component`] should be in the [`Diagram`](crate::Diagram).
//     pub async fn color(&self, ctx: &DalContext) -> ComponentResult<Option<String>> {
//         let schema_variant_id = Self::schema_variant_id(ctx, self.id).await?;
//         let color_attribute_value = Component::find_si_child_attribute_value(
//             ctx,
//             self.id,
//             schema_variant_id,
//             SiPropChild::Color,
//         )
//         .await?;
//         let color = color_attribute_value
//             .get_value(ctx)
//             .await?
//             .map(serde_json::from_value)
//             .transpose()?;
//         Ok(color)
//     }

//     /// Check if the [`Component`] has been fully destroyed.
//     pub fn is_destroyed(&self) -> bool {
//         self.visibility.deleted_at.is_some() && !self.needs_destroy()
//     }
// }

// #[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
// #[serde(rename_all = "camelCase")]
// pub struct ComponentCreatedPayload {
//     success: bool,
// }

// impl WsEvent {
//     pub async fn component_created(ctx: &DalContext) -> WsEventResult<Self> {
//         WsEvent::new(
//             ctx,
//             WsPayload::ComponentCreated(ComponentCreatedPayload { success: true }),
//         )
//         .await
//     }
// }
