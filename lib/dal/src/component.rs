//! This module contains [`Component`], which is an instance of a
//! [`SchemaVariant`](crate::SchemaVariant) and a _model_ of a "real world resource".

use serde::{Deserialize, Serialize};
use serde_json::Value;
use si_data_nats::NatsError;
use si_data_pg::PgError;
use std::collections::HashMap;
use strum_macros::{AsRefStr, Display, EnumIter, EnumString};
use telemetry::prelude::*;
use thiserror::Error;

use crate::attribute::value::AttributeValue;
use crate::attribute::value::AttributeValueError;
use crate::code_view::CodeViewError;
use crate::func::backend::{
    js_validation::FuncBackendJsValidationArgs, validation::FuncBackendValidationArgs,
};
use crate::func::binding::{FuncBinding, FuncBindingError};
use crate::func::binding_return_value::{
    FuncBindingReturnValue, FuncBindingReturnValueError, FuncBindingReturnValueId,
};

use crate::schema::variant::{SchemaVariantError, SchemaVariantId};
use crate::schema::SchemaVariant;
use crate::socket::SocketEdgeKind;
use crate::standard_model::object_from_row;
use crate::validation::ValidationConstructorError;
use crate::ws_event::WsEventError;
use crate::{
    func::FuncId, impl_standard_model, node::NodeId, pk, provider::internal::InternalProviderError,
    standard_model, standard_model_accessor, standard_model_belongs_to, standard_model_has_many,
    ActionPrototypeError, AttributeContext, AttributeContextBuilderError, AttributeContextError,
    AttributePrototypeId, AttributeReadContext, CodeLanguage, DalContext, EdgeError,
    ExternalProviderId, Func, FuncBackendKind, HistoryEventError, InternalProvider,
    InternalProviderId, Node, NodeError, OrganizationError, Prop, PropError, PropId,
    ReadTenancyError, Schema, SchemaError, SchemaId, Socket, StandardModel, StandardModelError,
    Timestamp, TransactionsError, ValidationPrototype, ValidationPrototypeError,
    ValidationResolver, ValidationResolverError, Visibility, WorkflowRunnerError, WorkspaceError,
    WriteTenancy,
};
use crate::{AttributeValueId, QualificationError};

pub use view::{ComponentView, ComponentViewError};

pub mod code;
pub mod diff;
pub mod qualification;
pub mod resource;
pub mod stats;
pub mod view;

#[derive(Error, Debug)]
pub enum ComponentError {
    #[error("attribute context error: {0}")]
    AttributeContext(#[from] AttributeContextError),
    #[error("attribute context builder error: {0}")]
    AttributeContextBuilder(#[from] AttributeContextBuilderError),
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

    // FIXME: change the below to be alphabetical and re-join with the above variants.
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
}

pub type ComponentResult<T> = Result<T, ComponentError>;

const FIND_FOR_NODE: &str = include_str!("./queries/component_find_for_node.sql");
//const GET_RESOURCE: &str = include_str!("./queries/component_get_resource.sql");
const LIST_FOR_SCHEMA_VARIANT: &str =
    include_str!("./queries/component_list_for_schema_variant.sql");
const LIST_SOCKETS_FOR_SOCKET_EDGE_KIND: &str =
    include_str!("queries/component_list_sockets_for_socket_edge_kind.sql");
const NAME_FROM_CONTEXT: &str = include_str!("./queries/component/name_from_context.sql");
const ROOT_CHILD_ATTRIBUTE_VALUE_FOR_COMPONENT: &str =
    include_str!("queries/component/root_child_attribute_value_for_component.sql");
const LIST_CONNECTED_INPUT_SOCKETS_FOR_ATTRIBUTE_VALUE: &str =
    include_str!("queries/component/list_connected_input_sockets_for_attribute_value.sql");

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
    #[instrument(skip_all)]
    pub async fn new(
        ctx: &DalContext,
        name: impl AsRef<str>,
        schema_variant_id: SchemaVariantId,
    ) -> ComponentResult<(Self, Node)> {
        let schema_variant = SchemaVariant::get_by_id(ctx, &schema_variant_id)
            .await?
            .ok_or(SchemaVariantError::NotFound(schema_variant_id))?;

        // NOTE(nick): we may want to replace this with a better solution. We use this temporarily to
        // ensure components are not created unless the variant has been finalized at least once.
        if !schema_variant.finalized_once() {
            return Err(ComponentError::SchemaVariantNotFinalized(schema_variant_id));
        }

        let schema = schema_variant
            .schema(ctx)
            .await?
            .ok_or(SchemaVariantError::MissingSchema(schema_variant_id))?;

        let row = ctx
            .txns()
            .pg()
            .query_one(
                "SELECT object FROM component_create_v1($1, $2, $3)",
                &[
                    ctx.write_tenancy(),
                    ctx.visibility(),
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
        let node = Node::new(ctx, &(*schema.kind()).into()).await?;
        node.set_component(ctx, component.id()).await?;
        component.set_name(ctx, Some(name.as_ref())).await?;

        Ok((component, node))
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

    pub async fn check_single_validation(
        &self,
        ctx: &DalContext,
        validation_prototype: &ValidationPrototype,
        value_cache: &mut HashMap<PropId, (Option<Value>, AttributeValue)>,
    ) -> ComponentResult<()> {
        let base_attribute_read_context = AttributeReadContext {
            prop_id: None,
            external_provider_id: Some(ExternalProviderId::NONE),
            internal_provider_id: Some(InternalProviderId::NONE),
            component_id: Some(self.id),
        };

        let prop_id = validation_prototype.context().prop_id();

        let (maybe_value, attribute_value) = match value_cache.get(&prop_id) {
            Some((value, attribute_value)) => (value.to_owned(), attribute_value.clone()),
            None => {
                let attribute_read_context = AttributeReadContext {
                    prop_id: Some(prop_id),
                    ..base_attribute_read_context
                };
                let attribute_value = AttributeValue::find_for_context(ctx, attribute_read_context)
                    .await?
                    .ok_or(ComponentError::AttributeValueNotFoundForContext(
                        attribute_read_context,
                    ))?;

                let value = match FuncBindingReturnValue::get_by_id(
                    ctx,
                    &attribute_value.func_binding_return_value_id(),
                )
                .await?
                {
                    Some(func_binding_return_value) => func_binding_return_value.value().cloned(),
                    None => None,
                };

                value_cache.insert(prop_id, (value.clone(), attribute_value.clone()));
                (value, attribute_value)
            }
        };

        let func = Func::get_by_id(ctx, &validation_prototype.func_id())
            .await?
            .ok_or_else(|| PropError::MissingFuncById(validation_prototype.func_id()))?;

        let mutated_args = match func.backend_kind() {
            FuncBackendKind::Validation => {
                // Deserialize the args, update the "value", and serialize the mutated args.
                let mut args = FuncBackendValidationArgs::deserialize(validation_prototype.args())?;
                args.validation = args.validation.update_value(&maybe_value)?;

                serde_json::to_value(args)?
            }
            FuncBackendKind::JsValidation => serde_json::to_value(FuncBackendJsValidationArgs {
                value: maybe_value.unwrap_or(serde_json::json!(null)),
            })?,
            kind => {
                return Err(ComponentError::InvalidFuncBackendKindForValidations(*kind));
            }
        };

        // Now, we can load in the mutated args!
        let (func_binding, _) =
            FuncBinding::create_and_execute(ctx, mutated_args, *func.id()).await?;

        let attribute_value_id = *attribute_value.id();

        // Does a resolver already exist for this validation func and attribute value? If so, we
        // need to make sure the attribute_value_func_binding_return_value_id matches the
        // func_binding_return_value_id of the current attribute value, since it could be different
        // *even if the value is the same*. We also need to be sure to create a resolver for each
        // attribute_value_id, since the way func_bindings are cached means the validation func
        // won't be created for the same validation func + value, despite running this on a
        // completely different attribute value (or even prop).
        match ValidationResolver::find_for_attribute_value_and_validation_func(
            ctx,
            attribute_value_id,
            *func.id(),
        )
        .await?
        .pop()
        {
            Some(mut existing_resolver) => {
                existing_resolver
                    .set_validation_func_binding_id(ctx, func_binding.id())
                    .await?;
                existing_resolver
                    .set_attribute_value_func_binding_return_value_id(
                        ctx,
                        attribute_value.func_binding_return_value_id(),
                    )
                    .await?;
            }
            None => {
                ValidationResolver::new(
                    ctx,
                    *validation_prototype.id(),
                    attribute_value_id,
                    *func_binding.id(),
                )
                .await?;
            }
        }

        Ok(())
    }

    /// Check validations for [`Self`].
    pub async fn check_validations(&self, ctx: &DalContext) -> ComponentResult<()> {
        let schema_variant = self
            .schema_variant(ctx)
            .await?
            .ok_or(ComponentError::NoSchemaVariant(self.id))?;

        let validation_prototypes =
            ValidationPrototype::list_for_schema_variant(ctx, *schema_variant.id()).await?;

        // Cache data necessary for assembling func arguments. We do this since a prop can have
        // multiple validation prototypes within schema variant.
        let mut cache: HashMap<PropId, (Option<Value>, AttributeValue)> = HashMap::new();

        for validation_prototype in validation_prototypes {
            self.check_single_validation(ctx, &validation_prototype, &mut cache)
                .await?;
        }

        Ok(())
    }

    #[instrument(skip_all)]
    pub async fn is_in_tenancy(ctx: &DalContext, id: ComponentId) -> ComponentResult<bool> {
        let row = ctx
            .pg_txn()
            .query_opt(
                "SELECT id FROM components WHERE id = $1 AND in_tenancy_v1($2, components.tenancy_universal, components.tenancy_billing_account_ids,
                                                                           components.tenancy_organization_ids, components.tenancy_workspace_ids) LIMIT 1",
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

    /// Return the name of the [`Component`][Self] that corresponds to the provided [`AttributeReadContext`][AttributeReadContext].
    ///
    /// While this takes a full [`AttributeReadContext`][AttributeReadContext], it is only concerned with the [`ComponentId`][ComponentId],
    /// and anything more specific than that.
    #[instrument(skip_all)]
    pub async fn name_from_context(
        ctx: &DalContext,
        attribute_read_context: AttributeReadContext,
    ) -> ComponentResult<String> {
        let component_id = attribute_read_context.component_id().ok_or_else(|| {
            ComponentError::BadAttributeReadContext(
                "Must specify ComponentId to retrieve the name of a Component".to_string(),
            )
        })?;
        let row = ctx
            .pg_txn()
            .query_one(
                NAME_FROM_CONTEXT,
                &[ctx.read_tenancy(), ctx.visibility(), &component_id],
            )
            .await?;
        let value: serde_json::Value = row.try_get("component_name")?;
        let component_name: String = serde_json::from_value(value)?;

        Ok(component_name)
    }

    pub async fn name(&self, ctx: &DalContext) -> ComponentResult<String> {
        let context = AttributeReadContext {
            component_id: Some(self.id),
            ..AttributeReadContext::any()
        };
        Self::name_from_context(ctx, context).await
    }

    /// Grabs the [`AttributeValue`](crate::AttributeValue) corresponding to the `/root/<child>`
    /// [`Prop`](crate::Prop) for the given [`Component`](Self) (e.g. "/root/resource" and
    /// "/root/code").
    #[instrument(skip_all)]
    pub async fn root_child_attribute_value_for_component(
        ctx: &DalContext,
        root_child_prop_name: impl AsRef<str>,
        component_id: ComponentId,
    ) -> ComponentResult<AttributeValue> {
        let root_child_prop_name = root_child_prop_name.as_ref();
        let row = ctx
            .pg_txn()
            .query_one(
                ROOT_CHILD_ATTRIBUTE_VALUE_FOR_COMPONENT,
                &[
                    ctx.read_tenancy(),
                    ctx.visibility(),
                    &root_child_prop_name,
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
}
