use std::collections::HashMap;
use std::convert::TryFrom;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use si_data::{NatsError, NatsTxn, PgError, PgTxn};
use telemetry::prelude::*;
use thiserror::Error;

use crate::attribute_resolver::{AttributeResolverContext, UNSET_ID_VALUE};
use crate::edit_field::{
    value_and_visibility_diff, value_and_visibility_diff_json_option, EditField, EditFieldAble,
    EditFieldBaggage, EditFieldBaggageComponentProp, EditFieldDataType, EditFieldError,
    EditFieldObjectKind, EditFields, TextWidget, Widget,
};
use crate::func::backend::validation::{FuncBackendValidateStringValueArgs, ValidationError};
use crate::func::backend::{
    FuncBackendJsQualificationArgs, FuncBackendJsResourceSyncArgs, FuncBackendStringArgs,
};
use crate::func::binding::{FuncBinding, FuncBindingError};
use crate::func::binding_return_value::FuncBindingReturnValue;
use crate::node::NodeKind;
use crate::qualification::QualificationView;
use crate::qualification_resolver::QualificationResolverContext;
use crate::resource_resolver::ResourceResolverContext;
use crate::schema::variant::{SchemaVariantError, SchemaVariantId};
use crate::schema::SchemaVariant;
use crate::validation_resolver::ValidationResolverContext;
use crate::{Edge, EdgeError, System};

use crate::{
    impl_standard_model, pk, qualification::QualificationError, standard_model,
    standard_model_accessor, standard_model_belongs_to, standard_model_has_many, AttributeResolver,
    AttributeResolverError, Func, FuncBackendKind, HistoryActor, HistoryEventError, Node,
    NodeError, Prop, PropId, PropKind, QualificationPrototype, QualificationPrototypeError,
    QualificationResolver, QualificationResolverError, Resource, ResourceError, ResourcePrototype,
    ResourcePrototypeError, ResourceResolver, ResourceResolverError, ResourceView, Schema,
    SchemaError, SchemaId, StandardModel, StandardModelError, SystemId, Tenancy, Timestamp,
    ValidationPrototype, ValidationPrototypeError, ValidationResolver, ValidationResolverError,
    Visibility,
};

#[derive(Error, Debug)]
pub enum ComponentError {
    #[error("edit field error: {0}")]
    EditField(#[from] EditFieldError),
    #[error("edge error: {0}")]
    Edge(#[from] EdgeError),
    #[error("qualification prototype error: {0}")]
    QualificationPrototype(#[from] QualificationPrototypeError),
    #[error("qualification resolver error: {0}")]
    QualificationResolver(#[from] QualificationResolverError),
    #[error("resource prototype error: {0}")]
    ResourcePrototype(#[from] ResourcePrototypeError),
    #[error("resource resolver error: {0}")]
    ResourceResolver(#[from] ResourceResolverError),
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
    #[error("attribute resolver error: {0}")]
    AttributeResolver(#[from] AttributeResolverError),
    #[error("missing a prop in attribute update: {0} not found")]
    MissingProp(PropId),
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
    #[error("resource not found")]
    ResourceNotFound,
}

pub type ComponentResult<T> = Result<T, ComponentError>;

const GET_RESOURCE: &str = include_str!("./queries/component_get_resource.sql");
const LIST_QUALIFICATIONS: &str = include_str!("./queries/component_list_qualifications.sql");
const LIST_FOR_RESOURCE_SYNC: &str = include_str!("./queries/component_list_for_resource_sync.sql");

pk!(ComponentPk);
pk!(ComponentId);

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct Component {
    pk: ComponentPk,
    id: ComponentId,
    name: String,
    #[serde(flatten)]
    tenancy: Tenancy,
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
    #[tracing::instrument(skip(txn, nats, name))]
    pub async fn new(
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        tenancy: &Tenancy,
        visibility: &Visibility,
        history_actor: &HistoryActor,
        name: impl AsRef<str>,
    ) -> ComponentResult<Self> {
        let name = name.as_ref();
        let row = txn
            .query_one(
                "SELECT object FROM component_create_v1($1, $2, $3)",
                &[&tenancy, &visibility, &name],
            )
            .await?;
        let object = standard_model::finish_create_from_row(
            txn,
            nats,
            tenancy,
            visibility,
            history_actor,
            row,
        )
        .await?;
        Ok(object)
    }

    #[allow(clippy::too_many_arguments)]
    #[tracing::instrument(skip(txn, nats, name))]
    pub async fn new_for_schema_with_node(
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        veritech: veritech::Client,
        tenancy: &Tenancy,
        visibility: &Visibility,
        history_actor: &HistoryActor,
        name: impl AsRef<str>,
        schema_id: &SchemaId,
    ) -> ComponentResult<(Self, Node)> {
        let mut schema_tenancy = tenancy.clone();
        schema_tenancy.universal = true;

        let schema = Schema::get_by_id(txn, &schema_tenancy, visibility, schema_id)
            .await?
            .ok_or(ComponentError::SchemaNotFound)?;

        let schema_variant_id = schema
            .default_schema_variant_id()
            .ok_or(ComponentError::SchemaVariantNotFound)?;

        Self::new_for_schema_variant_with_node(
            txn,
            nats,
            veritech,
            tenancy,
            visibility,
            history_actor,
            name,
            schema_variant_id,
        )
        .await
    }

    #[allow(clippy::too_many_arguments)]
    #[tracing::instrument(skip(txn, nats, name))]
    pub async fn new_for_schema_variant_with_node(
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        veritech: veritech::Client,
        tenancy: &Tenancy,
        visibility: &Visibility,
        history_actor: &HistoryActor,
        name: impl AsRef<str>,
        schema_variant_id: &SchemaVariantId,
    ) -> ComponentResult<(Self, Node)> {
        let name = name.as_ref();

        let mut schema_tenancy = tenancy.clone();
        schema_tenancy.universal = true;

        let schema_variant =
            SchemaVariant::get_by_id(txn, &schema_tenancy, visibility, schema_variant_id)
                .await?
                .ok_or(ComponentError::SchemaVariantNotFound)?;
        let schema = schema_variant
            .schema(txn, visibility)
            .await?
            .ok_or(ComponentError::SchemaNotFound)?;

        let row = txn
            .query_one(
                "SELECT object FROM component_create_v1($1, $2, $3)",
                &[&tenancy, &visibility, &name],
            )
            .await?;

        let component: Component = standard_model::finish_create_from_row(
            txn,
            nats,
            tenancy,
            visibility,
            history_actor,
            row,
        )
        .await?;
        component
            .set_schema(txn, nats, visibility, history_actor, schema.id())
            .await?;
        component
            .set_schema_variant(txn, nats, visibility, history_actor, schema_variant.id())
            .await?;

        // Need to flesh out node so that the template data is also included in the node we
        // persist. But it isn't, - our node is anemic.
        let node = Node::new(
            txn,
            nats,
            tenancy,
            visibility,
            history_actor,
            &NodeKind::Component,
        )
        .await?;
        node.set_component(txn, nats, visibility, history_actor, component.id())
            .await?;

        // TODO: Eventually, we'll need the logic to be more complex than stuffing everything into the "production" system, but that's a problem for "a week or two from now" us.
        let mut systems =
            System::find_by_attr(txn, tenancy, visibility, "name", &"production").await?;
        let system = systems.pop().ok_or(ComponentError::SystemNotFound)?;
        let _edge = Edge::include_component_in_system(
            txn,
            nats,
            tenancy,
            visibility,
            history_actor,
            component.id(),
            system.id(),
        )
        .await?;

        Ok((component, node))
    }

    #[tracing::instrument(skip(txn, nats, name))]
    pub async fn new_application_with_node(
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        veritech: veritech::Client,
        tenancy: &Tenancy,
        visibility: &Visibility,
        history_actor: &HistoryActor,
        name: impl AsRef<str>,
    ) -> ComponentResult<(Self, Node)> {
        let universal_tenancy = Tenancy::new_universal();

        let schema_variant_id = Schema::default_schema_variant_id_for_name(
            txn,
            &universal_tenancy,
            visibility,
            "application",
        )
        .await?;

        let (component, node) = Component::new_for_schema_variant_with_node(
            txn,
            nats,
            veritech,
            tenancy,
            visibility,
            history_actor,
            name,
            &schema_variant_id,
        )
        .await?;
        Ok((component, node))
    }

    standard_model_accessor!(name, String, ComponentResult);

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

    fn name_edit_field(
        visibility: &Visibility,
        object: &Self,
        head_object: &Option<Self>,
        change_set_object: &Option<Self>,
    ) -> ComponentResult<EditField> {
        let field_name = "name";
        let target_fn = Self::name;
        let object_kind = EditFieldObjectKind::Component;

        let (value, visibility_diff) = value_and_visibility_diff(
            visibility,
            Some(object),
            target_fn,
            head_object.as_ref(),
            change_set_object.as_ref(),
        )?;

        Ok(EditField::new(
            field_name,
            vec![],
            object_kind,
            object.id,
            EditFieldDataType::String,
            Widget::Text(TextWidget::new()),
            value,
            visibility_diff,
            vec![ValidationError {
                message: "Aieeee! This should appear in the name field.".to_string(),
                link: Some("https://placekitten.com".to_string()),
                ..ValidationError::default()
            }], // TODO: actually validate to generate ValidationErrors
        ))
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn update_prop_from_edit_field(
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        veritech: veritech::Client,
        tenancy: &Tenancy,
        visibility: &Visibility,
        history_actor: &HistoryActor,
        component_id: ComponentId,
        prop_id: PropId,
        _edit_field_id: String,
        value: Option<serde_json::Value>,
    ) -> ComponentResult<()> {
        let prop = Prop::get_by_id(txn, tenancy, visibility, &prop_id)
            .await?
            .ok_or(ComponentError::MissingProp(prop_id))?;
        let component = Component::get_by_id(txn, tenancy, visibility, &component_id)
            .await?
            .ok_or(ComponentError::NotFound(component_id))?;
        // We shouldn't be leaking this value, because it may or may not be actually set. But
        // when you YOLO, YOLO hard. -- Adam
        let (func, func_binding, created, value) = match (prop.kind(), value) {
            (PropKind::String, Some(value_json)) => {
                let value = if !value_json.is_string() {
                    return Err(ComponentError::InvalidPropValue(
                        "String".to_string(),
                        value_json,
                    ));
                } else {
                    value_json.as_str().unwrap().to_string()
                };
                let func_name = "si:setString".to_string();
                let mut funcs =
                    Func::find_by_attr(txn, tenancy, visibility, "name", &func_name).await?;
                let func = funcs.pop().ok_or(ComponentError::MissingFunc(func_name))?;
                let func_backend_string_args =
                    serde_json::to_value(FuncBackendStringArgs::new(value.clone()))?;
                let (func_binding, created) = FuncBinding::find_or_create(
                    txn,
                    nats,
                    tenancy,
                    visibility,
                    history_actor,
                    func_backend_string_args,
                    *func.id(),
                    *func.backend_kind(),
                )
                .await?;

                // Note for future humans - if this isn't a built in, then we need to
                // think about execution time. Probably higher up than this? But just
                // an FYI.
                if created {
                    func_binding.execute(txn, nats, veritech.clone()).await?;
                }
                (func, func_binding, created, value)
            }
            (PropKind::String, None) => {
                todo!("we haven't dealt with unseting a string");
            }
        };

        let mut attribute_resolver_context = AttributeResolverContext::new();
        attribute_resolver_context.set_prop_id(prop_id);
        attribute_resolver_context.set_component_id(component_id);
        AttributeResolver::upsert(
            txn,
            nats,
            tenancy,
            visibility,
            history_actor,
            *func.id(),
            *func_binding.id(),
            attribute_resolver_context,
        )
        .await?;

        component
            .check_validations(
                txn,
                nats,
                veritech.clone(),
                tenancy,
                visibility,
                history_actor,
                &prop,
                &value,
                created,
            )
            .await?;

        component
            .check_qualifications(
                txn,
                nats,
                veritech,
                tenancy,
                visibility,
                history_actor,
                UNSET_ID_VALUE.into(),
            )
            .await?;

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn check_validations(
        &self,
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        veritech: veritech::Client,
        tenancy: &Tenancy,
        visibility: &Visibility,
        history_actor: &HistoryActor,
        prop: &Prop,
        value: &str, // This is obviously wrong, as we don't know what kind of value this is.
        created: bool,
    ) -> ComponentResult<()> {
        let validators = ValidationPrototype::find_for_prop(
            txn,
            tenancy,
            visibility,
            *prop.id(),
            UNSET_ID_VALUE.into(),
        )
        .await?;

        for validator in validators {
            let func = Func::get_by_id(txn, tenancy, visibility, &validator.func_id())
                .await?
                .ok_or_else(|| ComponentError::MissingFunc(validator.func_id().to_string()))?;
            let func_binding = match func.backend_kind() {
                FuncBackendKind::ValidateStringValue => {
                    let mut args =
                        FuncBackendValidateStringValueArgs::deserialize(validator.args())?;
                    args.value = Some(value.to_string());
                    let args_json = serde_json::to_value(args)?;
                    let (func_binding, binding_created) = FuncBinding::find_or_create(
                        txn,
                        nats,
                        tenancy,
                        visibility,
                        history_actor,
                        args_json,
                        *func.id(),
                        *func.backend_kind(),
                    )
                    .await?;
                    // Note for future humans - if this isn't a built in, then we need to
                    // think about execution time. Probably higher up than this? But just
                    // an FYI.
                    if binding_created {
                        func_binding.execute(txn, nats, veritech.clone()).await?;
                    }
                    func_binding
                }
                kind => unimplemented!("Validator Backend not supported yet: {}", kind),
            };

            if created {
                let mut existing_validation_resolvers = ValidationResolver::find_for_prototype(
                    txn,
                    tenancy,
                    visibility,
                    validator.id(),
                )
                .await?;

                // If we dont' have one, create the validation resolver. If we do, update the
                // func binding id to point to the new value. Interesting to think about
                // garbage collecting the left over funcbinding + func result value?
                if let Some(mut validation_resolver) = existing_validation_resolvers.pop() {
                    validation_resolver
                        .set_func_binding_id(
                            txn,
                            nats,
                            visibility,
                            history_actor,
                            *func_binding.id(),
                        )
                        .await?;
                } else {
                    let mut validation_resolver_context = ValidationResolverContext::new();
                    validation_resolver_context.set_prop_id(*prop.id());
                    validation_resolver_context.set_component_id(*self.id());
                    ValidationResolver::new(
                        txn,
                        nats,
                        tenancy,
                        visibility,
                        history_actor,
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
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        veritech: veritech::Client,
        tenancy: &Tenancy,
        visibility: &Visibility,
        history_actor: &HistoryActor,
        system_id: SystemId,
    ) -> ComponentResult<()> {
        let mut schema_tenancy = tenancy.clone();
        schema_tenancy.universal = true;

        let schema = self
            .schema_with_tenancy(txn, &schema_tenancy, visibility)
            .await?
            .ok_or(ComponentError::SchemaNotFound)?;
        let schema_variant = self
            .schema_variant_with_tenancy(txn, &schema_tenancy, visibility)
            .await?
            .ok_or(ComponentError::SchemaVariantNotFound)?;

        let qualification_prototypes = QualificationPrototype::find_for_component(
            txn,
            &schema_tenancy,
            visibility,
            *self.id(),
            *schema.id(),
            *schema_variant.id(),
            system_id,
        )
        .await?;

        for prototype in qualification_prototypes {
            let func = Func::get_by_id(txn, &schema_tenancy, visibility, &prototype.func_id())
                .await?
                .ok_or_else(|| ComponentError::MissingFunc(prototype.func_id().to_string()))?;

            let args = FuncBackendJsQualificationArgs {
                component: self
                    .veritech_qualification_check_component(txn, &schema_tenancy, visibility)
                    .await?,
            };
            let json_args = serde_json::to_value(args)?;

            let (func_binding, created) = FuncBinding::find_or_create(
                txn,
                nats,
                &schema_tenancy,
                visibility,
                history_actor,
                json_args,
                prototype.func_id(),
                *func.backend_kind(),
            )
            .await?;

            if created {
                // Note for future humans - if this isn't a built in, then we need to
                // think about execution time. Probably higher up than this? But just
                // an FYI.
                func_binding.execute(txn, nats, veritech.clone()).await?;

                let mut existing_resolvers =
                    QualificationResolver::find_for_prototype_and_component(
                        txn,
                        &schema_tenancy,
                        visibility,
                        prototype.id(),
                        self.id(),
                    )
                    .await?;

                // If we do not have one, create the qualification resolver. If we do, update the
                // func binding id to point to the new value.
                if let Some(mut resolver) = existing_resolvers.pop() {
                    resolver
                        .set_func_binding_id(
                            txn,
                            nats,
                            visibility,
                            history_actor,
                            *func_binding.id(),
                        )
                        .await?;
                } else {
                    let mut resolver_context = QualificationResolverContext::new();
                    resolver_context.set_component_id(*self.id());
                    QualificationResolver::new(
                        txn,
                        nats,
                        tenancy,
                        visibility,
                        history_actor,
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

    #[tracing::instrument(skip(txn))]
    pub async fn list_validations_as_qualification_for_component_id(
        txn: &PgTxn<'_>,
        tenancy: &Tenancy,
        visibility: &Visibility,
        component_id: ComponentId,
        system_id: SystemId,
    ) -> ComponentResult<QualificationView> {
        let validation_field_values = ValidationResolver::list_values_for_component(
            txn,
            tenancy,
            visibility,
            component_id,
            system_id,
        )
        .await?;

        let mut validation_errors: Vec<(Prop, Vec<ValidationError>)> = Vec::new();
        for (prop, field_value) in validation_field_values.into_iter() {
            if let Some(value_json) = field_value.value() {
                // This clone shouldn't be neccessary, but we have no way to get to the owned value -- Adam
                let internal_validation_errors: Vec<ValidationError> =
                    serde_json::from_value(value_json.clone())?;
                validation_errors.push((prop, internal_validation_errors));
            }
        }
        let qualification_view = QualificationView::new_for_validation_errors(validation_errors);
        Ok(qualification_view)
    }

    #[tracing::instrument(skip(txn))]
    pub async fn list_qualifications(
        &self,
        txn: &PgTxn<'_>,
        tenancy: &Tenancy,
        visibility: &Visibility,
        system_id: SystemId,
    ) -> ComponentResult<Vec<QualificationView>> {
        Self::list_qualifications_by_component_id(txn, tenancy, visibility, *self.id(), system_id)
            .await
    }

    #[tracing::instrument(skip(txn))]
    pub async fn list_qualifications_by_component_id(
        txn: &PgTxn<'_>,
        tenancy: &Tenancy,
        visibility: &Visibility,
        component_id: ComponentId,
        system_id: SystemId,
    ) -> ComponentResult<Vec<QualificationView>> {
        let mut results: Vec<QualificationView> = Vec::new();

        // This is the "All Fields Valid" universal qualification
        let validation_qualification = Self::list_validations_as_qualification_for_component_id(
            txn,
            tenancy,
            visibility,
            component_id,
            system_id,
        )
        .await?;
        results.push(validation_qualification);

        let rows = txn
            .query(
                LIST_QUALIFICATIONS,
                &[&tenancy, &visibility, &component_id, &system_id],
            )
            .await?;
        for row in rows.into_iter() {
            let json: serde_json::Value = row.try_get("object")?;
            let func_binding_return_value: FuncBindingReturnValue = serde_json::from_value(json)?;
            let mut qual_view = QualificationView::try_from(func_binding_return_value)?;
            let title: String = row.try_get("title")?;
            let link: Option<String> = row.try_get("link")?;
            qual_view.title = title;
            qual_view.link = link;
            results.push(qual_view);
        }

        Ok(results)
    }

    #[tracing::instrument(skip(txn))]
    pub async fn get_resource_by_component_and_system(
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        tenancy: &Tenancy,
        visibility: &Visibility,
        history_actor: &HistoryActor,
        component_id: ComponentId,
        system_id: SystemId,
    ) -> ComponentResult<ResourceView> {
        // Note(paulo): This should not be an upsert, but we currently don't have a way to create resource attached to a system during Component creation
        let resource = Resource::upsert(
            txn,
            nats,
            tenancy,
            visibility,
            history_actor,
            &component_id,
            &system_id,
        )
        .await?;

        let row = txn
            .query_opt(
                GET_RESOURCE,
                &[&tenancy, &visibility, &component_id, &system_id],
            )
            .await?;

        let json: Option<serde_json::Value> = row.map(|row| row.try_get("object")).transpose()?;

        let func_binding_return_value: Option<FuncBindingReturnValue> =
            json.map(serde_json::from_value).transpose()?;
        let res_view = ResourceView::from((resource, func_binding_return_value));

        Ok(res_view)
    }

    pub async fn veritech_resource_sync_component(
        &self,
        txn: &PgTxn<'_>,
        tenancy: &Tenancy,
        visibility: &Visibility,
    ) -> ComponentResult<veritech::ResourceSyncComponent> {
        let mut tenancy = tenancy.clone();
        tenancy.universal = true;

        let mut qualification_view = veritech::ResourceSyncComponent {
            name: self.name().into(),
            properties: HashMap::new(),
        };

        for edit_field in Component::get_edit_fields(txn, &tenancy, visibility, self.id()).await? {
            // This whole segment needs to be replaced with something that can handle the
            // full complexity here.
            if let Some(v) = edit_field.value {
                qualification_view.properties.insert(edit_field.name, v);
            }
        }

        Ok(qualification_view)
    }

    pub async fn veritech_qualification_check_component(
        &self,
        txn: &PgTxn<'_>,
        tenancy: &Tenancy,
        visibility: &Visibility,
    ) -> ComponentResult<veritech::QualificationCheckComponent> {
        let mut tenancy = tenancy.clone();
        tenancy.universal = true;

        let mut qualification_view = veritech::QualificationCheckComponent {
            name: self.name().into(),
            properties: HashMap::new(),
        };

        for edit_field in Component::get_edit_fields(txn, &tenancy, visibility, self.id()).await? {
            // This whole segment needs to be replaced with something that can handle the
            // full complexity here.
            if let Some(v) = edit_field.value {
                qualification_view.properties.insert(edit_field.name, v);
            }
        }

        Ok(qualification_view)
    }

    #[tracing::instrument(skip(txn))]
    pub async fn list_for_resource_sync(txn: &PgTxn<'_>) -> ComponentResult<Vec<Component>> {
        let visibility = Visibility::new_head(false);
        let rows = txn.query(LIST_FOR_RESOURCE_SYNC, &[&visibility]).await?;
        let results = standard_model::objects_from_rows(rows)?;
        Ok(results)
    }

    #[tracing::instrument]
    pub async fn sync_resource(
        &self,
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        veritech: veritech::Client,
        history_actor: &HistoryActor,
        system_id: SystemId,
    ) -> ComponentResult<()> {
        tracing::warn!("checking resource: {:?}", self);

        // Note(paulo): we don't actually care about the Resource here, we only care about the ResourcePrototype, is this wrong?

        let mut schema_tenancy = self.tenancy.clone();
        schema_tenancy.universal = true;

        let schema = self
            .schema_with_tenancy(txn, &schema_tenancy, &self.visibility)
            .await?
            .ok_or(ComponentError::SchemaNotFound)?;
        let schema_variant = self
            .schema_variant_with_tenancy(txn, &schema_tenancy, &self.visibility)
            .await?
            .ok_or(ComponentError::SchemaVariantNotFound)?;

        let resource_prototype = ResourcePrototype::get_for_component(
            txn,
            &schema_tenancy,
            &self.visibility,
            *self.id(),
            *schema.id(),
            *schema_variant.id(),
            system_id,
        )
        .await?;

        if let Some(prototype) = resource_prototype {
            let func =
                Func::get_by_id(txn, &schema_tenancy, &self.visibility, &prototype.func_id())
                    .await?
                    .ok_or_else(|| ComponentError::MissingFunc(prototype.func_id().to_string()))?;

            let args = FuncBackendJsResourceSyncArgs {
                component: self
                    .veritech_resource_sync_component(txn, &schema_tenancy, &self.visibility)
                    .await?,
            };
            let json_args = serde_json::to_value(args)?;

            let (func_binding, _created) = FuncBinding::find_or_create(
                txn,
                nats,
                &schema_tenancy,
                &self.visibility,
                history_actor,
                json_args,
                prototype.func_id(),
                *func.backend_kind(),
            )
            .await?;

            // Note: We need to execute the same func binding a bunch of times
            func_binding.execute(txn, nats, veritech.clone()).await?;

            // Note for future humans - if this isn't a built in, then we need to
            // think about execution time. Probably higher up than this? But just
            // an FYI.
            let existing_resolver = ResourceResolver::get_for_prototype_and_component(
                txn,
                &schema_tenancy,
                &self.visibility,
                prototype.id(),
                self.id(),
            )
            .await?;

            // If we do not have one, create the resource resolver. If we do, update the
            // func binding id to point to the new value.
            let mut resolver = if let Some(resolver) = existing_resolver {
                resolver
            } else {
                let mut resolver_context = ResourceResolverContext::new();
                resolver_context.set_component_id(*self.id());
                ResourceResolver::new(
                    txn,
                    nats,
                    &self.tenancy,
                    &self.visibility,
                    history_actor,
                    *prototype.id(),
                    *func.id(),
                    *func_binding.id(),
                    resolver_context,
                )
                .await?
            };
            resolver
                .set_func_binding_id(
                    txn,
                    nats,
                    &self.visibility,
                    history_actor,
                    *func_binding.id(),
                )
                .await?;
        }

        Ok(())
    }
}

#[async_trait]
impl EditFieldAble for Component {
    type Id = ComponentId;
    type Error = ComponentError;

    async fn get_edit_fields(
        txn: &PgTxn<'_>,
        tenancy: &Tenancy,
        visibility: &Visibility,
        id: &ComponentId,
    ) -> ComponentResult<EditFields> {
        let mut tenancy = tenancy.clone();
        tenancy.universal = true;
        let head_visibility = Visibility::new_head(visibility.deleted);
        let change_set_visibility =
            Visibility::new_change_set(visibility.change_set_pk, visibility.deleted);

        let component = Component::get_by_id(txn, &tenancy, visibility, id)
            .await?
            .ok_or(ComponentError::NotFound(*id))?;
        let head_object: Option<Component> = if visibility.in_change_set() {
            Component::get_by_id(txn, &tenancy, &head_visibility, id).await?
        } else {
            None
        };
        let change_set_object: Option<Component> = if visibility.in_change_set() {
            Component::get_by_id(txn, &tenancy, &change_set_visibility, id).await?
        } else {
            None
        };

        let mut edit_fields: EditFields = vec![Self::name_edit_field(
            visibility,
            &component,
            &head_object,
            &change_set_object,
        )?];

        let schema_variant = component
            .schema_variant_with_tenancy(txn, &tenancy, visibility)
            .await?
            .ok_or(ComponentError::SchemaVariantNotFound)?;

        let props = schema_variant.props(txn, visibility).await?;
        for prop in props.iter() {
            let system_id = UNSET_ID_VALUE.into();
            let current_value: Option<FuncBindingReturnValue> =
                match AttributeResolver::find_value_for_prop_and_component(
                    txn,
                    &tenancy,
                    visibility,
                    *prop.id(),
                    *component.id(),
                    system_id,
                )
                .await
                {
                    Ok(v) => Some(v),
                    Err(e) => {
                        dbg!("missing attribute resolver; might be fine, might be a bug! who knows? only god.");
                        dbg!(&e);
                        None
                    }
                };
            let head_value: Option<FuncBindingReturnValue> = if visibility.in_change_set() {
                match AttributeResolver::find_value_for_prop_and_component(
                    txn,
                    &tenancy,
                    &head_visibility,
                    *prop.id(),
                    *component.id(),
                    system_id,
                )
                .await
                {
                    Ok(v) => Some(v),
                    Err(e) => {
                        dbg!("missing attribute resolver; might be fine, might be a bug! who knows? only god.");
                        dbg!(&e);
                        None
                    }
                }
            } else {
                None
            };
            let change_set_value: Option<FuncBindingReturnValue> = if visibility.in_change_set() {
                match AttributeResolver::find_value_for_prop_and_component(
                    txn,
                    &tenancy,
                    &change_set_visibility,
                    *prop.id(),
                    *component.id(),
                    system_id,
                )
                .await
                {
                    Ok(v) => Some(v),
                    Err(e) => {
                        dbg!("missing attribute resolver; might be fine, might be a bug! who knows? only god.");
                        dbg!(&e);
                        None
                    }
                }
            } else {
                None
            };

            let field_name = prop.name();
            let object_kind = EditFieldObjectKind::ComponentProp;

            fn extract_value(fbrv: &FuncBindingReturnValue) -> Option<&serde_json::Value> {
                fbrv.value()
            }

            let (value, visibility_diff) = value_and_visibility_diff_json_option(
                visibility,
                current_value.as_ref(),
                extract_value,
                head_value.as_ref(),
                change_set_value.as_ref(),
            )?;

            let mut validation_errors = Vec::new();
            let validation_field_values = ValidationResolver::find_values_for_prop_and_component(
                txn,
                &tenancy,
                visibility,
                *prop.id(),
                *component.id(),
                system_id,
            )
            .await?;
            for field_value in validation_field_values.into_iter() {
                if let Some(value_json) = field_value.value() {
                    // This clone shouldn't be neccessary, but we have no way to get to the owned value -- Adam
                    let mut validation_error: Vec<ValidationError> =
                        serde_json::from_value(value_json.clone())?;
                    validation_errors.append(&mut validation_error);
                }
            }

            let mut edit_field = EditField::new(
                field_name,
                vec!["properties".to_string()],
                object_kind,
                *id,
                EditFieldDataType::String,
                Widget::Text(TextWidget::new()),
                value,
                visibility_diff,
                validation_errors,
            );
            edit_field.set_baggage(EditFieldBaggage::ComponentProp(
                EditFieldBaggageComponentProp {
                    prop_id: *prop.id(),
                    system_id: None,
                },
            ));
            edit_fields.push(edit_field);
        }

        Ok(edit_fields)
    }

    async fn update_from_edit_field(
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        tenancy: &Tenancy,
        visibility: &Visibility,
        history_actor: &HistoryActor,
        id: Self::Id,
        edit_field_id: String,
        value: Option<serde_json::Value>,
    ) -> ComponentResult<()> {
        let edit_field_id = edit_field_id.as_ref();
        let mut component = Component::get_by_id(txn, tenancy, visibility, &id)
            .await?
            .ok_or(ComponentError::NotFound(id))?;

        match edit_field_id {
            "name" => match value {
                Some(json_value) => {
                    let value = json_value.as_str().map(|s| s.to_string()).ok_or(
                        Self::Error::EditField(EditFieldError::InvalidValueType("string")),
                    )?;
                    component
                        .set_name(txn, nats, visibility, history_actor, value)
                        .await?;
                }
                None => return Err(EditFieldError::MissingValue.into()),
            },
            invalid => return Err(EditFieldError::invalid_field(invalid).into()),
        }

        Ok(())
    }
}
