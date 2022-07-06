pub mod view;

pub use view::{ComponentView, ComponentViewError};

use serde::{Deserialize, Serialize};
use si_data::{NatsError, PgError, PgTxn};
use strum_macros::{AsRefStr, Display, EnumIter, EnumString};
use telemetry::prelude::*;
use thiserror::Error;

use crate::attribute::value::AttributeValue;
use crate::attribute::{context::UNSET_ID_VALUE, value::AttributeValueError};
use crate::code_generation_resolver::CodeGenerationResolverContext;
use crate::context::JobContent;
use crate::func::backend::validation::FuncBackendValidateStringValueArgs;
use crate::func::backend::{
    js_code_generation::FuncBackendJsCodeGenerationArgs,
    js_qualification::FuncBackendJsQualificationArgs, js_resource::FuncBackendJsResourceSyncArgs,
};
use crate::func::binding::{FuncBinding, FuncBindingError};
use crate::func::binding_return_value::{
    FuncBindingReturnValue, FuncBindingReturnValueError, FuncBindingReturnValueId,
};
use crate::qualification::QualificationView;
use crate::qualification_resolver::QualificationResolverContext;
use crate::resource_resolver::ResourceResolverContext;
use crate::schema::variant::{SchemaVariantError, SchemaVariantId};
use crate::schema::SchemaVariant;
use crate::ws_event::{WsEvent, WsEventError};
use crate::{
    context::AccessBuilder, context::DalContextBuilder, func::FuncId, impl_standard_model,
    node::NodeId, pk, provider::internal::InternalProviderError, qualification::QualificationError,
    standard_model, standard_model_accessor, standard_model_belongs_to, standard_model_has_many,
    AttributeContext, AttributeContextBuilderError, AttributeContextError, AttributeReadContext,
    CodeGenerationPrototype, CodeGenerationPrototypeError, CodeGenerationResolver,
    CodeGenerationResolverError, CodeLanguage, CodeView, DalContext, Edge, EdgeError, Func,
    FuncBackendKind, HistoryEventError, Node, NodeError, OrganizationError, Prop, PropError,
    PropId, QualificationPrototype, QualificationPrototypeError, QualificationResolver,
    QualificationResolverError, ReadTenancyError, Resource, ResourceError, ResourcePrototype,
    ResourcePrototypeError, ResourceResolver, ResourceResolverError, ResourceView, Schema,
    SchemaError, SchemaId, StandardModel, StandardModelError, SystemId, Timestamp,
    TransactionsError, ValidationPrototype, ValidationPrototypeError, ValidationResolver,
    ValidationResolverError, Visibility, WorkspaceError, WriteTenancy,
};
use crate::{AttributeValueId, QualificationPrototypeId};

#[derive(Error, Debug)]
pub enum ComponentError {
    #[error("attribute context error: {0}")]
    AttributeContext(#[from] AttributeContextError),
    #[error("attribute context builder error: {0}")]
    AttributeContextBuilder(#[from] AttributeContextBuilderError),
    #[error("attribute value error: {0}")]
    AttributeValue(#[from] AttributeValueError),
    #[error("invalid json pointer: {0} for {1}")]
    BadJsonPointer(String, String),
    #[error("codegen function returned unexpected format, expected {0:?}, got {1:?}")]
    CodeLanguageMismatch(CodeLanguage, CodeLanguage),
    #[error("internal provider error: {0}")]
    InternalProvider(#[from] InternalProviderError),
    #[error("edge error: {0}")]
    Edge(#[from] EdgeError),
    #[error("func not found: {0}")]
    FuncNotFound(FuncId),
    #[error(transparent)]
    FuncBindingReturnValue(#[from] FuncBindingReturnValueError),
    #[error("missing attribute value for id: ({0})")]
    MissingAttributeValue(AttributeValueId),
    #[error("missing index map on attribute value: {0}")]
    MissingIndexMap(AttributeValueId),
    #[error("expected one root prop, found multiple: {0:?}")]
    MultipleRootProps(Vec<Prop>),
    #[error("root prop not found for schema variant: {0}")]
    RootPropNotFound(SchemaVariantId),
    #[error("internal provider not found for prop: {0}")]
    InternalProviderNotFoundForProp(PropId),
    #[error("attrubte value not found for context: {0:?}")]
    AttributeValueNotFoundForContext(AttributeReadContext),

    // FIXME: change the below to be alphabetical and re-join with the above variants.
    #[error("qualification prototype error: {0}")]
    QualificationPrototype(#[from] QualificationPrototypeError),
    #[error("qualification resolver error: {0}")]
    QualificationResolver(#[from] QualificationResolverError),
    #[error("resource prototype error: {0}")]
    ResourcePrototype(#[from] ResourcePrototypeError),
    #[error("resource resolver error: {0}")]
    ResourceResolver(#[from] ResourceResolverError),
    #[error("code generation prototype error: {0}")]
    CodeGenerationPrototype(#[from] CodeGenerationPrototypeError),
    #[error("code generation resolver error: {0}")]
    CodeGenerationResolver(#[from] CodeGenerationResolverError),
    #[error("unable to find code generated")]
    CodeGeneratedNotFound,
    #[error("qualification prototype not found")]
    QualificationPrototypeNotFound,
    #[error("error serializing/deserializing json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error(transparent)]
    PgPool(#[from] si_data::PgPoolError),
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
    #[error("resource not found for component ({0}) in system ({1})")]
    ResourceNotFound(ComponentId, SystemId),
    #[error("schema error: {0}")]
    Schema(#[from] SchemaError),
    #[error("schema variant not found")]
    SchemaVariantNotFound,
    #[error("schema not found")]
    SchemaNotFound,
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
    #[error("invalid prop value; expected {0} but got {1}")]
    InvalidPropValue(String, serde_json::Value),
    #[error("func binding error: {0}")]
    FuncBinding(#[from] FuncBindingError),
    #[error("validation resolver error: {0}")]
    ValidationResolver(#[from] ValidationResolverError),
    #[error("validation prototype error: {0}")]
    ValidationPrototype(#[from] ValidationPrototypeError),
    #[error("qualification view error: {0}")]
    QualificationView(#[from] QualificationError),
    #[error("resource error: {0}")]
    Resource(#[from] ResourceError),
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
}

pub type ComponentResult<T> = Result<T, ComponentError>;

const GET_RESOURCE: &str = include_str!("./queries/component_get_resource.sql");
const LIST_QUALIFICATIONS: &str = include_str!("./queries/component_list_qualifications.sql");
const LIST_CODE_GENERATED: &str = include_str!("./queries/component_list_code_generated.sql");
const LIST_FOR_RESOURCE_SYNC: &str = include_str!("./queries/component_list_for_resource_sync.sql");
const LIST_FOR_SCHEMA_VARIANT: &str =
    include_str!("./queries/component_list_for_schema_variant.sql");

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
        ctx: &DalContext<'_, '_>,
        name: impl AsRef<str>,
        schema_id: &SchemaId,
    ) -> ComponentResult<(Self, Node)> {
        let schema = Schema::get_by_id(ctx, schema_id)
            .await?
            .ok_or(ComponentError::SchemaNotFound)?;

        let schema_variant_id = schema
            .default_schema_variant_id()
            .ok_or(ComponentError::SchemaVariantNotFound)?;

        Self::new_for_schema_variant_with_node(ctx, name, schema_variant_id).await
    }

    #[instrument(skip_all)]
    pub async fn new_for_schema_variant_with_node(
        ctx: &DalContext<'_, '_>,
        name: impl AsRef<str>,
        schema_variant_id: &SchemaVariantId,
    ) -> ComponentResult<(Self, Node)> {
        let schema_variant = SchemaVariant::get_by_id(ctx, schema_variant_id)
            .await?
            .ok_or(ComponentError::SchemaVariantNotFound)?;
        let schema = schema_variant
            .schema(ctx)
            .await?
            .ok_or(ComponentError::SchemaNotFound)?;

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

        if let Some(root_node_id) = ctx.application_node_id() {
            let _edge = Edge::include_component_in_node(
                ctx,
                component.id(),
                &schema.kind().into(),
                &root_node_id,
            )
            .await?;
        }

        let _ = component
            .set_value_by_json_pointer(ctx, "/root/si/name", Some(name.as_ref()))
            .await?;
        Ok((component, node))
    }

    pub async fn build_async_tasks(
        &self,
        ctx: &DalContext<'_, '_>,
        system_id: SystemId,
    ) -> ComponentResult<ComponentAsyncTasks> {
        self.prepare_code_generation(ctx, system_id).await?;
        self.prepare_qualifications_check(ctx, system_id).await?;
        Ok(ComponentAsyncTasks::new(*self.id(), system_id))
    }

    pub async fn build_async_task(
        &self,
        ctx: &DalContext<'_, '_>,
        system_id: SystemId,
        qualification_prototype_id: QualificationPrototypeId,
    ) -> ComponentResult<ComponentAsyncTasks> {
        self.prepare_code_generation(ctx, system_id).await?;
        self.prepare_qualification_check(ctx, system_id, qualification_prototype_id)
            .await?;
        let mut task = ComponentAsyncTasks::new(*self.id(), system_id);
        task.set_qualification_prototype_id(qualification_prototype_id);
        Ok(task)
    }

    #[instrument(skip_all)]
    pub async fn new_for_schema_variant_with_node_in_deployment(
        ctx: &DalContext<'_, '_>,
        name: impl AsRef<str>,
        schema_variant_id: &SchemaVariantId,
        parent_node_id: &NodeId,
    ) -> ComponentResult<(Self, Node)> {
        let (component, node) =
            Self::new_for_schema_variant_with_node(ctx, name, schema_variant_id).await?;
        let schema = component
            .schema(ctx)
            .await?
            .ok_or(ComponentError::SchemaNotFound)?;
        let _edge = Edge::include_component_in_node(
            ctx,
            component.id(),
            &schema.kind().into(),
            parent_node_id,
        )
        .await?;

        Ok((component, node))
    }

    #[instrument(skip_all)]
    pub async fn new_application_with_node(
        ctx: &DalContext<'_, '_>,
        name: impl AsRef<str>,
    ) -> ComponentResult<(Self, Node)> {
        let ctx = ctx.clone_with_new_application_node_id(None);

        let schema_variant_id =
            Schema::default_schema_variant_id_for_name(&ctx, "application").await?;
        let (comp, node) =
            Self::new_for_schema_variant_with_node(&ctx, name, &schema_variant_id).await?;
        Ok((comp, node))
    }

    #[instrument(skip_all)]
    pub async fn add_to_system(
        &self,
        ctx: &DalContext<'_, '_>,
        system_id: &SystemId,
    ) -> ComponentResult<()> {
        let schema = self
            .schema(ctx)
            .await?
            .ok_or(ComponentError::SchemaNotFound)?;
        let _edge =
            Edge::include_component_in_system(ctx, &self.id, &schema.kind().into(), system_id)
                .await;

        // NOTE: We may want to be a bit smarter about when we create the Resource
        //       at some point in the future, by only creating it if there is also
        //       a ResourcePrototype for the Component's SchemaVariant.
        let _resource = Resource::new(ctx, &self.id, system_id).await?;

        ctx.enqueue_job(JobContent::ComponentPostProcessing(
            self.build_async_tasks(ctx, *system_id).await?,
        ))
        .await;

        Ok(())
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

    pub async fn check_validations(
        &self,
        ctx: &DalContext<'_, '_>,
        attribute_value_id: AttributeValueId,
        value: &Option<serde_json::Value>,
    ) -> ComponentResult<()> {
        let attribute_value = AttributeValue::get_by_id(ctx, &attribute_value_id)
            .await?
            .ok_or(ComponentError::MissingAttributeValue(attribute_value_id))?;
        let prop_id = attribute_value.context.prop_id();

        let validators =
            ValidationPrototype::find_for_prop(ctx, prop_id, UNSET_ID_VALUE.into()).await?;

        for validator in validators {
            let func = Func::get_by_id(ctx, &validator.func_id())
                .await?
                .ok_or_else(|| ComponentError::MissingFunc(validator.func_id().to_string()))?;
            let (func_binding, created) = match func.backend_kind() {
                FuncBackendKind::ValidateStringValue => {
                    let mut args =
                        FuncBackendValidateStringValueArgs::deserialize(validator.args())?;
                    if let Some(json_value) = value {
                        if json_value.is_string() {
                            args.value = Some(json_value.to_string());
                        } else {
                            return Err(ComponentError::InvalidPropValue(
                                "String".to_string(),
                                json_value.clone(),
                            ));
                        }
                    } else {
                        // TODO: This might not be quite the right error to return here if we got a None.
                        return Err(ComponentError::MissingProp(prop_id));
                    };
                    let args_json = serde_json::to_value(args)?;
                    let (func_binding, binding_created) = FuncBinding::find_or_create(
                        ctx,
                        args_json,
                        *func.id(),
                        *func.backend_kind(),
                    )
                    .await?;
                    // Note for future humans - if this isn't a built in, then we need to
                    // think about execution time. Probably higher up than this? But just
                    // an FYI.
                    if binding_created {
                        func_binding.execute(ctx).await?;
                    }
                    (func_binding, binding_created)
                }
                kind => unimplemented!("Validator Backend not supported yet: {}", kind),
            };

            if created {
                ValidationResolver::new(
                    ctx,
                    *validator.id(),
                    attribute_value_id,
                    *func_binding.id(),
                )
                .await?;
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
        ctx: &DalContext<'_, '_>,
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
            resolver_context.set_component_id(*self.id());
            QualificationResolver::new(
                ctx,
                *prototype.id(),
                *func.id(),
                *func_binding.id(),
                resolver_context,
            )
            .await?;
        }

        WsEvent::checked_qualifications(
            *prototype.id(),
            *self.id(),
            system_id,
            ctx.read_tenancy().billing_accounts().into(),
            ctx.history_actor(),
        )
        .publish(ctx.txns().nats())
        .await?;

        Ok(())
    }

    /// Creates a qualification [`FuncBinding`](crate::FuncBinding), a
    /// [`FuncBindingReturnValue`](crate::FuncBindingReturnValue) without a value and a
    /// [`QualificationResolver`](crate::QualificationResolver). The func is not executed yet; it's
    /// just a placeholder for some qualification that will be executed.
    pub async fn prepare_qualifications_check(
        &self,
        ctx: &DalContext<'_, '_>,
        system_id: SystemId,
    ) -> ComponentResult<()> {
        let schema = self
            .schema(ctx)
            .await?
            .ok_or(ComponentError::SchemaNotFound)?;
        let schema_variant = self
            .schema_variant(ctx)
            .await?
            .ok_or(ComponentError::SchemaVariantNotFound)?;

        let qualification_prototypes = QualificationPrototype::find_for_component(
            ctx,
            *self.id(),
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
                resolver_context.set_component_id(*self.id());
                QualificationResolver::new(
                    ctx,
                    *prototype.id(),
                    *func.id(),
                    *func_binding.id(),
                    resolver_context,
                )
                .await?;
            }

            WsEvent::checked_qualifications(
                *prototype.id(),
                *self.id(),
                system_id,
                ctx.read_tenancy().billing_accounts().into(),
                ctx.history_actor(),
            )
            .publish(ctx.txns().nats())
            .await?;
        }

        Ok(())
    }

    pub async fn check_qualification(
        &self,
        ctx: &DalContext<'_, '_>,
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
            resolver_context.set_component_id(*self.id());
            QualificationResolver::new(
                ctx,
                *prototype.id(),
                *func.id(),
                *func_binding.id(),
                resolver_context,
            )
            .await?;
        }

        WsEvent::checked_qualifications(
            *prototype.id(),
            *self.id(),
            system_id,
            ctx.read_tenancy().billing_accounts().into(),
            ctx.history_actor(),
        )
        .publish(ctx.txns().nats())
        .await?;

        Ok(())
    }

    pub async fn check_qualifications(
        &self,
        ctx: &DalContext<'_, '_>,
        system_id: SystemId,
    ) -> ComponentResult<()> {
        let schema = self
            .schema(ctx)
            .await?
            .ok_or(ComponentError::SchemaNotFound)?;
        let schema_variant = self
            .schema_variant(ctx)
            .await?
            .ok_or(ComponentError::SchemaVariantNotFound)?;

        let qualification_prototypes = QualificationPrototype::find_for_component(
            ctx,
            *self.id(),
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
                resolver_context.set_component_id(*self.id());
                QualificationResolver::new(
                    ctx,
                    *prototype.id(),
                    *func.id(),
                    *func_binding.id(),
                    resolver_context,
                )
                .await?;
            }

            WsEvent::checked_qualifications(
                *prototype.id(),
                *self.id(),
                system_id,
                ctx.read_tenancy().billing_accounts().into(),
                ctx.history_actor(),
            )
            .publish(ctx.txns().nats())
            .await?;
        }

        Ok(())
    }

    /// Creates code generation [`FuncBinding`](crate::FuncBinding), a
    /// [`FuncBindingReturnValue`](crate::FuncBindingReturnValue) without a value and a
    /// [`CodeGenerationResolver`](crate::CodeGenerationResolver). The func is not executed yet,
    /// it's just a placeholder for some code generation that will be executed.
    pub async fn prepare_code_generation(
        &self,
        ctx: &DalContext<'_, '_>,
        system_id: SystemId,
    ) -> ComponentResult<()> {
        let schema = self
            .schema(ctx)
            .await?
            .ok_or(ComponentError::SchemaNotFound)?;
        let schema_variant = self
            .schema_variant(ctx)
            .await?
            .ok_or(ComponentError::SchemaVariantNotFound)?;

        let code_generation_prototypes = CodeGenerationPrototype::find_for_component(
            ctx,
            *self.id(),
            *schema.id(),
            *schema_variant.id(),
            system_id,
        )
        .await?;

        for prototype in code_generation_prototypes {
            let func = Func::get_by_id(ctx, &prototype.func_id())
                .await?
                .ok_or_else(|| ComponentError::MissingFunc(prototype.func_id().to_string()))?;

            let args = FuncBackendJsCodeGenerationArgs {
                component: self
                    .veritech_code_generation_component(ctx, system_id)
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

            // Empty func_binding_return_value means the function is still being executed
            let _func_binding_return_value = FuncBindingReturnValue::upsert(
                ctx,
                None,
                None,
                prototype.func_id(),
                *func_binding.id(),
                UNSET_ID_VALUE.into(),
            )
            .await?;

            let mut existing_resolvers = CodeGenerationResolver::find_for_prototype_and_component(
                ctx,
                prototype.id(),
                self.id(),
            )
            .await?;

            // If we do not have one, create the code generation resolver. If we do, update the
            // func binding id to point to the new value.
            if let Some(mut resolver) = existing_resolvers.pop() {
                resolver
                    .set_func_binding_id(ctx, *func_binding.id())
                    .await?;
            } else {
                let mut resolver_context = CodeGenerationResolverContext::new();
                resolver_context.set_component_id(*self.id());
                let _resolver = CodeGenerationResolver::new(
                    ctx,
                    *prototype.id(),
                    *func.id(),
                    *func_binding.id(),
                    resolver_context,
                )
                .await?;
            }
        }

        WsEvent::code_generated(
            *self.id(),
            system_id,
            ctx.read_tenancy().billing_accounts().into(),
            ctx.history_actor(),
        )
        .publish(ctx.txns().nats())
        .await?;

        Ok(())
    }

    pub async fn generate_code(
        &self,
        ctx: &DalContext<'_, '_>,
        system_id: SystemId,
    ) -> ComponentResult<()> {
        let schema = self
            .schema(ctx)
            .await?
            .ok_or(ComponentError::SchemaNotFound)?;
        let schema_variant = self
            .schema_variant(ctx)
            .await?
            .ok_or(ComponentError::SchemaVariantNotFound)?;

        let code_generation_prototypes = CodeGenerationPrototype::find_for_component(
            ctx,
            *self.id(),
            *schema.id(),
            *schema_variant.id(),
            system_id,
        )
        .await?;

        for prototype in code_generation_prototypes {
            let func = Func::get_by_id(ctx, &prototype.func_id())
                .await?
                .ok_or_else(|| ComponentError::MissingFunc(prototype.func_id().to_string()))?;

            let args = FuncBackendJsCodeGenerationArgs {
                component: self
                    .veritech_code_generation_component(ctx, system_id)
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

            // We always re-execute the code generation, as the previous one might have failed
            // This is a temporary work-around until we have a battle-tested failure-detection
            // system for async tasks

            // Note for future humans - if this isn't a built in, then we need to
            // think about execution time. Probably higher up than this? But just
            // an FYI.
            func_binding.execute(ctx).await?;

            let mut existing_resolvers = CodeGenerationResolver::find_for_prototype_and_component(
                ctx,
                prototype.id(),
                self.id(),
            )
            .await?;

            // If we do not have one, create the code generation resolver. If we do, update the
            // func binding id to point to the new value.
            if let Some(mut resolver) = existing_resolvers.pop() {
                resolver
                    .set_func_binding_id(ctx, *func_binding.id())
                    .await?;
            } else {
                let mut resolver_context = CodeGenerationResolverContext::new();
                resolver_context.set_component_id(*self.id());
                let _resolver = CodeGenerationResolver::new(
                    ctx,
                    *prototype.id(),
                    *func.id(),
                    *func_binding.id(),
                    resolver_context,
                )
                .await?;
            }
        }

        WsEvent::code_generated(
            *self.id(),
            system_id,
            ctx.read_tenancy().billing_accounts().into(),
            ctx.history_actor(),
        )
        .publish(ctx.txns().nats())
        .await?;

        Ok(())
    }

    #[instrument(skip_all)]
    pub async fn is_in_tenancy(ctx: &DalContext<'_, '_>, id: ComponentId) -> ComponentResult<bool> {
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
        ctx: &DalContext<'_, '_>,
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
    pub async fn list_code_generated_by_component_id(
        ctx: &DalContext<'_, '_>,
        component_id: ComponentId,
        system_id: SystemId,
    ) -> ComponentResult<Vec<CodeView>> {
        let mut results = Vec::new();

        let rows = ctx
            .txns()
            .pg()
            .query(
                LIST_CODE_GENERATED,
                &[
                    ctx.read_tenancy(),
                    ctx.visibility(),
                    &component_id,
                    &system_id,
                ],
            )
            .await?;
        for row in rows {
            let format: String = row.try_get("format")?;
            let format = CodeLanguage::deserialize(serde_json::Value::String(format.clone()))
                .unwrap_or_else(|err| {
                    error!("Unable to identify format {} ({err})", format);
                    CodeLanguage::Unknown
                });

            let json: serde_json::Value = row.try_get("object")?;
            let func_binding_return_value: FuncBindingReturnValue = serde_json::from_value(json)?;
            if let Some(value) = func_binding_return_value.value() {
                let code_generated = veritech::CodeGenerated::deserialize(value)?;

                let lang = CodeLanguage::deserialize(serde_json::Value::String(
                    code_generated.format.clone(),
                ))
                .unwrap_or_else(|err| {
                    error!(
                        "Unable to identify format {} ({err})",
                        code_generated.format
                    );
                    CodeLanguage::Unknown
                });

                if lang != format {
                    return Err(ComponentError::CodeLanguageMismatch(lang, format));
                }

                results.push(CodeView::new(format, Some(code_generated.code)));
            } else {
                // Means the code generation is being executed
                results.push(CodeView::new(format, None));
            }
        }
        Ok(results)
    }

    #[instrument(skip_all)]
    pub async fn list_qualifications(
        &self,
        ctx: &DalContext<'_, '_>,
        system_id: SystemId,
    ) -> ComponentResult<Vec<QualificationView>> {
        Self::list_qualifications_by_component_id(ctx, *self.id(), system_id).await
    }

    #[instrument(skip_all)]
    pub async fn list_qualifications_by_component_id(
        ctx: &DalContext<'_, '_>,
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

            let json: serde_json::Value = row.try_get("prototype")?;
            let prototype: QualificationPrototype = serde_json::from_value(json)?;
            let qual_view = QualificationView::new_for_func_binding_return_value(
                ctx,
                prototype,
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
                .ok_or(ComponentError::SchemaNotFound)?;
            let schema_variant = component
                .schema_variant(ctx)
                .await?
                .ok_or(ComponentError::SchemaVariantNotFound)?;
            let prototypes = QualificationPrototype::find_for_component(
                ctx,
                component_id,
                *schema.id(),
                *schema_variant.id(),
                system_id,
            )
            .await?;
            for prototype in prototypes.into_iter() {
                let qual_view = QualificationView::new_for_qualification_prototype(prototype);
                results.push(qual_view);
            }
        }
        Ok(results)
    }

    #[instrument(skip_all)]
    pub async fn get_resource_by_component_and_system(
        ctx: &DalContext<'_, '_>,
        component_id: ComponentId,
        system_id: SystemId,
    ) -> ComponentResult<Option<ResourceView>> {
        let resource =
            Resource::get_by_component_id_and_system_id(ctx, &component_id, &system_id).await?;
        let resource = match resource {
            Some(r) => r,
            None => return Ok(None),
        };

        let row = ctx
            .txns()
            .pg()
            .query_opt(
                GET_RESOURCE,
                &[
                    ctx.read_tenancy(),
                    ctx.visibility(),
                    &component_id,
                    &system_id,
                ],
            )
            .await?;

        let json: Option<serde_json::Value> = row.map(|row| row.try_get("object")).transpose()?;

        let func_binding_return_value: Option<FuncBindingReturnValue> =
            json.map(serde_json::from_value).transpose()?;
        let res_view = ResourceView::from((resource, func_binding_return_value));

        Ok(Some(res_view))
    }

    pub async fn veritech_code_generation_component(
        &self,
        ctx: &DalContext<'_, '_>,
        system_id: SystemId,
    ) -> ComponentResult<ComponentView> {
        let schema = self
            .schema(ctx)
            .await?
            .ok_or(ComponentError::SchemaNotFound)?;
        let schema_variant = self
            .schema_variant(ctx)
            .await?
            .ok_or(ComponentError::SchemaVariantNotFound)?;
        let attribute_context = AttributeReadContext {
            prop_id: None,
            schema_id: Some(*schema.id()),
            schema_variant_id: Some(*schema_variant.id()),
            component_id: Some(*self.id()),
            system_id: Some(system_id),
            ..AttributeReadContext::default()
        };

        let component = ComponentView::for_context(ctx, attribute_context).await?;
        Ok(component)
    }

    pub async fn veritech_resource_sync_component(
        &self,
        ctx: &DalContext<'_, '_>,
        system_id: SystemId,
    ) -> ComponentResult<ComponentView> {
        let schema = self
            .schema(ctx)
            .await?
            .ok_or(ComponentError::SchemaNotFound)?;
        let schema_variant = self
            .schema_variant(ctx)
            .await?
            .ok_or(ComponentError::SchemaVariantNotFound)?;
        let attribute_context = AttributeReadContext {
            prop_id: None,
            schema_id: Some(*schema.id()),
            schema_variant_id: Some(*schema_variant.id()),
            component_id: Some(*self.id()),
            system_id: Some(system_id),
            ..AttributeReadContext::default()
        };

        let component = ComponentView::for_context(ctx, attribute_context).await?;
        Ok(component)
    }

    pub async fn veritech_qualification_check_component(
        &self,
        ctx: &DalContext<'_, '_>,
        system_id: SystemId,
    ) -> ComponentResult<veritech::QualificationCheckComponent> {
        let schema = self
            .schema(ctx)
            .await?
            .ok_or(ComponentError::SchemaNotFound)?;
        let schema_variant = self
            .schema_variant(ctx)
            .await?
            .ok_or(ComponentError::SchemaVariantNotFound)?;
        let read_context = AttributeReadContext {
            prop_id: None,
            schema_id: Some(*schema.id()),
            schema_variant_id: Some(*schema_variant.id()),
            component_id: Some(*self.id()),
            system_id: Some(system_id),
            ..AttributeReadContext::default()
        };

        let parent_ids = Edge::find_component_configuration_parents(ctx, self.id()).await?;

        let mut parents = Vec::new();
        for id in parent_ids {
            let read_context = AttributeReadContext {
                component_id: Some(id),
                ..read_context
            };
            let view = ComponentView::for_context(ctx, read_context).await?;
            parents.push(veritech::ComponentView::from(view));
        }

        let qualification_view = veritech::QualificationCheckComponent {
            data: ComponentView::for_context(ctx, read_context).await?.into(),
            codes: Self::list_code_generated_by_component_id(ctx, *self.id(), system_id)
                .await?
                .into_iter()
                .flat_map(|view| {
                    let format = view.language.to_string();
                    view.code
                        .map(|code| veritech::CodeGenerated { format, code })
                })
                .collect(),
            parents,
        };
        Ok(qualification_view)
    }

    #[instrument(skip_all)]
    pub async fn list_for_schema_variant(
        ctx: &DalContext<'_, '_>,
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

    #[instrument(skip_all)]
    pub async fn list_for_resource_sync(
        txn: &PgTxn<'_>,
    ) -> ComponentResult<Vec<(Component, SystemId)>> {
        let visibility = Visibility::new_head(false);
        let rows = txn.query(LIST_FOR_RESOURCE_SYNC, &[&visibility]).await?;

        let mut results = Vec::new();
        for row in rows.into_iter() {
            let json: serde_json::Value = row.try_get("object")?;
            let object: Self = serde_json::from_value(json)?;
            let system_id: SystemId = row.try_get("system_id")?;
            results.push((object, system_id));
        }

        Ok(results)
    }

    #[instrument(skip_all)]
    pub async fn sync_resource(
        &self,
        ctx: &DalContext<'_, '_>,
        system_id: SystemId,
    ) -> ComponentResult<()> {
        // Note(paulo): we don't actually care about the Resource here, we only care about the ResourcePrototype, is this wrong?

        let schema = self
            .schema(ctx)
            .await?
            .ok_or(ComponentError::SchemaNotFound)?;
        let schema_variant = self
            .schema_variant(ctx)
            .await?
            .ok_or(ComponentError::SchemaVariantNotFound)?;

        let resource_prototype = ResourcePrototype::get_for_component(
            ctx,
            *self.id(),
            *schema.id(),
            *schema_variant.id(),
            system_id,
        )
        .await?;

        if let Some(prototype) = resource_prototype {
            let func = Func::get_by_id(ctx, &prototype.func_id())
                .await?
                .ok_or_else(|| ComponentError::MissingFunc(prototype.func_id().to_string()))?;

            let args = FuncBackendJsResourceSyncArgs {
                component: self
                    .veritech_resource_sync_component(ctx, system_id)
                    .await?,
            };

            let (func_binding, _created) = FuncBinding::find_or_create(
                ctx,
                serde_json::to_value(args)?,
                prototype.func_id(),
                *func.backend_kind(),
            )
            .await?;

            // Note: We need to execute the same func binding a bunch of times
            func_binding.execute(ctx).await?;

            // Note for future humans - if this isn't a built in, then we need to
            // think about execution time. Probably higher up than this? But just
            // an FYI.
            let existing_resolver =
                ResourceResolver::get_for_prototype_and_component(ctx, prototype.id(), self.id())
                    .await?;

            // If we do not have one, create the resource resolver. If we do, update the
            // func binding id to point to the new value.
            let mut resolver = if let Some(resolver) = existing_resolver {
                resolver
            } else {
                let mut resolver_context = ResourceResolverContext::new();
                resolver_context.set_component_id(*self.id());
                ResourceResolver::new(
                    ctx,
                    *prototype.id(),
                    *func.id(),
                    *func_binding.id(),
                    resolver_context,
                )
                .await?
            };
            resolver
                .set_func_binding_id(ctx, *func_binding.id())
                .await?;
        }

        WsEvent::resource_synced(
            *self.id(),
            system_id,
            ctx.read_tenancy().billing_accounts().into(),
            ctx.history_actor(),
        )
        .publish(ctx.txns().nats())
        .await?;

        Ok(())
    }

    // Note: Won't work for arrays and maps
    #[instrument(skip_all)]
    pub async fn set_value_by_json_pointer<T: Serialize + std::fmt::Debug + std::clone::Clone>(
        &self,
        ctx: &DalContext<'_, '_>,
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
            .ok_or(ComponentError::SchemaVariantNotFound)?;
        let schema = schema_variant
            .schema(ctx)
            .await?
            .ok_or(ComponentError::SchemaNotFound)?;

        let attribute_context = AttributeContext::builder()
            .set_component_id(*self.id())
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
        ctx: &DalContext<'_, '_>,
        json_pointer: &str,
    ) -> ComponentResult<Option<Prop>> {
        let schema_variant = self
            .schema_variant(ctx)
            .await?
            .ok_or(ComponentError::SchemaVariantNotFound)?;

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
        ctx: &DalContext<'_, '_>,
        json_pointer: &str,
    ) -> ComponentResult<Option<AttributeValue>> {
        if let Some(prop) = self.find_prop_by_json_pointer(ctx, json_pointer).await? {
            let schema = self
                .schema(ctx)
                .await?
                .ok_or(ComponentError::SchemaNotFound)?;
            let schema_variant = self
                .schema_variant(ctx)
                .await?
                .ok_or(ComponentError::SchemaVariantNotFound)?;

            // System will be unset since this method should only be used when creating a component.
            let read_context = AttributeReadContext {
                prop_id: Some(*prop.id()),
                schema_id: Some(*schema.id()),
                schema_variant_id: Some(*schema_variant.id()),
                component_id: Some(*self.id()),
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
        ctx: &DalContext<'_, '_>,
        json_pointer: &str,
    ) -> ComponentResult<Option<T>> {
        if let Some(attribute_value) = self
            .find_attribute_value_by_json_pointer(ctx, json_pointer)
            .await?
        {
            if let Some(func_binding_return_value) = FuncBindingReturnValue::get_by_id(
                ctx,
                &attribute_value.func_binding_return_value_id(),
            )
            .await?
            {
                return Ok(func_binding_return_value
                    .value()
                    .cloned()
                    .map(serde_json::from_value)
                    .transpose()?);
            };
        };

        Ok(None)
    }
}

#[must_use]
#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
pub struct ComponentAsyncTasks {
    pub component_id: ComponentId,
    pub system_id: SystemId,
    // Allows running only one specific qualification
    qualification_prototype_id: Option<QualificationPrototypeId>,
}

impl ComponentAsyncTasks {
    // Don't call this directly, call Component::build_async_tasks
    fn new(component_id: ComponentId, system_id: SystemId) -> Self {
        Self {
            component_id,
            system_id,
            qualification_prototype_id: None,
        }
    }

    pub fn set_qualification_prototype_id(&mut self, id: QualificationPrototypeId) {
        self.qualification_prototype_id = Some(id);
    }

    pub async fn run_in_ctx(self, ctx: &DalContext<'_, '_>) -> ComponentResult<()> {
        let component = Component::get_by_id(ctx, &self.component_id)
            .await?
            .ok_or(ComponentError::NotFound(self.component_id))?;

        if let Some(prototype_id) = self.qualification_prototype_id {
            component
                .check_qualification(ctx, self.system_id, prototype_id)
                .await?;

            return Ok(());
        }

        component.generate_code(ctx, self.system_id).await?;
        // Some qualifications depend on code generation, so remember to generate the code first
        component.check_qualifications(ctx, self.system_id).await?;
        Ok(())
    }

    pub async fn run(
        self,
        access_builder: AccessBuilder,
        visibility: Visibility,
        ctx_builder: &DalContextBuilder,
    ) -> ComponentResult<()> {
        if let Some(prototype_id) = self.qualification_prototype_id {
            self.run_qualification_check(access_builder, visibility, ctx_builder, prototype_id)
                .await?;

            return Ok(());
        }

        self.run_code_generation(access_builder.clone(), visibility, ctx_builder)
            .await?;
        // Some qualifications depend on code generation, so remember to generate the code first
        self.run_qualifications_check(access_builder, visibility, ctx_builder)
            .await?;
        Ok(())
    }

    async fn run_code_generation(
        &self,
        access_builder: AccessBuilder,
        visibility: Visibility,
        ctx_builder: &DalContextBuilder,
    ) -> ComponentResult<()> {
        let mut txns = ctx_builder.transactions_starter().await?;
        let txns = txns.start().await?;
        let ctx = ctx_builder.build(access_builder.build(visibility), &txns);
        let component = Component::get_by_id(&ctx, &self.component_id)
            .await?
            .ok_or(ComponentError::NotFound(self.component_id))?;
        component.generate_code(&ctx, self.system_id).await?;
        txns.commit().await?;
        Ok(())
    }

    async fn run_qualification_check(
        &self,
        access_builder: AccessBuilder,
        visibility: Visibility,
        ctx_builder: &DalContextBuilder,
        prototype_id: QualificationPrototypeId,
    ) -> ComponentResult<()> {
        let mut txns = ctx_builder.transactions_starter().await?;
        let txns = txns.start().await?;
        let ctx = ctx_builder.build(access_builder.build(visibility), &txns);
        let component = Component::get_by_id(&ctx, &self.component_id)
            .await?
            .ok_or(ComponentError::NotFound(self.component_id))?;
        component
            .check_qualification(&ctx, self.system_id, prototype_id)
            .await?;
        txns.commit().await?;
        Ok(())
    }

    async fn run_qualifications_check(
        &self,
        access_builder: AccessBuilder,
        visibility: Visibility,
        ctx_builder: &DalContextBuilder,
    ) -> ComponentResult<()> {
        let mut txns = ctx_builder.transactions_starter().await?;
        let txns = txns.start().await?;
        let ctx = ctx_builder.build(access_builder.build(visibility), &txns);
        let component = Component::get_by_id(&ctx, &self.component_id)
            .await?
            .ok_or(ComponentError::NotFound(self.component_id))?;
        component.check_qualifications(&ctx, self.system_id).await?;
        txns.commit().await?;
        Ok(())
    }
}
