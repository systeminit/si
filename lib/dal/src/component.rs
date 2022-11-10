use serde::{Deserialize, Serialize};
use serde_json::Value;
use si_data_nats::NatsError;
use si_data_pg::PgError;
use std::collections::HashMap;
use strum_macros::{AsRefStr, Display, EnumIter, EnumString};
use telemetry::prelude::*;
use thiserror::Error;

use crate::attribute::context::AttributeContextBuilder;
use crate::attribute::value::AttributeValue;
use crate::attribute::{context::UNSET_ID_VALUE, value::AttributeValueError};
use crate::func::backend::{
    js_qualification::FuncBackendJsQualificationArgs, js_validation::FuncBackendJsValidationArgs,
    validation::FuncBackendValidationArgs,
};
use crate::func::binding::{FuncBinding, FuncBindingError};
use crate::func::binding_return_value::{
    FuncBindingReturnValue, FuncBindingReturnValueError, FuncBindingReturnValueId,
};
use crate::qualification::QualificationView;
use crate::qualification_resolver::QualificationResolverContext;
use crate::schema::variant::{SchemaVariantError, SchemaVariantId};
use crate::schema::SchemaVariant;
use crate::socket::SocketEdgeKind;
use crate::standard_model::object_from_row;
use crate::validation::ValidationConstructorError;
use crate::ws_event::{WsEvent, WsEventError};
use crate::{
    edge::EdgeId,
    func::{FuncId, FuncMetadataView},
    impl_standard_model,
    node::NodeId,
    pk,
    provider::internal::InternalProviderError,
    qualification::QualificationError,
    standard_model, standard_model_accessor, standard_model_belongs_to, standard_model_has_many,
    ActionPrototype, ActionPrototypeError, AttributeContext, AttributeContextBuilderError,
    AttributeContextError, AttributeReadContext, CodeGenerationPrototypeError, CodeLanguage,
    DalContext, Edge, EdgeError, ExternalProviderId, Func, FuncBackendKind, HistoryEventError,
    InternalProvider, InternalProviderId, Node, NodeError, OrganizationError, Prop, PropError,
    PropId, QualificationPrototype, QualificationPrototypeError, QualificationResolver,
    QualificationResolverError, ReadTenancyError, Schema, SchemaError, SchemaId, Socket, SocketId,
    StandardModel, StandardModelError, SystemId, Timestamp, TransactionsError, ValidationPrototype,
    ValidationPrototypeError, ValidationResolver, ValidationResolverError, Visibility,
    WorkflowPrototype, WorkflowPrototypeId, WorkflowRunner, WorkflowRunnerError, WorkspaceError,
    WriteTenancy, WsPayload,
};
use crate::{AttributeValueId, QualificationPrototypeId};

pub use view::{ComponentView, ComponentViewError};

pub mod code;
pub mod diff;
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
    #[error("workflow prototype {0} not found")]
    WorkflowPrototypeNotFound(WorkflowPrototypeId),
    #[error("validation error: {0}")]
    Validation(#[from] ValidationConstructorError),

    // FIXME: change the below to be alphabetical and re-join with the above variants.
    #[error(transparent)]
    ComponentView(#[from] ComponentViewError),
    #[error("qualification prototype error: {0}")]
    QualificationPrototype(#[from] QualificationPrototypeError),
    #[error("qualification resolver error: {0}")]
    QualificationResolver(#[from] QualificationResolverError),
    #[error("code generation prototype error: {0}")]
    CodeGenerationPrototype(#[from] CodeGenerationPrototypeError),
    #[error("unable to find code generated")]
    CodeGeneratedNotFound,
    #[error("qualification prototype not found")]
    QualificationPrototypeNotFound,
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
    #[error("no system sockets found for component: {0}")]
    NoSystemSocketsFound(ComponentId),
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
    #[error("unable to find system")]
    SystemNotFound,
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
    #[error("qualification view error: {0}")]
    QualificationView(#[from] QualificationError),
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
    #[error("need exactly one system socket for component ({0}), found: {1:?}")]
    NeedOneSystemSocket(ComponentId, Vec<SocketId>),
}

pub type ComponentResult<T> = Result<T, ComponentError>;

const FIND_FOR_NODE: &str = include_str!("./queries/component_find_for_node.sql");
//const GET_RESOURCE: &str = include_str!("./queries/component_get_resource.sql");
const LIST_QUALIFICATIONS: &str = include_str!("./queries/component_list_qualifications.sql");
const LIST_FOR_SCHEMA_VARIANT: &str =
    include_str!("./queries/component_list_for_schema_variant.sql");
const LIST_SOCKETS_FOR_SOCKET_EDGE_KIND: &str =
    include_str!("queries/component_list_sockets_for_socket_edge_kind.sql");
const NAME_FROM_CONTEXT: &str = include_str!("./queries/component/name_from_context.sql");
const ROOT_CHILD_ATTRIBUTE_VALUE_FOR_COMPONENT: &str =
    include_str!("queries/component/root_child_attribute_value_for_component.sql");

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
    pub async fn new_for_schema_with_node(
        ctx: &DalContext,
        name: impl AsRef<str>,
        schema_id: &SchemaId,
    ) -> ComponentResult<(Self, Node)> {
        let schema = Schema::get_by_id(ctx, schema_id)
            .await?
            .ok_or(SchemaError::NotFound(*schema_id))?;

        let schema_variant_id = schema
            .default_schema_variant_id()
            .ok_or(SchemaError::NoDefaultVariant(*schema_id))?;

        Self::new_for_schema_variant_with_node(ctx, name, schema_variant_id).await
    }

    #[instrument(skip_all)]
    pub async fn new_for_schema_variant_with_node(
        ctx: &DalContext,
        name: impl AsRef<str>,
        schema_variant_id: &SchemaVariantId,
    ) -> ComponentResult<(Self, Node)> {
        let schema_variant = SchemaVariant::get_by_id(ctx, schema_variant_id)
            .await?
            .ok_or(SchemaVariantError::NotFound(*schema_variant_id))?;
        let schema = schema_variant
            .schema(ctx)
            .await?
            .ok_or(SchemaVariantError::MissingSchema(*schema_variant_id))?;

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
            .set_schema_variant(ctx, schema_variant.id())
            .await?;

        // Need to flesh out node so that the template data is also included in the node we
        // persist. But it isn't, - our node is anemic.
        let node = Node::new(ctx, &(*schema.kind()).into()).await?;
        node.set_component(ctx, component.id()).await?;
        component
            .set_value_by_json_pointer(ctx, "/root/si/name", Some(name.as_ref()))
            .await?;

        Ok((component, node))
    }

    #[instrument(skip_all)]
    pub async fn add_to_system(
        &self,
        ctx: &DalContext,
        system_id: SystemId,
    ) -> ComponentResult<EdgeId> {
        let system_sockets =
            Self::list_sockets_for_kind(ctx, self.id, SocketEdgeKind::System).await?;

        if system_sockets.len() != 1 {
            let system_socket_ids: Vec<SocketId> = system_sockets.iter().map(|s| *s.id()).collect();
            return Err(ComponentError::NeedOneSystemSocket(
                self.id,
                system_socket_ids,
            ));
        }

        let schema = self
            .schema(ctx)
            .await?
            .ok_or(ComponentError::NoSchema(self.id))?;
        let diagram_kind = schema
            .diagram_kind()
            .ok_or_else(|| SchemaError::NoDiagramKindForSchemaKind(*schema.kind()))?;

        let edge = Edge::include_component_in_system(ctx, self.id, diagram_kind, system_id).await?;

        Ok(*edge.id())
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

    /// Check validations for [`Self`].
    pub async fn check_validations(&self, ctx: &DalContext) -> ComponentResult<()> {
        let schema_variant = self
            .schema_variant(ctx)
            .await?
            .ok_or(ComponentError::NoSchemaVariant(self.id))?;
        let schema = self
            .schema(ctx)
            .await?
            .ok_or(ComponentError::NoSchema(self.id))?;

        let base_attribute_read_context = AttributeReadContext {
            prop_id: None,
            external_provider_id: Some(ExternalProviderId::NONE),
            internal_provider_id: Some(InternalProviderId::NONE),
            schema_id: Some(*schema.id()),
            schema_variant_id: Some(*schema_variant.id()),
            component_id: Some(self.id),
            system_id: Some(SystemId::NONE),
        };

        let validation_prototypes =
            ValidationPrototype::list_for_schema_variant(ctx, *schema_variant.id(), SystemId::NONE)
                .await?;

        // Cache data necessary for assembling func arguments. We do this since a prop can have
        // multiple validation prototypes within schema variant.
        let mut cache: HashMap<PropId, (Option<Value>, AttributeValue)> = HashMap::new();

        for validation_prototype in validation_prototypes {
            let prop_id = validation_prototype.context().prop_id();

            // Grab the data necessary for assembling the func arguments. We'll check if it's in
            // the cache first.
            let (maybe_value, attribute_value) = match cache.get(&prop_id) {
                Some((value, attribute_value)) => (value.to_owned(), attribute_value.clone()),
                None => {
                    let attribute_read_context = AttributeReadContext {
                        prop_id: Some(prop_id),
                        ..base_attribute_read_context
                    };
                    let attribute_value =
                        AttributeValue::find_for_context(ctx, attribute_read_context)
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
                        Some(func_binding_return_value) => {
                            func_binding_return_value.value().cloned()
                        }
                        None => None,
                    };

                    cache.insert(prop_id, (value.clone(), attribute_value.clone()));
                    (value, attribute_value)
                }
            };

            let func = Func::get_by_id(ctx, &validation_prototype.func_id())
                .await?
                .ok_or_else(|| PropError::MissingFuncById(validation_prototype.func_id()))?;

            let mutated_args = match func.backend_kind() {
                FuncBackendKind::Validation => {
                    // Deserialize the args, update the "value", and serialize the mutated args.
                    let mut args =
                        FuncBackendValidationArgs::deserialize(validation_prototype.args())?;
                    args.validation = args.validation.update_value(&maybe_value)?;

                    serde_json::to_value(args)?
                }
                FuncBackendKind::JsValidation => {
                    serde_json::to_value(FuncBackendJsValidationArgs {
                        value: maybe_value.unwrap_or(serde_json::json!(null)),
                    })?
                }
                kind => {
                    return Err(ComponentError::InvalidFuncBackendKindForValidations(*kind));
                }
            };

            // Now, we can load in the mutated args!
            let (func_binding, _, _) =
                FuncBinding::find_or_create_and_execute(ctx, mutated_args, *func.id()).await?;

            let attribute_value_id = *attribute_value.id();

            // Does a resolver already exist for this validation? If so, we need to make
            // sure the func_binding_return_value_id matches the func_binding_return_value_id of
            // the current attribute value, since it could be different *even if the value is
            // the same* (it could have been set by a different function that returned the
            // same value, for example). We also need to create a resolver for each
            // attribute_value_id, since the way func_bindings are cached means the validation
            // func won't be created for the same validation func + value, despite running
            // this on a completely different attribute value (or even prop).
            match ValidationResolver::find_for_attribute_value_and_validation_func_binding(
                ctx,
                &attribute_value_id,
                func_binding.id(),
            )
            .await?
            .pop()
            {
                Some(mut existing_resolver) => {
                    existing_resolver
                        .set_func_binding_return_value_id(
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
        }

        Ok(())
    }

    /// Creates a qualification [`FuncBinding`](crate::FuncBinding), a
    /// [`FuncBindingReturnValue`](crate::FuncBindingReturnValue) without a value and a
    /// [`QualificationResolver`](crate::QualificationResolver). The func is not executed yet; it's
    /// just a placeholder for some qualification that will be executed.
    pub async fn prepare_qualification_check(
        &self,
        ctx: &DalContext,
        system_id: SystemId,
        qualification_prototype_id: QualificationPrototypeId,
    ) -> ComponentResult<()> {
        let prototype = QualificationPrototype::get_by_id(ctx, &qualification_prototype_id)
            .await?
            .ok_or(ComponentError::QualificationPrototypeNotFound)?;

        let func = Func::get_by_id(ctx, &prototype.func_id())
            .await?
            .ok_or_else(|| ComponentError::MissingFunc(prototype.func_id().to_string()))?;

        let args = FuncBackendJsQualificationArgs {
            component: self
                .veritech_qualification_check_component(ctx, system_id)
                .await?,
        };

        let json_args = serde_json::to_value(args)?;
        let (func_binding, _created) =
            FuncBinding::find_or_create(ctx, json_args, prototype.func_id(), *func.backend_kind())
                .await?;

        // Empty func binding return value means the function is still being executed
        let _func_binding_return_value = FuncBindingReturnValue::upsert(
            ctx,
            None,
            None,
            prototype.func_id(),
            *func_binding.id(),
            UNSET_ID_VALUE.into(),
        )
        .await?;

        let mut existing_resolvers =
            QualificationResolver::find_for_prototype_and_component(ctx, prototype.id(), self.id())
                .await?;

        // If we do not have one, create the qualification resolver. If we do, update the
        // func binding id to point to the new value.
        if let Some(mut resolver) = existing_resolvers.pop() {
            resolver
                .set_func_binding_id(ctx, *func_binding.id())
                .await?;
        } else {
            let mut resolver_context = QualificationResolverContext::new();
            resolver_context.set_component_id(self.id);
            QualificationResolver::new(
                ctx,
                *prototype.id(),
                *func.id(),
                *func_binding.id(),
                resolver_context,
            )
            .await?;
        }

        WsEvent::checked_qualifications(ctx, *prototype.id(), self.id, system_id)
            .publish(ctx)
            .await?;

        Ok(())
    }

    /// Creates a qualification [`FuncBinding`](crate::FuncBinding), a
    /// [`FuncBindingReturnValue`](crate::FuncBindingReturnValue) without a value and a
    /// [`QualificationResolver`](crate::QualificationResolver). The func is not executed yet; it's
    /// just a placeholder for some qualification that will be executed.
    pub async fn prepare_qualifications_check(
        &self,
        ctx: &DalContext,
        system_id: SystemId,
    ) -> ComponentResult<()> {
        let schema = self
            .schema(ctx)
            .await?
            .ok_or(ComponentError::NoSchema(self.id))?;
        let schema_variant = self
            .schema_variant(ctx)
            .await?
            .ok_or(ComponentError::NoSchemaVariant(self.id))?;

        let qualification_prototypes = QualificationPrototype::find_for_component(
            ctx,
            self.id,
            *schema.id(),
            *schema_variant.id(),
            system_id,
        )
        .await?;

        for prototype in qualification_prototypes {
            let func = Func::get_by_id(ctx, &prototype.func_id())
                .await?
                .ok_or_else(|| ComponentError::MissingFunc(prototype.func_id().to_string()))?;

            let args = FuncBackendJsQualificationArgs {
                component: self
                    .veritech_qualification_check_component(ctx, system_id)
                    .await?,
            };

            let json_args = serde_json::to_value(args)?;
            let (func_binding, _created) = FuncBinding::find_or_create(
                ctx,
                json_args,
                prototype.func_id(),
                *func.backend_kind(),
            )
            .await?;

            // Empty func binding return value means the function is still being executed
            let _func_binding_return_value = FuncBindingReturnValue::upsert(
                ctx,
                None,
                None,
                prototype.func_id(),
                *func_binding.id(),
                UNSET_ID_VALUE.into(),
            )
            .await?;

            let mut existing_resolvers = QualificationResolver::find_for_prototype_and_component(
                ctx,
                prototype.id(),
                self.id(),
            )
            .await?;

            // If we do not have one, create the qualification resolver. If we do, update the
            // func binding id to point to the new value.
            if let Some(mut resolver) = existing_resolvers.pop() {
                resolver
                    .set_func_binding_id(ctx, *func_binding.id())
                    .await?;
            } else {
                let mut resolver_context = QualificationResolverContext::new();
                resolver_context.set_component_id(self.id);
                QualificationResolver::new(
                    ctx,
                    *prototype.id(),
                    *func.id(),
                    *func_binding.id(),
                    resolver_context,
                )
                .await?;
            }

            WsEvent::checked_qualifications(ctx, *prototype.id(), self.id, system_id)
                .publish(ctx)
                .await?;
        }

        Ok(())
    }

    pub async fn check_qualification(
        &self,
        ctx: &DalContext,
        system_id: SystemId,
        prototype_id: QualificationPrototypeId,
    ) -> ComponentResult<()> {
        let prototype = QualificationPrototype::get_by_id(ctx, &prototype_id)
            .await?
            .ok_or(ComponentError::QualificationPrototypeNotFound)?;

        let func = Func::get_by_id(ctx, &prototype.func_id())
            .await?
            .ok_or_else(|| ComponentError::MissingFunc(prototype.func_id().to_string()))?;

        let args = FuncBackendJsQualificationArgs {
            component: self
                .veritech_qualification_check_component(ctx, system_id)
                .await?,
        };

        let json_args = serde_json::to_value(args)?;
        let (func_binding, _created) =
            FuncBinding::find_or_create(ctx, json_args, prototype.func_id(), *func.backend_kind())
                .await?;

        // We always re-execute the qualification checks as they are not idempotent

        // Note for future humans - if this isn't a built in, then we need to
        // think about execution time. Probably higher up than this? But just
        // an FYI.
        func_binding.execute(ctx).await?;

        let mut existing_resolvers =
            QualificationResolver::find_for_prototype_and_component(ctx, prototype.id(), self.id())
                .await?;

        // If we do not have one, create the qualification resolver. If we do, update the
        // func binding id to point to the new value.
        if let Some(mut resolver) = existing_resolvers.pop() {
            resolver
                .set_func_binding_id(ctx, *func_binding.id())
                .await?;
        } else {
            let mut resolver_context = QualificationResolverContext::new();
            resolver_context.set_component_id(self.id);
            QualificationResolver::new(
                ctx,
                *prototype.id(),
                *func.id(),
                *func_binding.id(),
                resolver_context,
            )
            .await?;
        }

        WsEvent::checked_qualifications(ctx, *prototype.id(), self.id, system_id)
            .publish(ctx)
            .await?;

        Ok(())
    }

    pub async fn check_qualifications(
        &self,
        ctx: &DalContext,
        system_id: SystemId,
    ) -> ComponentResult<()> {
        let schema = self
            .schema(ctx)
            .await?
            .ok_or(ComponentError::NoSchema(self.id))?;
        let schema_variant = self
            .schema_variant(ctx)
            .await?
            .ok_or(ComponentError::NoSchemaVariant(self.id))?;

        let qualification_prototypes = QualificationPrototype::find_for_component(
            ctx,
            self.id,
            *schema.id(),
            *schema_variant.id(),
            system_id,
        )
        .await?;

        for prototype in qualification_prototypes {
            let func = Func::get_by_id(ctx, &prototype.func_id())
                .await?
                .ok_or_else(|| ComponentError::MissingFunc(prototype.func_id().to_string()))?;

            let args = FuncBackendJsQualificationArgs {
                component: self
                    .veritech_qualification_check_component(ctx, system_id)
                    .await?,
            };

            let json_args = serde_json::to_value(args)?;
            let (func_binding, _created) = FuncBinding::find_or_create(
                ctx,
                json_args,
                prototype.func_id(),
                *func.backend_kind(),
            )
            .await?;

            // We always re-execute the qualification checks as they are not idempotent

            // Note for future humans - if this isn't a built in, then we need to
            // think about execution time. Probably higher up than this? But just
            // an FYI.
            func_binding.execute(ctx).await?;

            let mut existing_resolvers = QualificationResolver::find_for_prototype_and_component(
                ctx,
                prototype.id(),
                self.id(),
            )
            .await?;

            // If we do not have one, create the qualification resolver. If we do, update the
            // func binding id to point to the new value.
            if let Some(mut resolver) = existing_resolvers.pop() {
                resolver
                    .set_func_binding_id(ctx, *func_binding.id())
                    .await?;
            } else {
                let mut resolver_context = QualificationResolverContext::new();
                resolver_context.set_component_id(self.id);
                QualificationResolver::new(
                    ctx,
                    *prototype.id(),
                    *func.id(),
                    *func_binding.id(),
                    resolver_context,
                )
                .await?;
            }

            WsEvent::checked_qualifications(ctx, *prototype.id(), self.id, system_id)
                .publish(ctx)
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
    pub async fn list_validations_as_qualification_for_component_id(
        ctx: &DalContext,
        component_id: ComponentId,
        system_id: SystemId,
    ) -> ComponentResult<QualificationView> {
        let validation_errors = ValidationResolver::find_status(ctx, component_id, system_id)
            .await?
            .into_iter()
            .flat_map(|s| s.errors)
            .collect();
        let qualification_view = QualificationView::new_for_validation_errors(validation_errors);
        Ok(qualification_view)
    }

    #[instrument(skip_all)]
    pub async fn list_qualifications(
        &self,
        ctx: &DalContext,
        system_id: SystemId,
    ) -> ComponentResult<Vec<QualificationView>> {
        Self::list_qualifications_by_component_id(ctx, self.id, system_id).await
    }

    #[instrument(skip_all)]
    pub async fn list_qualifications_by_component_id(
        ctx: &DalContext,
        component_id: ComponentId,
        system_id: SystemId,
    ) -> ComponentResult<Vec<QualificationView>> {
        let mut results: Vec<QualificationView> = Vec::new();

        // This is the "All Fields Valid" universal qualification
        let validation_qualification =
            Self::list_validations_as_qualification_for_component_id(ctx, component_id, system_id)
                .await?;
        results.push(validation_qualification);

        let rows = ctx
            .txns()
            .pg()
            .query(
                LIST_QUALIFICATIONS,
                &[
                    ctx.read_tenancy(),
                    ctx.visibility(),
                    &component_id,
                    &system_id,
                ],
            )
            .await?;
        let no_qualification_results = rows.is_empty();
        for row in rows.into_iter() {
            let json: serde_json::Value = row.try_get("object")?;
            let func_binding_return_value: FuncBindingReturnValue = serde_json::from_value(json)?;

            let json: serde_json::Value = row.try_get("func_metadata_view")?;
            let func_metadata_view: FuncMetadataView = serde_json::from_value(json)?;

            let json: serde_json::Value = row.try_get("prototype")?;
            let prototype: QualificationPrototype = serde_json::from_value(json)?;
            let qual_view = QualificationView::new_for_func_binding_return_value(
                ctx,
                prototype,
                func_metadata_view,
                func_binding_return_value,
            )
            .await?;
            results.push(qual_view);
        }
        // This is inefficient, but effective
        if no_qualification_results {
            let component = Self::get_by_id(ctx, &component_id)
                .await?
                .ok_or(ComponentError::NotFound(component_id))?;
            let schema = component
                .schema(ctx)
                .await?
                .ok_or(ComponentError::NoSchema(component.id))?;
            let schema_variant = component
                .schema_variant(ctx)
                .await?
                .ok_or(ComponentError::NoSchemaVariant(component.id))?;
            let prototypes = QualificationPrototype::find_for_component(
                ctx,
                component_id,
                *schema.id(),
                *schema_variant.id(),
                system_id,
            )
            .await?;
            for prototype in prototypes.into_iter() {
                let func = Func::get_by_id(ctx, &prototype.func_id())
                    .await?
                    .ok_or_else(|| ComponentError::FuncNotFound(prototype.func_id()))?;

                let qual_view = QualificationView::new_for_qualification_prototype(
                    prototype,
                    func.metadata_view(),
                );
                results.push(qual_view);
            }
        }
        Ok(results)
    }

    pub async fn veritech_code_generation_component(
        &self,
        ctx: &DalContext,
    ) -> ComponentResult<ComponentView> {
        Self::view(ctx, self.id, SystemId::NONE).await
    }

    pub async fn view(
        ctx: &DalContext,
        component_id: ComponentId,
        system_id: SystemId,
    ) -> ComponentResult<ComponentView> {
        let component = Self::get_by_id(ctx, &component_id)
            .await?
            .ok_or(ComponentError::NotFound(component_id))?;
        let schema = component
            .schema(ctx)
            .await?
            .ok_or(ComponentError::NoSchema(component.id))?;
        let schema_variant = component
            .schema_variant(ctx)
            .await?
            .ok_or(ComponentError::NoSchemaVariant(component.id))?;
        let read_context = AttributeReadContext {
            prop_id: None,
            schema_id: Some(*schema.id()),
            schema_variant_id: Some(*schema_variant.id()),
            component_id: Some(*component.id()),
            system_id: Some(system_id),
            ..AttributeReadContext::default()
        };
        Ok(ComponentView::for_context(ctx, read_context, false).await?)
    }

    pub async fn veritech_qualification_check_component(
        &self,
        ctx: &DalContext,
        system_id: SystemId,
    ) -> ComponentResult<veritech_client::QualificationCheckComponent> {
        let parent_ids = Edge::list_parents_for_component(ctx, self.id).await?;

        let mut parents = Vec::new();
        for id in parent_ids {
            let view = Self::view(ctx, id, system_id).await?;
            parents.push(view.into());
        }

        let qualification_view = veritech_client::QualificationCheckComponent {
            data: Self::view(ctx, self.id, system_id).await?.into(),
            codes: Self::list_code_generated(ctx, self.id)
                .await?
                .into_iter()
                .flat_map(|view| {
                    let format = view.language.to_string();
                    view.code
                        .map(|code| veritech_client::CodeGenerated { format, code })
                })
                .collect(),
            parents,
        };
        Ok(qualification_view)
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
                &[ctx.visibility(), ctx.read_tenancy(), &schema_variant_id],
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
    pub async fn set_value_by_json_pointer<T: Serialize + std::fmt::Debug + std::clone::Clone>(
        &self,
        ctx: &DalContext,
        json_pointer: &str,
        value: Option<T>,
    ) -> ComponentResult<Option<T>> {
        let attribute_value = self
            .find_attribute_value_by_json_pointer(ctx, json_pointer)
            .await?
            .ok_or(AttributeValueError::Missing)?;

        let schema_variant = self
            .schema_variant(ctx)
            .await?
            .ok_or(ComponentError::NoSchemaVariant(self.id))?;
        let schema = self
            .schema(ctx)
            .await?
            .ok_or(ComponentError::NoSchema(self.id))?;

        let attribute_context = AttributeContext::builder()
            .set_component_id(self.id)
            .set_schema_variant_id(*schema_variant.id())
            .set_schema_id(*schema.id())
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
            let schema = self
                .schema(ctx)
                .await?
                .ok_or(ComponentError::NoSchema(self.id))?;
            let schema_variant = self
                .schema_variant(ctx)
                .await?
                .ok_or(ComponentError::NoSchemaVariant(self.id))?;

            // System will be unset since this method should only be used when creating a component.
            let read_context = AttributeReadContext {
                prop_id: Some(*prop.id()),
                schema_id: Some(*schema.id()),
                schema_variant_id: Some(*schema_variant.id()),
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
        let schema = self
            .schema(ctx)
            .await?
            .ok_or(ComponentError::NoSchema(self.id))?;
        let schema_variant = self
            .schema_variant(ctx)
            .await?
            .ok_or(ComponentError::NoSchemaVariant(self.id))?;
        let prop = Prop::find_root_for_schema_variant(ctx, *schema_variant.id())
            .await?
            .ok_or_else(|| ComponentError::RootPropNotFound(*schema_variant.id()))?;

        let implicit_provider = InternalProvider::get_for_prop(ctx, *prop.id())
            .await?
            .ok_or_else(|| ComponentError::InternalProviderNotFoundForProp(*prop.id()))?;

        let value_context = AttributeReadContext {
            schema_id: Some(*schema.id()),
            schema_variant_id: Some(*schema_variant.id()),
            component_id: Some(self.id),
            system_id: Some(SystemId::NONE),

            internal_provider_id: Some(*implicit_provider.id()),
            prop_id: Some(PropId::NONE),
            external_provider_id: Some(ExternalProviderId::NONE),
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
                &[
                    ctx.read_tenancy(),
                    ctx.visibility(),
                    &component_id,
                    &attribute_read_context.system_id(),
                ],
            )
            .await?;
        let value: serde_json::Value = row.try_get("component_name")?;
        let component_name: String = serde_json::from_value(value)?;

        Ok(component_name)
    }

    pub async fn name(&self, ctx: &DalContext) -> ComponentResult<String> {
        let context = AttributeReadContext {
            component_id: Some(self.id),
            system_id: Some(SystemId::NONE),
            ..AttributeReadContext::any()
        };
        Self::name_from_context(ctx, context).await
    }

    /// Grabs the [`AttributeValue`](crate::AttributeValue) corresponding to the "/root/<child>"
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

    pub async fn resource(&self, ctx: &DalContext) -> ComponentResult<Value> {
        let attribute_value =
            Component::root_child_attribute_value_for_component(ctx, "resource", self.id).await?;
        let func_binding_return_value =
            FuncBindingReturnValue::get_by_id(ctx, &attribute_value.func_binding_return_value_id())
                .await?
                .ok_or_else(|| {
                    ComponentError::FuncBindingReturnValueNotFound(
                        attribute_value.func_binding_return_value_id(),
                    )
                })?;

        // Resources currently get stored as JSON in a string prop, they eventually will be complete objects
        let value = match func_binding_return_value.value() {
            None => serde_json::Value::Null,
            Some(serde_json::Value::String(string)) => serde_json::from_str(string)?,
            Some(v) => v.clone(),
        };
        Ok(value)
    }

    /// Sets the "string" field, "/root/resource" with a given value. After that, ensure dependent
    /// [`AttributeValues`](crate::AttributeValue) are updated.
    pub async fn set_resource(&self, ctx: &DalContext, data: Value) -> ComponentResult<()> {
        let resource_attribute_value =
            Component::root_child_attribute_value_for_component(ctx, "resource", self.id).await?;
        let func_binding_return_value = FuncBindingReturnValue::get_by_id(
            ctx,
            &resource_attribute_value.func_binding_return_value_id(),
        )
        .await?
        .ok_or_else(|| {
            ComponentError::FuncBindingReturnValueNotFound(
                resource_attribute_value.func_binding_return_value_id(),
            )
        })?;

        // No need to trigger a /root update if the value hasn't changed
        if Some(&data) == func_binding_return_value.value() {
            return Ok(());
        }

        let root_attribute_value = resource_attribute_value
            .parent_attribute_value(ctx)
            .await?
            .ok_or_else(|| AttributeValueError::ParentNotFound(*resource_attribute_value.id()))?;

        let schema = self
            .schema(ctx)
            .await?
            .ok_or(ComponentError::NoSchema(self.id))?;
        let schema_variant = self
            .schema_variant(ctx)
            .await?
            .ok_or(ComponentError::NoSchemaVariant(self.id))?;

        let update_attribute_context =
            AttributeContextBuilder::from(resource_attribute_value.context)
                .set_schema_id(*schema.id())
                .set_schema_variant_id(*schema_variant.id())
                .set_component_id(self.id)
                .to_context()?;

        let (_, _) = AttributeValue::update_for_context(
            ctx,
            *resource_attribute_value.id(),
            Some(*root_attribute_value.id()),
            update_attribute_context,
            Some(data),
            None,
        )
        .await?;
        Ok(())
    }

    pub async fn act(&self, ctx: &DalContext, action_name: &str) -> ComponentResult<()> {
        let schema_variant = self
            .schema_variant(ctx)
            .await?
            .ok_or(ComponentError::NoSchemaVariant(self.id))?;
        let schema = self
            .schema(ctx)
            .await?
            .ok_or(ComponentError::NoSchema(self.id))?;
        let action = match ActionPrototype::find_by_name(
            ctx,
            action_name,
            *schema.id(),
            *schema_variant.id(),
            SystemId::NONE,
        )
        .await?
        {
            Some(action) => action,
            None => return Ok(()),
        };

        let prototype = WorkflowPrototype::get_by_id(ctx, &action.workflow_prototype_id())
            .await?
            .ok_or_else(|| {
                ComponentError::WorkflowPrototypeNotFound(action.workflow_prototype_id())
            })?;
        let run_id: usize = rand::random();
        let (_runner, _state, _func_binding_return_values, _created_resources, _updated_resources) =
            WorkflowRunner::run(ctx, run_id, *prototype.id(), self.id).await?;
        WsEvent::resource_refreshed(ctx, self.id, SystemId::NONE)
            .publish(ctx)
            .await?;
        Ok(())
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ResourceView {
    pub data: Value,
}

impl ResourceView {
    pub fn new(data: serde_json::Value) -> Self {
        Self { data }
    }
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ResourceRefreshId {
    component_id: ComponentId,
    system_id: SystemId,
}

impl WsEvent {
    pub fn resource_refreshed(
        ctx: &DalContext,
        component_id: ComponentId,
        system_id: SystemId,
    ) -> Self {
        WsEvent::new(
            ctx,
            WsPayload::ResourceRefreshed(ResourceRefreshId {
                component_id,
                system_id,
            }),
        )
    }
}
