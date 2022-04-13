pub mod view;

pub use view::{ComponentView, ComponentViewError};

use async_recursion::async_recursion;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use si_data::{NatsError, PgError, PgTxn};
use strum_macros::{AsRefStr, Display, EnumIter, EnumString};
use telemetry::prelude::*;
use thiserror::Error;

use crate::attribute::value::AttributeValue;
use crate::attribute::{context::UNSET_ID_VALUE, value::AttributeValueError};
use crate::code_generation_resolver::CodeGenerationResolverContext;
use crate::edit_field::{
    widget::prelude::*, EditField, EditFieldAble, EditFieldBaggage, EditFieldError,
    EditFieldObjectKind,
};
use crate::func::backend::validation::{FuncBackendValidateStringValueArgs, ValidationError};
use crate::func::backend::{
    js_code_generation::FuncBackendJsCodeGenerationArgs,
    js_qualification::FuncBackendJsQualificationArgs, js_resource::FuncBackendJsResourceSyncArgs,
};
use crate::func::binding::{FuncBinding, FuncBindingError};
use crate::func::binding_return_value::FuncBindingReturnValue;
use crate::qualification::QualificationView;
use crate::qualification_resolver::QualificationResolverContext;
use crate::resource_resolver::ResourceResolverContext;
use crate::schema::variant::{SchemaVariantError, SchemaVariantId};
use crate::schema::SchemaVariant;
use crate::validation_resolver::ValidationResolverContext;
use crate::ws_event::{WsEvent, WsEventError};
use crate::AttributeValueId;
use crate::{
    edit_field, impl_standard_model, node::NodeId, pk, qualification::QualificationError,
    standard_model, standard_model_accessor, standard_model_belongs_to, standard_model_has_many,
    AttributeContext, AttributeContextBuilderError, AttributeContextError, AttributeReadContext,
    CodeGenerationPrototype, CodeGenerationPrototypeError, CodeGenerationResolver,
    CodeGenerationResolverError, DalContext, Edge, EdgeError, Func, FuncBackendKind,
    HistoryEventError, LabelEntry, LabelList, Node, NodeError, OrganizationError, Prop, PropError,
    PropId, PropKind, QualificationPrototype, QualificationPrototypeError, QualificationResolver,
    QualificationResolverError, ReadTenancyError, Resource, ResourceError, ResourcePrototype,
    ResourcePrototypeError, ResourceResolver, ResourceResolverError, ResourceView, Schema,
    SchemaError, SchemaId, Secret, StandardModel, StandardModelError, System, SystemId, Timestamp,
    ValidationPrototype, ValidationPrototypeError, ValidationResolver, ValidationResolverError,
    Visibility, WorkspaceError, WriteTenancy,
};

#[derive(Error, Debug)]
pub enum ComponentError {
    #[error("AttributeContext error: {0}")]
    AttributeContext(#[from] AttributeContextError),
    #[error("AttributeContextBuilder error: {0}")]
    AttributeContextBuilder(#[from] AttributeContextBuilderError),
    #[error("AttributeValue error: {0}")]
    AttributeValue(#[from] AttributeValueError),
    #[error("edit field error: {0}")]
    EditField(#[from] EditFieldError),
    #[error("edge error: {0}")]
    Edge(#[from] EdgeError),
    #[error("missing attribute value for id: ({0})")]
    MissingAttributeValue(AttributeValueId),
    #[error("Missing IndexMap on AttributeValue: {0}")]
    MissingIndexMap(AttributeValueId),
    #[error("expected one root prop, found multiple: {0:?}")]
    MultipleRootProps(Vec<Prop>),
    #[error("root prop not found for schema variant: {0}")]
    RootPropNotFound(SchemaVariantId),

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
    #[error("error serializing/deserializing json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
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
    #[error("billing account not found")]
    BillingAccountNotFound,
    #[error("ws event error: {0}")]
    WsEvent(#[from] WsEventError),
    #[error("workspace error: {0}")]
    Workspace(#[from] WorkspaceError),
    #[error("organization error: {0}")]
    Organization(#[from] OrganizationError),
    #[error("invalid json pointer: {0} for {1}")]
    BadJsonPointer(String, String),
    #[error("invalid AttributeReadContext: {0}")]
    BadAttributeReadContext(String),
}

pub type ComponentResult<T> = Result<T, ComponentError>;

const GET_RESOURCE: &str = include_str!("./queries/component_get_resource.sql");
const LIST_QUALIFICATIONS: &str = include_str!("./queries/component_list_qualifications.sql");
const LIST_CODE_GENERATED: &str = include_str!("./queries/component_list_code_generated.sql");
const LIST_FOR_RESOURCE_SYNC: &str = include_str!("./queries/component_list_for_resource_sync.sql");

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
    #[allow(clippy::too_many_arguments)]
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

    #[allow(clippy::too_many_arguments)]
    #[instrument(skip_all)]
    pub async fn new_for_schema_variant_with_node(
        ctx: &DalContext<'_, '_>,
        name: impl AsRef<str>,
        schema_variant_id: &SchemaVariantId,
    ) -> ComponentResult<(Self, Node)> {
        // TODO: Eventually, we'll need the logic to be more complex than stuffing everything into the "production" system, but that's a problem for "a week or two from now" us.
        let mut systems = System::find_by_attr(ctx, "name", &"production").await?;
        let system = systems.pop().ok_or(ComponentError::SystemNotFound)?;
        Self::new_for_schema_variant_with_node_in_system(ctx, name, schema_variant_id, system.id())
            .await
    }

    #[allow(clippy::too_many_arguments)]
    #[instrument(skip_all)]
    pub async fn new_for_schema_variant_with_node_in_deployment(
        ctx: &DalContext<'_, '_>,
        name: impl AsRef<str>,
        schema_variant_id: &SchemaVariantId,
        system_id: &SystemId,
        parent_node_id: &NodeId,
    ) -> ComponentResult<(Self, Node)> {
        let (component, node) = Self::new_for_schema_variant_with_node_in_system(
            ctx,
            name,
            schema_variant_id,
            system_id,
        )
        .await?;
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

    #[allow(clippy::too_many_arguments)]
    #[instrument(skip_all)]
    pub async fn new_for_schema_variant_with_node_in_system(
        ctx: &DalContext<'_, '_>,
        name: impl AsRef<str>,
        schema_variant_id: &SchemaVariantId,
        system_id: &SystemId,
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

        // NOTE: We may want to be a bit smarter about when we create the Resource
        //       at some point in the future, by only creating it if there is also
        //       a ResourcePrototype for the Component's SchemaVariant.
        let _resource = Resource::new(ctx, component.id(), system_id).await?;

        let name: &str = name.as_ref();
        component
            .set_value_by_json_pointer(ctx, "/root/si/name", Some(name))
            .await?;
        let _edge = Edge::include_component_in_system(
            ctx,
            component.id(),
            &schema.kind().into(),
            system_id,
        )
        .await?;

        Ok((component, node))
    }

    #[instrument(skip_all)]
    #[allow(clippy::too_many_arguments)]
    pub async fn new_application_with_node(
        ctx: &DalContext<'_, '_>,
        name: impl AsRef<str>,
    ) -> ComponentResult<(Self, Node)> {
        let schema_variant_id =
            Schema::default_schema_variant_id_for_name(ctx, "application").await?;

        let (component, node) =
            Self::new_for_schema_variant_with_node(ctx, name, &schema_variant_id).await?;
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

    // FIXME(nick): this should eventually become "update_from_edit_field", but we are going to
    // maintain the old SDF logic for now since this code is in flux.
    #[allow(clippy::too_many_arguments)]
    #[instrument(skip_all)]
    pub async fn update_from_edit_field_with_baggage(
        ctx: &DalContext<'_, '_>,
        value: Option<serde_json::Value>,
        attribute_context: AttributeContext,
        baggage: EditFieldBaggage,
    ) -> ComponentResult<()> {
        let (updated_value, _updated_attribute_value_id) = AttributeValue::update_for_context(
            ctx,
            baggage.attribute_value_id,
            baggage.parent_attribute_value_id,
            attribute_context,
            value,
            baggage.key,
        )
        .await?;

        // Check validations and qualifications for our component.
        let component = Self::get_by_id(ctx, &attribute_context.component_id())
            .await?
            .ok_or_else(|| ComponentError::NotFound(attribute_context.component_id()))?;
        let prop = Prop::get_by_id(ctx, &attribute_context.prop_id())
            .await?
            .ok_or_else(|| ComponentError::MissingProp(attribute_context.prop_id()))?;
        component
            .check_validations(ctx, &prop, &updated_value, false)
            .await?;

        // Some qualifications depend on code generation, so we have to generate first
        component
            .generate_code(ctx, attribute_context.system_id())
            .await?;
        component
            .check_qualifications(ctx, attribute_context.system_id())
            .await?;

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn check_validations(
        &self,
        ctx: &DalContext<'_, '_>,
        prop: &Prop,
        value: &Option<serde_json::Value>,
        created: bool,
    ) -> ComponentResult<()> {
        let validators =
            ValidationPrototype::find_for_prop(ctx, *prop.id(), UNSET_ID_VALUE.into()).await?;

        for validator in validators {
            let func = Func::get_by_id(ctx, &validator.func_id())
                .await?
                .ok_or_else(|| ComponentError::MissingFunc(validator.func_id().to_string()))?;
            let func_binding = match func.backend_kind() {
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
                        return Err(ComponentError::MissingProp(*prop.id()));
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
                    func_binding
                }
                kind => unimplemented!("Validator Backend not supported yet: {}", kind),
            };

            if created {
                let mut existing_validation_resolvers =
                    ValidationResolver::find_for_prototype(ctx, validator.id()).await?;

                // If we don't have one, create the validation resolver. If we do, update the
                // func binding id to point to the new value. Interesting to think about
                // garbage collecting the left over funcbinding + func result value?
                if let Some(mut validation_resolver) = existing_validation_resolvers.pop() {
                    validation_resolver
                        .set_func_binding_id(ctx, *func_binding.id())
                        .await?;
                } else {
                    let mut validation_resolver_context = ValidationResolverContext::new();
                    validation_resolver_context.set_prop_id(*prop.id());
                    validation_resolver_context.set_component_id(*self.id());
                    ValidationResolver::new(
                        ctx,
                        *validator.id(),
                        *func.id(),
                        *func_binding.id(),
                        validation_resolver_context,
                    )
                    .await?;
                }
            }
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn check_qualifications(
        &self,
        ctx: &DalContext<'_, '_>,
        system_id: SystemId,
    ) -> ComponentResult<()> {
        let schema = self
            .schema_with_tenancy(ctx)
            .await?
            .ok_or(ComponentError::SchemaNotFound)?;
        let schema_variant = self
            .schema_variant_with_tenancy(ctx)
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
            let (func_binding, created) = FuncBinding::find_or_create(
                ctx,
                json_args,
                prototype.func_id(),
                *func.backend_kind(),
            )
            .await?;

            if created {
                // Note for future humans - if this isn't a built in, then we need to
                // think about execution time. Probably higher up than this? But just
                // an FYI.
                func_binding.execute(ctx).await?;

                let mut existing_resolvers =
                    QualificationResolver::find_for_prototype_and_component(
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
            }
        }

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn generate_code(
        &self,
        ctx: &DalContext<'_, '_>,
        system_id: SystemId,
    ) -> ComponentResult<()> {
        let schema = self
            .schema_with_tenancy(ctx)
            .await?
            .ok_or(ComponentError::SchemaNotFound)?;
        let schema_variant = self
            .schema_variant_with_tenancy(ctx)
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

            let (func_binding, created) = FuncBinding::find_or_create(
                ctx,
                json_args,
                prototype.func_id(),
                *func.backend_kind(),
            )
            .await?;

            if created {
                // Note for future humans - if this isn't a built in, then we need to
                // think about execution time. Probably higher up than this? But just
                // an FYI.
                func_binding.execute(ctx).await?;

                let mut existing_resolvers =
                    CodeGenerationResolver::find_for_prototype_and_component(
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
        }

        Ok(())
    }

    #[instrument(skip_all)]
    pub async fn list_validations_as_qualification_for_component_id(
        ctx: &DalContext<'_, '_>,
        component_id: ComponentId,
        system_id: SystemId,
    ) -> ComponentResult<QualificationView> {
        let validation_field_values =
            ValidationResolver::list_values_for_component(ctx, component_id, system_id).await?;

        let mut validation_errors: Vec<(Prop, Vec<ValidationError>)> = Vec::new();
        for (prop, field_value) in validation_field_values.into_iter() {
            if let Some(value_json) = field_value.value() {
                // This clone shouldn't be necessary, but we have no way to get to the owned value -- Adam
                let internal_validation_errors: Vec<ValidationError> =
                    serde_json::from_value(value_json.clone())?;
                validation_errors.push((prop, internal_validation_errors));
            }
        }
        let qualification_view = QualificationView::new_for_validation_errors(validation_errors);
        Ok(qualification_view)
    }

    #[instrument(skip_all)]
    pub async fn list_code_generated_by_component_id(
        ctx: &DalContext<'_, '_>,
        component_id: ComponentId,
        system_id: SystemId,
    ) -> ComponentResult<Vec<veritech::CodeGenerated>> {
        let mut results: Vec<veritech::CodeGenerated> = Vec::new();

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
        for row in rows.into_iter() {
            let json: serde_json::Value = row.try_get("object")?;
            let func_binding_return_value: FuncBindingReturnValue = serde_json::from_value(json)?;
            let value = func_binding_return_value
                .value()
                .ok_or(ComponentError::CodeGeneratedNotFound)?;
            let code_generated = veritech::CodeGenerated::deserialize(value)?;
            results.push(code_generated);
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
            let mut qual_view = QualificationView::new_for_func_binding_return_value(
                ctx,
                func_binding_return_value,
            )
            .await?;
            let title: String = row.try_get("title")?;
            let link: Option<String> = row.try_get("link")?;
            qual_view.title = title;
            qual_view.link = link;
            results.push(qual_view);
        }
        // This is inefficient, but effective
        if no_qualification_results {
            let component = Self::get_by_id(ctx, &component_id)
                .await?
                .ok_or(ComponentError::NotFound(component_id))?;
            let schema = component
                .schema_with_tenancy(ctx)
                .await?
                .ok_or(ComponentError::SchemaNotFound)?;
            let schema_variant = component
                .schema_variant_with_tenancy(ctx)
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

    pub async fn veritech_attribute_resolver_component(
        &self,
        ctx: &DalContext<'_, '_>,
        system_id: SystemId,
    ) -> ComponentResult<veritech::ResolverFunctionComponent> {
        let schema = self
            .schema(ctx)
            .await?
            .ok_or(ComponentError::SchemaNotFound)?;
        let schema_variant = self
            .schema_variant(ctx)
            .await?
            .ok_or(ComponentError::SchemaVariantNotFound)?;
        let attribute_context_base = AttributeReadContext {
            schema_id: Some(*schema.id()),
            schema_variant_id: Some(*schema_variant.id()),
            component_id: Some(*self.id()),
            system_id: Some(system_id),
            ..AttributeReadContext::any()
        };

        let parent_ids = Edge::find_component_configuration_parents(ctx, self.id()).await?;
        let mut parents = Vec::with_capacity(parent_ids.len());
        for id in parent_ids {
            let component = Component::get_by_id(ctx, &id)
                .await?
                .ok_or(ComponentError::NotFound(id))?;
            let schema = component
                .schema_with_tenancy(ctx)
                .await?
                .ok_or(ComponentError::SchemaNotFound)?;
            let schema_variant = component
                .schema_variant_with_tenancy(ctx)
                .await?
                .ok_or(ComponentError::SchemaVariantNotFound)?;
            let read_context = AttributeReadContext {
                schema_id: Some(*schema.id()),
                schema_variant_id: Some(*schema_variant.id()),
                component_id: Some(id),
                ..attribute_context_base
            };
            let view = ComponentView::for_context(ctx, read_context).await?;
            parents.push(veritech::ComponentView::from(view));
        }

        let schema = self
            .schema_with_tenancy(ctx)
            .await?
            .ok_or(ComponentError::SchemaNotFound)?;
        let schema_variant = self
            .schema_variant_with_tenancy(ctx)
            .await?
            .ok_or(ComponentError::SchemaVariantNotFound)?;
        let read_context = AttributeReadContext {
            schema_id: Some(*schema.id()),
            schema_variant_id: Some(*schema_variant.id()),
            component_id: Some(*self.id()),
            ..attribute_context_base
        };
        let component = veritech::ResolverFunctionComponent {
            data: ComponentView::for_context(ctx, read_context).await?.into(),
            parents,
        };
        Ok(component)
    }

    pub async fn veritech_code_generation_component(
        &self,
        ctx: &DalContext<'_, '_>,
        system_id: SystemId,
    ) -> ComponentResult<ComponentView> {
        let schema = self
            .schema_with_tenancy(ctx)
            .await?
            .ok_or(ComponentError::SchemaNotFound)?;
        let schema_variant = self
            .schema_variant_with_tenancy(ctx)
            .await?
            .ok_or(ComponentError::SchemaVariantNotFound)?;
        let attribute_context = AttributeReadContext {
            schema_id: Some(*schema.id()),
            schema_variant_id: Some(*schema_variant.id()),
            component_id: Some(*self.id()),
            system_id: Some(system_id),
            ..AttributeReadContext::any()
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
            .schema_with_tenancy(ctx)
            .await?
            .ok_or(ComponentError::SchemaNotFound)?;
        let schema_variant = self
            .schema_variant_with_tenancy(ctx)
            .await?
            .ok_or(ComponentError::SchemaVariantNotFound)?;
        let attribute_context = AttributeReadContext {
            schema_id: Some(*schema.id()),
            schema_variant_id: Some(*schema_variant.id()),
            component_id: Some(*self.id()),
            system_id: Some(system_id),
            ..AttributeReadContext::any()
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
        let base_attribute_context = AttributeReadContext {
            schema_id: Some(*schema.id()),
            schema_variant_id: Some(*schema_variant.id()),
            component_id: Some(*self.id()),
            system_id: Some(system_id),
            ..AttributeReadContext::any()
        };

        let parent_ids = Edge::find_component_configuration_parents(ctx, self.id()).await?;

        let mut parents = Vec::new();
        for id in parent_ids {
            let read_context = AttributeReadContext {
                component_id: Some(id),
                ..base_attribute_context
            };
            let view = ComponentView::for_context(ctx, read_context).await?;
            parents.push(veritech::ComponentView::from(view));
        }

        let read_context = AttributeReadContext {
            component_id: Some(*self.id()),
            ..base_attribute_context
        };
        let qualification_view = veritech::QualificationCheckComponent {
            data: ComponentView::for_context(ctx, read_context).await?.into(),
            codes: Self::list_code_generated_by_component_id(ctx, *self.id(), system_id).await?,
            parents,
        };
        Ok(qualification_view)
    }

    #[instrument(skip_all)]
    pub async fn list_for_resource_sync(txn: &PgTxn<'_>) -> ComponentResult<Vec<Component>> {
        let visibility = Visibility::new_head(false);
        let rows = txn.query(LIST_FOR_RESOURCE_SYNC, &[&visibility]).await?;
        let results = standard_model::objects_from_rows(rows)?;
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
            .schema_with_tenancy(ctx)
            .await?
            .ok_or(ComponentError::SchemaNotFound)?;
        let schema_variant = self
            .schema_variant_with_tenancy(ctx)
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
    #[allow(clippy::too_many_arguments)]
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
            .schema_variant_with_tenancy(ctx)
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

        AttributeValue::update_for_context(
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
            .schema_variant_with_tenancy(ctx)
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
            // Should we specify system, schema, schema_variant here?
            let read_context = AttributeReadContext {
                prop_id: Some(*prop.id()),
                component_id: Some(*self.id()),
                ..AttributeReadContext::any()
            };

            return Ok(Some(
                AttributeValue::find_for_context(ctx, read_context)
                    .await?
                    .pop()
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
            if let Some(fbrv) = FuncBindingReturnValue::get_by_id(
                ctx,
                &attribute_value.func_binding_return_value_id(),
            )
            .await?
            {
                return Ok(fbrv
                    .value()
                    .cloned()
                    .map(serde_json::from_value)
                    .transpose()?);
            };
        };

        Ok(None)
    }
}

#[async_trait]
impl EditFieldAble for Component {
    type Id = ComponentId;
    type Error = ComponentError;

    async fn get_edit_fields(
        ctx: &DalContext<'_, '_>,
        id: &ComponentId,
    ) -> ComponentResult<Vec<EditField>> {
        let head_visibility = Visibility::new_head(ctx.visibility().deleted);
        let change_set_visibility =
            Visibility::new_change_set(ctx.visibility().change_set_pk, ctx.visibility().deleted);

        let component = Self::get_by_id(ctx, id)
            .await?
            .ok_or(ComponentError::NotFound(*id))?;

        let mut edit_fields = vec![];
        let schema_variant = component
            .schema_variant_with_tenancy(ctx)
            .await?
            .ok_or(ComponentError::SchemaVariantNotFound)?;
        let schema = schema_variant
            .schema(ctx)
            .await?
            .ok_or(ComponentError::SchemaNotFound)?;

        let attribute_read_context = AttributeReadContext {
            prop_id: None,
            schema_id: Some(*schema.id()),
            schema_variant_id: Some(*schema_variant.id()),
            component_id: Some(*id),
            ..AttributeReadContext::default()
        };

        // NOTE(nick): this can be more elegant, but it works. We want to ensure we only find the
        // root prop at this point.
        let mut props = schema_variant.props(ctx).await?;
        if props.len() > 1 {
            return Err(ComponentError::MultipleRootProps(props));
        }
        let root_prop = props
            .pop()
            .ok_or_else(|| ComponentError::RootPropNotFound(*schema_variant.id()))?;

        // NOTE(nick): it is a bit wasteful that we get the attribute value here and then do it
        // again within the call below, but it ~~works~~.
        let context = AttributeReadContext {
            prop_id: Some(*root_prop.id()),
            ..attribute_read_context
        };
        let attribute_value: AttributeValue = AttributeValue::find_for_context(ctx, context)
            .await?
            .pop()
            .ok_or_else(|| ComponentError::RootPropNotFound(*schema_variant.id()))?;

        // Parent attribute value must be "None" since we are dealing with the root prop.
        edit_fields.push(
            edit_field_for_attribute_value(
                ctx,
                &head_visibility,
                &change_set_visibility,
                attribute_read_context,
                *attribute_value.id(),
                None,
                None,
            )
            .await?,
        );

        Ok(edit_fields)
    }

    async fn update_from_edit_field(
        _ctx: &DalContext<'_, '_>,
        _id: Self::Id,
        edit_field_id: String,
        _value: Option<serde_json::Value>,
    ) -> ComponentResult<()> {
        Err(EditFieldError::invalid_field(edit_field_id).into())
    }
}

#[allow(clippy::too_many_arguments)]
#[async_recursion]
async fn edit_field_for_attribute_value(
    ctx: &DalContext<'_, '_>,
    head_visibility: &Visibility,
    change_set_visibility: &Visibility,
    attribute_read_context: AttributeReadContext,
    attribute_value_id: AttributeValueId,
    parent_attribute_value_id: Option<AttributeValueId>,
    edit_field_path: Option<Vec<String>>,
) -> ComponentResult<EditField> {
    let head_ctx = ctx.clone_with_new_visibility(*head_visibility);
    let change_set_ctx = ctx.clone_with_new_visibility(*change_set_visibility);
    let attribute_value: AttributeValue = AttributeValue::get_by_id(ctx, &attribute_value_id)
        .await?
        .ok_or(ComponentError::MissingAttributeValue(attribute_value_id))?;
    let prop = AttributeValue::find_prop_for_value(ctx, *attribute_value.id()).await?;

    let field_name = prop.name();
    let object_kind = EditFieldObjectKind::ComponentProp;

    // Gather the three values we need for visibility diff. For the head and change set values, we
    // need to use their respective visibilites for attribute value searches, but can use the
    // standard visibility for getting the func binding return value by id.
    let current_func_binding_return_value =
        FuncBindingReturnValue::get_by_id(ctx, &attribute_value.func_binding_return_value_id())
            .await?;
    let head_func_binding_return_value = if ctx.visibility().in_change_set() {
        if let Some(found_value) = AttributeValue::find_with_parent_and_key_for_context(
            &head_ctx,
            parent_attribute_value_id,
            attribute_value.key.clone(),
            attribute_value.context,
        )
        .await?
        {
            FuncBindingReturnValue::get_by_id(
                &head_ctx,
                &found_value.func_binding_return_value_id(),
            )
            .await?
        } else {
            None
        }
    } else {
        None
    };
    let change_set_func_binding_return_value = if ctx.visibility().in_edit_session() {
        if let Some(found_value) = AttributeValue::find_with_parent_and_key_for_context(
            &change_set_ctx,
            parent_attribute_value_id,
            attribute_value.key.clone(),
            attribute_value.context,
        )
        .await?
        {
            FuncBindingReturnValue::get_by_id(
                &change_set_ctx,
                &found_value.func_binding_return_value_id(),
            )
            .await?
        } else {
            None
        }
    } else {
        None
    };

    fn extract_value(
        func_binding_return_value: &FuncBindingReturnValue,
    ) -> Option<&serde_json::Value> {
        func_binding_return_value.value()
    }

    let (value, visibility_diff) = edit_field::value_and_visibility_diff_json_option(
        ctx.visibility(),
        current_func_binding_return_value.as_ref(),
        extract_value,
        head_func_binding_return_value.as_ref(),
        change_set_func_binding_return_value.as_ref(),
    )?;

    let validation_errors = Vec::new();

    // FIXME(nick): change validation resolver query to use attribute values instead.
    //
    // let validation_field_values = ValidationResolver::find_values_for_prop_and_component(
    //     txn,
    //     &tenancy.clone_into_read_tenancy(txn).await?,
    //     visibility,
    //     *prop.id(),
    //     *component.id(),
    //     system_id,
    // )
    // .await?;
    // for field_value in validation_field_values.into_iter() {
    //     if let Some(value_json) = field_value.value() {
    //         // This clone shouldn't be necessary, but we have no way to get to the owned value -- Adam
    //         let mut validation_error: Vec<ValidationError> =
    //             serde_json::from_value(value_json.clone())?;
    //         validation_errors.append(&mut validation_error);
    //     }
    // }

    let current_edit_field_path = match edit_field_path {
        None => vec!["properties".to_owned()],
        Some(path) => path,
    };
    let mut edit_field_path_for_children = current_edit_field_path.clone();
    edit_field_path_for_children.push(field_name.to_string());

    let widget = match prop.widget_kind() {
        WidgetKind::SecretSelect => {
            let mut entries = Vec::new();
            let secrets = Secret::list(ctx).await?;

            for secret in secrets.into_iter() {
                entries.push(LabelEntry::new(
                    secret.name(),
                    serde_json::json!(i64::from(*secret.id())),
                ));
            }
            Widget::Select(SelectWidget::new(LabelList::new(entries), None))
        }
        WidgetKind::Text => Widget::Text(TextWidget::new()),
        WidgetKind::Array | WidgetKind::Header => {
            let mut child_edit_fields = vec![];
            let mut child_attribute_values = attribute_value
                .child_attribute_values_in_context(ctx, attribute_read_context)
                .await?;
            if let Some(index_map) = attribute_value.index_map() {
                let child_order = index_map.order();

                child_attribute_values.sort_by_cached_key(|av| {
                    child_order
                        .iter()
                        .position(|attribute_value_id| attribute_value_id == av.id())
                        .unwrap_or(0)
                });
            }

            for child_attribute_value in child_attribute_values {
                // Use the current attribute value as the parent when creating the child edit field.
                child_edit_fields.push(
                    edit_field_for_attribute_value(
                        ctx,
                        head_visibility,
                        change_set_visibility,
                        attribute_read_context,
                        *child_attribute_value.id(),
                        Some(attribute_value_id),
                        Some(edit_field_path_for_children.clone()),
                    )
                    .await?,
                );
            }

            #[allow(clippy::if_same_then_else)]
            if *prop.kind() == PropKind::Array {
                Widget::Array(ArrayWidget::new(vec![child_edit_fields]))
            } else if *prop.kind() == PropKind::Map {
                // This is likely not correct.
                Widget::Header(HeaderWidget::new(child_edit_fields))
            } else {
                // Only option left is PropKind::Object
                Widget::Header(HeaderWidget::new(child_edit_fields))
            }
        }
        WidgetKind::Checkbox => Widget::Checkbox(CheckboxWidget::new()),
    };

    let mut edit_field = EditField::new(
        field_name,
        current_edit_field_path,
        object_kind,
        attribute_value_id,
        (*prop.kind()).into(),
        widget,
        value,
        visibility_diff,
        validation_errors,
    );
    edit_field.set_new_baggage(
        attribute_value_id,
        parent_attribute_value_id,
        attribute_value.key,
        *prop.id(),
    );

    Ok(edit_field)
}
