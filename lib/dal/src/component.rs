//! This module contains [`Component`], which is an instance of a
//! [`SchemaVariant`](crate::SchemaVariant) and a _model_ of a "real world resource".

use serde::{Deserialize, Serialize};
use serde_json::Value;
use si_data_nats::NatsError;
use si_data_pg::PgError;
use strum_macros::{AsRefStr, Display, EnumIter, EnumString};
use telemetry::prelude::*;
use thiserror::Error;

use crate::attribute::context::AttributeContextBuilder;
use crate::attribute::value::AttributeValue;
use crate::attribute::value::AttributeValueError;
use crate::code_view::CodeViewError;
use crate::func::binding::FuncBindingError;
use crate::func::binding_return_value::{
    FuncBindingReturnValue, FuncBindingReturnValueError, FuncBindingReturnValueId,
};
use crate::schema::variant::{SchemaVariantError, SchemaVariantId};
use crate::schema::SchemaVariant;
use crate::socket::SocketEdgeKind;
use crate::standard_model::object_from_row;
use crate::validation::ValidationConstructorError;
use crate::ws_event::WsEventError;
use crate::NodeKind;
use crate::{
    func::FuncId, impl_standard_model, node::NodeId, pk, provider::internal::InternalProviderError,
    standard_model, standard_model_accessor, standard_model_belongs_to, standard_model_has_many,
    ActionPrototypeError, AttributeContext, AttributeContextBuilderError, AttributeContextError,
    AttributePrototype, AttributePrototypeArgument, AttributePrototypeArgumentError,
    AttributePrototypeError, AttributePrototypeId, AttributeReadContext, CodeLanguage,
    ComponentType, DalContext, Edge, EdgeError, ExternalProvider, ExternalProviderError,
    ExternalProviderId, Func, FuncBackendKind, FuncError, HistoryActor, HistoryEventError,
    InternalProvider, InternalProviderId, Node, NodeError, OrganizationError, Prop, PropError,
    PropId, ReadTenancyError, RootPropChild, Schema, SchemaError, SchemaId, Socket, StandardModel,
    StandardModelError, Timestamp, TransactionsError, ValidationPrototypeError,
    ValidationResolverError, Visibility, WorkflowRunnerError, WorkspaceError, WriteTenancy,
};
use crate::{AttributeValueId, QualificationError};

pub use view::{ComponentView, ComponentViewError};

pub mod code;
pub mod confirmation;
pub mod diff;
pub mod qualification;
pub mod resource;
pub mod status;
pub mod validation;
pub mod view;

#[derive(Error, Debug)]
pub enum ComponentError {
    #[error("attribute context error: {0}")]
    AttributeContext(#[from] AttributeContextError),
    #[error("attribute context builder error: {0}")]
    AttributeContextBuilder(#[from] AttributeContextBuilderError),
    #[error(transparent)]
    AttributePrototype(#[from] AttributePrototypeError),
    #[error("attribute value error: {0}")]
    AttributeValue(#[from] AttributeValueError),
    #[error("attribute value not found for context: {0:?}")]
    AttributeValueNotFoundForContext(AttributeReadContext),
    #[error("invalid json pointer: {0} for {1}")]
    BadJsonPointer(String, String),
    #[error("code generation function returned unexpected format, expected {0:?}, got {1:?}")]
    CodeLanguageMismatch(CodeLanguage, CodeLanguage),
    #[error(transparent)]
    CodeView(#[from] CodeViewError),
    #[error("edge error: {0}")]
    Edge(#[from] EdgeError),
    #[error("unable to delete frame")]
    Frame,
    #[error("component marked as protected: {0}")]
    ComponentProtected(ComponentId),
    #[error(transparent)]
    WorkflowRunner(#[from] WorkflowRunnerError),
    #[error(transparent)]
    ActionPrototype(#[from] ActionPrototypeError),
    #[error("func not found: {0}")]
    FuncNotFound(FuncId),
    #[error(transparent)]
    FuncBindingReturnValue(#[from] FuncBindingReturnValueError),
    #[error("internal provider error: {0}")]
    InternalProvider(#[from] InternalProviderError),
    #[error("internal provider not found for prop: {0}")]
    InternalProviderNotFoundForProp(PropId),
    #[error("invalid context(s) provided for diff")]
    InvalidContextForDiff,
    #[error("invalid func backend kind (0:?) for checking validations (need validation kind)")]
    InvalidFuncBackendKindForValidations(FuncBackendKind),
    #[error("missing attribute value for id: ({0})")]
    MissingAttributeValue(AttributeValueId),
    #[error("missing index map on attribute value: {0}")]
    MissingIndexMap(AttributeValueId),
    #[error("expected one root prop, found multiple: {0:?}")]
    MultipleRootProps(Vec<Prop>),
    #[error("root prop not found for schema variant: {0}")]
    RootPropNotFound(SchemaVariantId),
    #[error("validation error: {0}")]
    Validation(#[from] ValidationConstructorError),
    #[error("attribute value does not have a prototype: {0}")]
    MissingAttributePrototype(AttributeValueId),
    #[error("attribute prototype does not have a function: {0}")]
    MissingAttributePrototypeFunction(AttributePrototypeId),
    #[error("/root/si/name is unset for component {0}")]
    NameIsUnset(ComponentId),
    #[error(transparent)]
    ComponentView(#[from] ComponentViewError),
    #[error("unable to find code generated")]
    CodeGeneratedNotFound,
    #[error("error serializing/deserializing json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error(transparent)]
    PgPool(#[from] si_data_pg::PgPoolError),
    #[error(transparent)]
    ContextTransaction(#[from] TransactionsError),
    #[error("nats txn error: {0}")]
    Nats(#[from] NatsError),
    #[error("history event error: {0}")]
    HistoryEvent(#[from] HistoryEventError),
    #[error("standard model error: {0}")]
    StandardModelError(#[from] StandardModelError),
    #[error("node error: {0}")]
    NodeError(#[from] NodeError),
    #[error("component not found: {0}")]
    NotFound(ComponentId),
    #[error("prop error: {0}")]
    Prop(#[from] PropError),
    #[error("schema error: {0}")]
    Schema(#[from] SchemaError),
    #[error("no schema variant for component {0}")]
    NoSchemaVariant(ComponentId),
    #[error("no schema for component {0}")]
    NoSchema(ComponentId),
    #[error("schema variant error: {0}")]
    SchemaVariant(#[from] SchemaVariantError),
    #[error("missing a prop in attribute update: {0} not found")]
    MissingProp(PropId),
    #[error("missing a prop in attribute update: {0} not found")]
    PropNotFound(String),
    #[error("missing a func in attribute update: {0} not found")]
    MissingFunc(String),
    #[error("func binding return value: {0} not found")]
    FuncBindingReturnValueNotFound(FuncBindingReturnValueId),
    #[error("invalid prop value kind; expected {0} but found {1}")]
    InvalidPropValue(&'static str, Value),
    #[error("func error: {0}")]
    Func(#[from] FuncError),
    #[error("func binding error: {0}")]
    FuncBinding(#[from] FuncBindingError),
    #[error("validation resolver error: {0}")]
    ValidationResolver(#[from] ValidationResolverError),
    #[error("validation prototype error: {0}")]
    ValidationPrototype(#[from] ValidationPrototypeError),
    #[error("validation prototype does not match component schema variant: {0}")]
    ValidationPrototypeMismatch(SchemaVariantId),
    #[error("qualification error: {0}")]
    Qualification(#[from] QualificationError),
    #[error("read tenancy error: {0}")]
    ReadTenancy(#[from] ReadTenancyError),
    #[error("workspace not found")]
    WorkspaceNotFound,
    #[error("organization not found")]
    OrganizationNotFound,
    #[error("ws event error: {0}")]
    WsEvent(#[from] WsEventError),
    #[error("workspace error: {0}")]
    Workspace(#[from] WorkspaceError),
    #[error("organization error: {0}")]
    Organization(#[from] OrganizationError),
    #[error("invalid AttributeReadContext: {0}")]
    BadAttributeReadContext(String),
    #[error("found child attribute value of a map without a key: {0}")]
    FoundMapEntryWithoutKey(AttributeValueId),
    #[error("schema variant has not been finalized at least once: {0}")]
    SchemaVariantNotFinalized(SchemaVariantId),
    #[error("cannot update the resource tree when in a change set")]
    CannotUpdateResourceTreeInChangeSet,
    #[error("no func binding return value for leaf entry name: {0}")]
    MissingFuncBindingReturnValueIdForLeafEntryName(String),

    /// Found an [`AttributePrototypeArgumentError`](crate::AttributePrototypeArgumentError).
    #[error("attribute prototype argument error: {0}")]
    AttributePrototypeArgument(#[from] AttributePrototypeArgumentError),
    /// Found an [`ExternalProviderError`](crate::ExternalProviderError).
    #[error("external provider error: {0}")]
    ExternalProvider(#[from] ExternalProviderError),
    /// A parent [`AttributeValue`](crate::AttributeValue) was not found for the specified
    /// [`AttributeValueId`](crate::AttributeValue).
    #[error("parent attribute value not found for attribute value: {0}")]
    ParentAttributeValueNotFound(AttributeValueId),
    /// No [`ComponentType`](crate::ComponentType) was found for the appropriate
    /// [`AttributeValue`](crate::AttributeValue) and [`Component`](crate::Component). In other
    /// words, the value contained in the [`AttributeValue`](crate::AttributeValue) was "none".
    #[error("component type is none for component ({0}) and attribute value ({1})")]
    ComponentTypeIsNone(AttributeValueId, ComponentId),
    #[error("component protection is none for component ({0})")]
    ComponentProtectionIsNone(ComponentId),
}

pub type ComponentResult<T> = Result<T, ComponentError>;

const FIND_FOR_NODE: &str = include_str!("queries/component/find_for_node.sql");
const FIND_ATTRIBUTE_VALUE: &str = include_str!("queries/component/find_attribute_value.sql");
const LIST_FOR_SCHEMA_VARIANT: &str = include_str!("queries/component/list_for_schema_variant.sql");
const LIST_SOCKETS_FOR_SOCKET_EDGE_KIND: &str =
    include_str!("queries/component/list_sockets_for_socket_edge_kind.sql");
const FIND_NAME: &str = include_str!("queries/component/find_name.sql");
const ROOT_CHILD_ATTRIBUTE_VALUE_FOR_COMPONENT: &str =
    include_str!("queries/component/root_child_attribute_value_for_component.sql");
const LIST_CONNECTED_INPUT_SOCKETS_FOR_ATTRIBUTE_VALUE: &str =
    include_str!("queries/component/list_connected_input_sockets_for_attribute_value.sql");
const LIST_ALL_RESOURCE_IMPLICIT_INTERNAL_PROVIDER_ATTRIBUTE_VALUES: &str = include_str!(
    "queries/component/list_all_resource_implicit_internal_provider_attribute_values.sql"
);
const COMPONENT_STATUS_UPDATE_BY_PK: &str =
    include_str!("queries/component/status_update_by_pk.sql");

pk!(ComponentPk);
pk!(ComponentId);

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
    Standard,
    Credential,
}

impl Default for ComponentKind {
    fn default() -> Self {
        Self::Standard
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct Component {
    pk: ComponentPk,
    id: ComponentId,
    kind: ComponentKind,
    #[serde(flatten)]
    tenancy: WriteTenancy,
    #[serde(flatten)]
    timestamp: Timestamp,
    #[serde(flatten)]
    visibility: Visibility,
}

impl_standard_model! {
    model: Component,
    pk: ComponentPk,
    id: ComponentId,
    table_name: "components",
    history_event_label_base: "component",
    history_event_message_name: "Component"
}

impl Component {
    /// The primary constructor method for creating [`Components`](Self). It returns a new
    /// [`Component`] with a corresponding [`Node`](crate::Node).
    ///
    /// If you would like to use the default [`SchemaVariant`](crate::SchemaVariant) for
    /// a [`Schema`](crate::Schema) rather than
    /// a specific [`SchemaVariantId`](crate::SchemaVariant), use
    /// [`Self::new_for_default_variant_from_schema()`].
    #[instrument(skip_all)]
    pub async fn new(
        ctx: &DalContext,
        name: impl AsRef<str>,
        schema_variant_id: SchemaVariantId,
    ) -> ComponentResult<(Self, Node)> {
        let schema_variant = SchemaVariant::get_by_id(ctx, &schema_variant_id)
            .await?
            .ok_or(SchemaVariantError::NotFound(schema_variant_id))?;

        // Ensure components are not created unless the variant has been finalized at least once.
        if !schema_variant.finalized_once() {
            return Err(ComponentError::SchemaVariantNotFinalized(schema_variant_id));
        }

        let schema = schema_variant
            .schema(ctx)
            .await?
            .ok_or(SchemaVariantError::MissingSchema(schema_variant_id))?;
        let actor_user_id = match ctx.history_actor() {
            HistoryActor::User(user_id) => Some(*user_id),
            _ => None,
        };

        let row = ctx
            .txns()
            .pg()
            .query_one(
                "SELECT object FROM component_create_v1($1, $2, $3, $4)",
                &[
                    ctx.write_tenancy(),
                    ctx.visibility(),
                    &actor_user_id,
                    &schema.component_kind().as_ref(),
                ],
            )
            .await?;

        let component: Component = standard_model::finish_create_from_row(ctx, row).await?;
        component.set_schema(ctx, schema.id()).await?;
        component
            .set_schema_variant(ctx, &schema_variant_id)
            .await?;

        // Need to flesh out node so that the template data is also included in the node we
        // persist. But it isn't, - our node is anemic.
        let node = Node::new(ctx, &NodeKind::Configuration).await?;
        node.set_component(ctx, component.id()).await?;
        component.set_name(ctx, Some(name.as_ref())).await?;

        // Ensure we have an attribute value and prototype for the resource tree in our exact
        // context. We need this in order to run confirmations upon applying a change set.
        let resource_implicit_internal_provider =
            SchemaVariant::find_root_child_implicit_internal_provider(
                ctx,
                schema_variant_id,
                RootPropChild::Resource,
            )
            .await?;
        let resource_attribute_read_context = AttributeReadContext {
            internal_provider_id: Some(*resource_implicit_internal_provider.id()),
            component_id: Some(*component.id()),
            ..AttributeReadContext::default()
        };
        let resource_attribute_value =
            AttributeValue::find_for_context(ctx, resource_attribute_read_context)
                .await?
                .ok_or(ComponentError::AttributeValueNotFoundForContext(
                    resource_attribute_read_context,
                ))?;
        let resource_attribute_prototype = resource_attribute_value
            .attribute_prototype(ctx)
            .await?
            .ok_or_else(|| {
                ComponentError::MissingAttributePrototype(*resource_attribute_value.id())
            })?;
        AttributePrototype::update_for_context(
            ctx,
            *resource_attribute_prototype.id(),
            AttributeContextBuilder::from(resource_attribute_read_context).to_context()?,
            resource_attribute_prototype.func_id(),
            resource_attribute_value.func_binding_id(),
            resource_attribute_value.func_binding_return_value_id(),
            None,
            None,
        )
        .await?;

        Ok((component, node))
    }

    /// A secondary constructor method that finds the default
    /// [`SchemaVariant`](crate::SchemaVariant) for a given [`SchemaId`](crate::Schema). Once found,
    /// the [`primary constructor method`](Self::new) is called.
    pub async fn new_for_default_variant_from_schema(
        ctx: &DalContext,
        name: impl AsRef<str>,
        schema_id: SchemaId,
    ) -> ComponentResult<(Self, Node)> {
        let schema = Schema::get_by_id(ctx, &schema_id)
            .await?
            .ok_or(SchemaError::NotFound(schema_id))?;

        let schema_variant_id = schema
            .default_schema_variant_id()
            .ok_or(SchemaError::NoDefaultVariant(schema_id))?;

        Self::new(ctx, name, *schema_variant_id).await
    }

    standard_model_accessor!(kind, Enum(ComponentKind), ComponentResult);

    standard_model_belongs_to!(
        lookup_fn: schema,
        set_fn: set_schema,
        unset_fn: unset_schema,
        table: "component_belongs_to_schema",
        model_table: "schemas",
        belongs_to_id: SchemaId,
        returns: Schema,
        result: ComponentResult,
    );

    standard_model_belongs_to!(
        lookup_fn: schema_variant,
        set_fn: set_schema_variant,
        unset_fn: unset_schema_variant,
        table: "component_belongs_to_schema_variant",
        model_table: "schema_variants",
        belongs_to_id: SchemaVariantId,
        returns: SchemaVariant,
        result: ComponentResult,
    );

    standard_model_has_many!(
        lookup_fn: node,
        table: "node_belongs_to_component",
        model_table: "nodes",
        returns: Node,
        result: ComponentResult,
    );

    pub fn tenancy(&self) -> &WriteTenancy {
        &self.tenancy
    }

    /// List [`Sockets`](crate::Socket) with a given
    /// [`SocketEdgeKind`](crate::socket::SocketEdgeKind).
    #[instrument(skip_all)]
    pub async fn list_sockets_for_kind(
        ctx: &DalContext,
        component_id: ComponentId,
        socket_edge_kind: SocketEdgeKind,
    ) -> ComponentResult<Vec<Socket>> {
        let rows = ctx
            .txns()
            .pg()
            .query(
                LIST_SOCKETS_FOR_SOCKET_EDGE_KIND,
                &[
                    ctx.read_tenancy(),
                    ctx.visibility(),
                    &component_id,
                    &(socket_edge_kind.to_string()),
                ],
            )
            .await?;
        Ok(standard_model::objects_from_rows(rows)?)
    }

    /// Find [`Self`] with a provided [`NodeId`](crate::Node).
    #[instrument(skip_all)]
    pub async fn find_for_node(ctx: &DalContext, node_id: NodeId) -> ComponentResult<Option<Self>> {
        let row = ctx
            .txns()
            .pg()
            .query_opt(
                FIND_FOR_NODE,
                &[ctx.read_tenancy(), ctx.visibility(), &node_id],
            )
            .await?;
        Ok(standard_model::object_option_from_row_option(row)?)
    }

    /// Find the [`AttributeValue`](crate::AttributeValue) whose
    /// [`context`](crate::AttributeContext) corresponds to the [`PropId`](crate::Prop)
    /// corresponding to "/root/si/type" and whose [`ComponentId`](Self) matches the provided
    /// [`ComponentId`](Self).
    ///
    /// _Note:_ if the type has never been updated, this will find the _default_
    /// [`AttributeValue`](crate::AttributeValue) where the [`ComponentId`](Self) is unset.
    #[instrument(skip_all)]
    pub async fn find_attribute_value(
        ctx: &DalContext,
        component_id: ComponentId,
        attribute_value_name: String,
    ) -> ComponentResult<AttributeValue> {
        let row = ctx
            .txns()
            .pg()
            .query_one(
                FIND_ATTRIBUTE_VALUE,
                &[
                    ctx.read_tenancy(),
                    ctx.visibility(),
                    &component_id,
                    &attribute_value_name,
                ],
            )
            .await?;
        Ok(object_from_row(row)?)
    }

    #[instrument(skip_all)]
    pub async fn is_in_tenancy(ctx: &DalContext, id: ComponentId) -> ComponentResult<bool> {
        let row = ctx
            .pg_txn()
            .query_opt(
                "SELECT id FROM components WHERE id = $1 AND in_tenancy_v1($2, components.tenancy_billing_account_pks,
                                                                           components.tenancy_organization_pks, components.tenancy_workspace_ids) LIMIT 1",
                &[
                    &id,
                    ctx.read_tenancy(),
                ],
            )
            .await?;
        Ok(row.is_some())
    }

    #[instrument(skip_all)]
    pub async fn list_for_schema_variant(
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
    ) -> ComponentResult<Vec<Component>> {
        let rows = ctx
            .pg_txn()
            .query(
                LIST_FOR_SCHEMA_VARIANT,
                &[ctx.read_tenancy(), ctx.visibility(), &schema_variant_id],
            )
            .await?;

        let mut results = Vec::new();
        for row in rows.into_iter() {
            let json: serde_json::Value = row.try_get("object")?;
            let object: Self = serde_json::from_value(json)?;
            results.push(object);
        }

        Ok(results)
    }

    // Note: Won't work for arrays and maps
    // NOTE(nick): please do not use this for anything other than setting "/root/si/name" in its
    // current state.
    #[instrument(skip_all)]
    pub async fn set_name<T: Serialize + std::fmt::Debug + std::clone::Clone>(
        &self,
        ctx: &DalContext,
        value: Option<T>,
    ) -> ComponentResult<Option<T>> {
        let json_pointer = "/root/si/name";

        let attribute_value = self
            .find_attribute_value_by_json_pointer(ctx, json_pointer)
            .await?
            .ok_or(AttributeValueError::Missing)?;

        let attribute_prototype = attribute_value
            .attribute_prototype(ctx)
            .await?
            .ok_or_else(|| ComponentError::MissingAttributePrototype(*attribute_value.id()))?;

        let prototype_func = Func::get_by_id(ctx, &attribute_prototype.func_id())
            .await?
            .ok_or_else(|| {
                ComponentError::MissingAttributePrototypeFunction(*attribute_prototype.id())
            })?;
        let name = prototype_func.name();
        if name != "si:unset" && name != "si:setString" {
            return Ok(None);
        }

        let attribute_context = AttributeContext::builder()
            .set_component_id(self.id)
            .set_prop_id(attribute_value.context.prop_id())
            .to_context()?;

        let json_value = match value.clone() {
            Some(v) => Some(serde_json::to_value(v)?),
            None => None,
        };

        let mut json_path_parts = json_pointer.split('/').collect::<Vec<&str>>();
        json_path_parts.pop();
        let parent_json_pointer = json_path_parts.join("/");
        let parent_attribute_value_id = self
            .find_attribute_value_by_json_pointer(ctx, &parent_json_pointer)
            .await?
            .map(|av| *av.id());

        let (_, _) = AttributeValue::update_for_context(
            ctx,
            *attribute_value.id(),
            parent_attribute_value_id,
            attribute_context,
            json_value,
            None,
        )
        .await?;

        Ok(value)
    }

    #[instrument(skip_all)]
    pub async fn find_prop_by_json_pointer(
        &self,
        ctx: &DalContext,
        json_pointer: &str,
    ) -> ComponentResult<Option<Prop>> {
        let schema_variant = self
            .schema_variant(ctx)
            .await?
            .ok_or(ComponentError::NoSchemaVariant(self.id))?;

        let mut hierarchy = json_pointer.split('/');
        hierarchy.next(); // Ignores empty part

        let mut next = match hierarchy.next() {
            Some(n) => n,
            None => return Ok(None),
        };

        let mut work_queue = schema_variant.props(ctx).await?;
        while let Some(prop) = work_queue.pop() {
            if prop.name() == next {
                next = match hierarchy.next() {
                    Some(n) => n,
                    None => return Ok(Some(prop)),
                };
                work_queue.clear();
                work_queue.extend(prop.child_props(ctx).await?);
            }
        }

        Ok(None)
    }

    #[instrument(skip_all)]
    pub async fn find_attribute_value_by_json_pointer(
        &self,
        ctx: &DalContext,
        json_pointer: &str,
    ) -> ComponentResult<Option<AttributeValue>> {
        if let Some(prop) = self.find_prop_by_json_pointer(ctx, json_pointer).await? {
            let read_context = AttributeReadContext {
                prop_id: Some(*prop.id()),
                component_id: Some(self.id),
                ..AttributeReadContext::default()
            };

            return Ok(Some(
                AttributeValue::find_for_context(ctx, read_context)
                    .await?
                    .ok_or(AttributeValueError::Missing)?,
            ));
        };

        Ok(None)
    }

    #[instrument(skip_all)]
    pub async fn find_value_by_json_pointer<T: serde::de::DeserializeOwned + std::fmt::Debug>(
        &self,
        ctx: &DalContext,
        json_pointer: &str,
    ) -> ComponentResult<Option<T>> {
        let schema_variant = self
            .schema_variant(ctx)
            .await?
            .ok_or(ComponentError::NoSchemaVariant(self.id))?;
        let prop = Prop::find_root_for_schema_variant(ctx, *schema_variant.id())
            .await?
            .ok_or_else(|| ComponentError::RootPropNotFound(*schema_variant.id()))?;

        let implicit_provider = InternalProvider::find_for_prop(ctx, *prop.id())
            .await?
            .ok_or_else(|| ComponentError::InternalProviderNotFoundForProp(*prop.id()))?;

        let value_context = AttributeReadContext {
            internal_provider_id: Some(*implicit_provider.id()),
            component_id: Some(self.id),
            ..AttributeReadContext::default()
        };

        let attribute_value = AttributeValue::find_for_context(ctx, value_context)
            .await?
            .ok_or(ComponentError::AttributeValueNotFoundForContext(
                value_context,
            ))?;

        let properties =
            FuncBindingReturnValue::get_by_id(ctx, &attribute_value.func_binding_return_value_id())
                .await?
                .ok_or_else(|| {
                    ComponentError::FuncBindingReturnValueNotFound(
                        attribute_value.func_binding_return_value_id(),
                    )
                })?;
        let value = serde_json::json!({ "root": properties.value() })
            .pointer(json_pointer)
            .map(T::deserialize)
            .transpose()?;

        Ok(value)
    }

    /// Return the name of the [`Component`](Self) for the provided [`ComponentId`](Self).
    #[instrument(skip_all)]
    pub async fn find_name(ctx: &DalContext, component_id: ComponentId) -> ComponentResult<String> {
        let row = ctx
            .pg_txn()
            .query_one(
                FIND_NAME,
                &[ctx.read_tenancy(), ctx.visibility(), &component_id],
            )
            .await?;
        let component_name: serde_json::Value = row.try_get("component_name")?;
        let component_name: Option<String> = serde_json::from_value(component_name)?;
        let component_name = component_name.ok_or(ComponentError::NameIsUnset(component_id))?;
        Ok(component_name)
    }

    pub async fn name(&self, ctx: &DalContext) -> ComponentResult<String> {
        Self::find_name(ctx, self.id).await
    }

    /// Grabs the [`AttributeValue`](crate::AttributeValue) corresponding to the
    /// [`RootPropChild`](crate::RootPropChild) [`Prop`](crate::Prop) for the given
    /// [`Component`](Self).
    #[instrument(skip_all)]
    pub async fn root_prop_child_attribute_value_for_component(
        ctx: &DalContext,
        component_id: ComponentId,
        root_prop_child: RootPropChild,
    ) -> ComponentResult<AttributeValue> {
        let row = ctx
            .pg_txn()
            .query_one(
                ROOT_CHILD_ATTRIBUTE_VALUE_FOR_COMPONENT,
                &[
                    ctx.read_tenancy(),
                    ctx.visibility(),
                    &root_prop_child.as_str(),
                    &component_id,
                ],
            )
            .await?;
        Ok(object_from_row(row)?)
    }

    /// List the connected input [`Sockets`](crate::Socket) for a given [`ComponentId`](Self) and
    /// [`AttributeValueId`](crate::AttributeValue) whose [`context`](crate::AttributeContext)'s
    /// least specific field corresponding to a [`PropId`](crate::Prop). In other words, this is
    /// the list of input [`Sockets`](crate::Socket) with incoming connections from other
    /// [`Component(s)`](Self) that the given [`AttributeValue`](crate::AttributeValue) depends on.
    ///
    /// ```raw
    ///                      ┌────────────────────────────┐
    ///                      │ This                       │
    ///                      │ Component                  │
    /// ┌───────────┐        │         ┌────────────────┐ │
    /// │ Another   │        │    ┌───►│ AttributeValue │ │
    /// │ Component │        │    │    │ for Prop       │ │
    /// │           │        │    │    └────────────────┘ │
    /// │  ┌────────┤        ├────┴─────────┐             │
    /// │  │ Output ├───────►│ Input        │             │
    /// │  │ Socket │        │ Socket       │             │
    /// │  │        │        │ (list these) │             │
    /// └──┴────────┘        └──────────────┴─────────────┘
    /// ```
    ///
    /// _Warning: users of this query must ensure that the
    /// [`AttributeValueId`](crate::AttributeValue) provided has a
    /// [`context`](crate::AttributeContext) whose least specific field corresponds to a
    /// [`PropId`](crate::Prop)._
    #[instrument(skip_all)]
    pub async fn list_connected_input_sockets_for_attribute_value(
        ctx: &DalContext,
        attribute_value_id: AttributeValueId,
        component_id: ComponentId,
    ) -> ComponentResult<Vec<Socket>> {
        let rows = ctx
            .pg_txn()
            .query(
                LIST_CONNECTED_INPUT_SOCKETS_FOR_ATTRIBUTE_VALUE,
                &[
                    ctx.read_tenancy(),
                    ctx.visibility(),
                    &attribute_value_id,
                    &component_id,
                ],
            )
            .await?;
        Ok(standard_model::objects_from_rows(rows)?)
    }

    pub async fn schema_variant_id(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> ComponentResult<SchemaVariantId> {
        let row = ctx
            .pg_txn()
            .query_one(
                "select belongs_to_id as schema_variant_id from 
                    component_belongs_to_schema_variant_v1($1, $2)
                    where object_id = $3
                ",
                &[ctx.read_tenancy(), ctx.visibility(), &component_id],
            )
            .await?;

        Ok(row.try_get("schema_variant_id")?)
    }

    /// Gets the [`ComponentType`](crate::ComponentType) of [`self`](Self).
    ///
    /// Mutate this with [`Self::set_type()`].
    pub async fn get_type(&self, ctx: &DalContext) -> ComponentResult<ComponentType> {
        let type_attribute_value =
            Self::find_attribute_value(ctx, self.id, "type".to_string()).await?;
        let raw_value = type_attribute_value.get_value(ctx).await?.ok_or_else(|| {
            ComponentError::ComponentTypeIsNone(*type_attribute_value.id(), self.id)
        })?;
        let component_type: ComponentType = serde_json::from_value(raw_value)?;
        Ok(component_type)
    }

    /// Gets the protected attribute value of [`self`](Self).
    pub async fn get_protected(&self, ctx: &DalContext) -> ComponentResult<bool> {
        let protected_attribute_value =
            Self::find_attribute_value(ctx, self.id, "protected".to_string()).await?;
        let raw_value = protected_attribute_value
            .get_value(ctx)
            .await?
            .ok_or(ComponentError::ComponentProtectionIsNone(self.id))?;
        let protected: bool = serde_json::from_value(raw_value)?;
        Ok(protected)
    }

    /// Sets the field corresponding to "/root/si/type" for the [`Component`]. Possible values
    /// are limited to variants of [`ComponentType`](crate::ComponentType).
    pub async fn set_type(
        &self,
        ctx: &DalContext,
        component_type: ComponentType,
    ) -> ComponentResult<()> {
        let type_attribute_value =
            Self::find_attribute_value(ctx, self.id, "type".to_string()).await?;

        // If we are setting the type for the first time, we will need to mutate the context to
        // be component-specific. This is because the attribute value will have an unset component
        // id and we will need to deviate from the schema variant default component type.
        let attribute_context = if type_attribute_value.context.is_component_unset() {
            AttributeContextBuilder::from(type_attribute_value.context)
                .set_component_id(self.id)
                .to_context()?
        } else {
            type_attribute_value.context
        };

        let si_attribute_value = type_attribute_value
            .parent_attribute_value(ctx)
            .await?
            .ok_or_else(|| {
                ComponentError::ParentAttributeValueNotFound(*type_attribute_value.id())
            })?;
        AttributeValue::update_for_context(
            ctx,
            *type_attribute_value.id(),
            Some(*si_attribute_value.id()),
            attribute_context,
            Some(serde_json::to_value(component_type)?),
            None,
        )
        .await?;

        // Now that we've updated the field, we need to see if we need to do additional work.
        let schema_variant = self
            .schema_variant(ctx)
            .await?
            .ok_or(ComponentError::NoSchemaVariant(self.id))?;
        let external_providers =
            ExternalProvider::list_for_schema_variant(ctx, *schema_variant.id()).await?;
        let internal_providers =
            InternalProvider::list_explicit_for_schema_variant(ctx, *schema_variant.id()).await?;

        // We have some work to do for all component types, but the aggregation frames need a
        // special look.
        if let ComponentType::AggregationFrame = component_type {
            let (func, func_binding, func_binding_return_value) =
                Func::identity_with_binding_and_return_value(ctx).await?;
            let func_id = *func.id();

            for external_provider in external_providers {
                let attribute_read_context = AttributeReadContext {
                    prop_id: Some(PropId::NONE),
                    internal_provider_id: Some(InternalProviderId::NONE),
                    external_provider_id: Some(*external_provider.id()),
                    component_id: Some(self.id),
                };

                let attribute_context =
                    AttributeContextBuilder::from(attribute_read_context).to_context()?;

                let attribute_value = AttributeValue::find_for_context(ctx, attribute_read_context)
                    .await?
                    .ok_or(ComponentError::AttributeValueNotFoundForContext(
                        attribute_read_context,
                    ))?;

                if attribute_value.context.is_component_unset() {
                    AttributePrototype::new(
                        ctx,
                        func_id,
                        *func_binding.id(),
                        *func_binding_return_value.id(),
                        attribute_context,
                        None,
                        None,
                    )
                    .await?;
                } else {
                    AttributePrototype::new_with_existing_value(
                        ctx,
                        func_id,
                        attribute_context,
                        None,
                        None,
                        *attribute_value.id(),
                    )
                    .await?;
                };
            }

            for internal_provider in internal_providers {
                let attribute_read_context = AttributeReadContext {
                    prop_id: Some(PropId::NONE),
                    internal_provider_id: Some(*internal_provider.id()),
                    external_provider_id: Some(ExternalProviderId::NONE),
                    component_id: Some(self.id),
                };

                let attr_write_context =
                    AttributeContextBuilder::from(attribute_read_context).to_context()?;

                let attribute_value = AttributeValue::find_for_context(ctx, attribute_read_context)
                    .await?
                    .ok_or(ComponentError::AttributeValueNotFoundForContext(
                        attribute_read_context,
                    ))?;

                let prototype =
                    attribute_value
                        .attribute_prototype(ctx)
                        .await?
                        .ok_or_else(|| {
                            ComponentError::MissingAttributePrototype(*attribute_value.id())
                        })?;

                let arguments = AttributePrototypeArgument::find_by_attr(
                    ctx,
                    "attribute_prototype_id",
                    prototype.id(),
                )
                .await?;

                let new_prototype = if attribute_value.context.is_component_unset() {
                    AttributePrototype::new(
                        ctx,
                        func_id,
                        *func_binding.id(),
                        *func_binding_return_value.id(),
                        attr_write_context,
                        None,
                        None,
                    )
                    .await?
                } else {
                    AttributePrototype::new_with_existing_value(
                        ctx,
                        func_id,
                        attr_write_context,
                        None,
                        None,
                        *attribute_value.id(),
                    )
                    .await?
                };

                for argument in arguments {
                    AttributePrototypeArgument::new_for_inter_component(
                        ctx,
                        *new_prototype.id(),
                        argument.func_argument_id(),
                        argument.head_component_id(),
                        argument.tail_component_id(),
                        argument.external_provider_id(),
                    )
                    .await?;
                }
            }
        } else {
            for external_provider in external_providers {
                let attribute_read_context = AttributeReadContext {
                    prop_id: Some(PropId::NONE),
                    internal_provider_id: Some(InternalProviderId::NONE),
                    external_provider_id: Some(*external_provider.id()),
                    component_id: Some(self.id),
                };

                let mut attribute_value =
                    AttributeValue::find_for_context(ctx, attribute_read_context)
                        .await?
                        .ok_or(ComponentError::AttributeValueNotFoundForContext(
                            attribute_read_context,
                        ))?;

                if !attribute_value.context.is_component_unset() {
                    attribute_value.unset_attribute_prototype(ctx).await?;
                    attribute_value.delete_by_id(ctx).await?;
                }
            }

            for internal_provider in internal_providers {
                let attribute_read_context = AttributeReadContext {
                    prop_id: Some(PropId::NONE),
                    internal_provider_id: Some(*internal_provider.id()),
                    external_provider_id: Some(ExternalProviderId::NONE),
                    component_id: Some(self.id),
                };

                let mut attribute_value =
                    AttributeValue::find_for_context(ctx, attribute_read_context)
                        .await?
                        .ok_or(ComponentError::AttributeValueNotFoundForContext(
                            attribute_read_context,
                        ))?;

                if !attribute_value.context.is_component_unset() {
                    attribute_value.unset_attribute_prototype(ctx).await?;
                    attribute_value.delete_by_id(ctx).await?;
                }
            }
        }

        Ok(())
    }

    pub async fn delete_and_propagate(&mut self, ctx: &DalContext) -> ComponentResult<()> {
        // TODO - This is temporary for now until we allow deleting frames
        if self.get_type(ctx).await? != ComponentType::Component {
            return Err(ComponentError::Frame);
        }

        if self.get_protected(ctx).await? {
            return Err(ComponentError::ComponentProtected(self.id));
        }

        let edges = Edge::list_for_component(ctx, self.id).await?;
        for mut edge in edges {
            edge.delete_and_propagate(ctx).await?;
        }

        for mut node in self.node(ctx).await? {
            node.delete_by_id(ctx).await?;
        }

        self.delete_by_id(ctx).await?;

        Ok(())
    }

    pub async fn restore_by_id(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> ComponentResult<Self> {
        let ctx_with_deleted = &ctx.clone_with_delete_visibility();

        for edge in Edge::list_for_component(ctx_with_deleted, component_id).await? {
            Edge::restore_by_id(ctx, *edge.id()).await?;
        }

        // TODO: When components that don't exist on HEAD but got deleted on the changeset try to
        // be restored, get_by_id does not find them, even with a "valid" visibility. we should discuss this
        let comp = Component::get_by_id(ctx_with_deleted, &component_id)
            .await?
            .ok_or(ComponentError::NotFound(component_id))?;

        for node in comp.node(ctx_with_deleted).await? {
            node.hard_delete(ctx_with_deleted).await?;
        }

        comp.hard_delete(ctx_with_deleted).await?;

        Component::get_by_id(ctx, &component_id)
            .await?
            .ok_or(ComponentError::NotFound(component_id))
    }
}
