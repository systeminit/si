use serde::{Deserialize, Serialize};
use si_data_nats::NatsError;
use si_data_pg::PgError;
use std::collections::HashMap;
use telemetry::prelude::*;
use thiserror::Error;

use crate::validation::ValidationError;
use crate::{
    func::{
        binding::FuncBindingId, binding_return_value::FuncBindingReturnValue,
        binding_return_value::FuncBindingReturnValueId, FuncId,
    },
    impl_standard_model, pk,
    schema::variant::SchemaVariantError,
    standard_model, standard_model_accessor, AttributeReadContext, AttributeValueId, Component,
    ComponentId, HistoryEventError, StandardModel, StandardModelError, Tenancy, Timestamp,
    ValidationPrototype, ValidationPrototypeId, Visibility,
};
use crate::{DalContext, TransactionsError};

#[allow(clippy::large_enum_variant)]
#[remain::sorted]
#[derive(Error, Debug)]
pub enum ValidationResolverError {
    #[error("component error: {0}")]
    Component(String),
    #[error("component not found: {0}")]
    ComponentNotFound(ComponentId),
    #[error("history event error: {0}")]
    HistoryEvent(#[from] HistoryEventError),
    #[error("invalid prop id")]
    InvalidPropId,
    #[error("nats txn error: {0}")]
    Nats(#[from] NatsError),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("schema not found")]
    SchemaNotFound,
    #[error("schema variant error: {0}")]
    SchemaVariant(#[from] SchemaVariantError),
    #[error("schema variant not found")]
    SchemaVariantNotFound,
    #[error("error serializing/deserializing json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("standard model error: {0}")]
    StandardModel(#[from] StandardModelError),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
}

pub type ValidationResolverResult<T> = Result<T, ValidationResolverError>;

const FIND_STATUS: &str = include_str!("../queries/validation_resolver/find_status.sql");
const FIND_FOR_ATTRIBUTE_VALUE_AND_FUNC_BINDING: &str =
    include_str!("../queries/validation_resolver/find_for_attribute_value_and_func_binding.sql");

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ValidationStatus {
    pub attribute_value_id: AttributeValueId,
    pub errors: Vec<ValidationError>,
}

pk!(ValidationResolverPk);
pk!(ValidationResolverId);

// An ValidationResolver joins a `FuncBinding` to the context in which
// its corresponding `FuncBindingResultValue` is consumed.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct ValidationResolver {
    pk: ValidationResolverPk,
    id: ValidationResolverId,
    validation_prototype_id: ValidationPrototypeId,
    attribute_value_id: AttributeValueId,
    /// The [`FuncId`] of the validation func that we're the resolver for.
    validation_func_id: FuncId,
    /// The [`FuncBindingId`] for the validation func itself.
    validation_func_binding_id: FuncBindingId,
    /// The [`FuncBindingReturnValueId`] that represents the value at this specific position & context.
    attribute_value_func_binding_return_value_id: FuncBindingReturnValueId,
    #[serde(flatten)]
    tenancy: Tenancy,
    #[serde(flatten)]
    timestamp: Timestamp,
    #[serde(flatten)]
    visibility: Visibility,
}

impl_standard_model! {
    model: ValidationResolver,
    pk: ValidationResolverPk,
    id: ValidationResolverId,
    table_name: "validation_resolvers",
    history_event_label_base: "validation_resolver",
    history_event_message_name: "Validation Resolver"
}

impl ValidationResolver {
    #[allow(clippy::too_many_arguments)]
    #[instrument(skip_all)]
    pub async fn new(
        ctx: &DalContext,
        validation_prototype_id: ValidationPrototypeId,
        attribute_value_id: AttributeValueId,
        func_binding_id: FuncBindingId,
    ) -> ValidationResolverResult<Self> {
        let row = ctx
            .txns()
            .await?
            .pg()
            .query_one(
                "SELECT object FROM validation_resolver_create_v1($1, $2, $3, $4, $5)",
                &[
                    ctx.tenancy(),
                    ctx.visibility(),
                    &validation_prototype_id,
                    &attribute_value_id,
                    &func_binding_id,
                ],
            )
            .await?;
        let object = standard_model::finish_create_from_row(ctx, row).await?;
        Ok(object)
    }

    standard_model_accessor!(
        validation_prototype_id,
        Pk(ValidationPrototypeId),
        ValidationResolverResult
    );
    standard_model_accessor!(validation_func_id, Pk(FuncId), ValidationResolverResult);
    standard_model_accessor!(
        validation_func_binding_id,
        Pk(FuncBindingId),
        ValidationResolverResult
    );
    standard_model_accessor!(
        attribute_value_func_binding_return_value_id,
        Pk(FuncBindingReturnValueId),
        ValidationResolverResult
    );

    pub async fn find_for_attribute_value_and_validation_func(
        ctx: &DalContext,
        attribute_value_id: AttributeValueId,
        func_id: FuncId,
    ) -> ValidationResolverResult<Vec<Self>> {
        let rows = ctx
            .txns()
            .await?
            .pg()
            .query(
                FIND_FOR_ATTRIBUTE_VALUE_AND_FUNC_BINDING,
                &[
                    ctx.tenancy(),
                    ctx.visibility(),
                    &attribute_value_id,
                    &func_id,
                ],
            )
            .await?;

        Ok(standard_model::objects_from_rows(rows)?)
    }

    /// Find the status of validation(s) for a given [`ComponentId`](crate::Component).
    pub async fn find_status(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> ValidationResolverResult<Vec<ValidationStatus>> {
        let schema_variant = Component::get_by_id(ctx, &component_id)
            .await
            .map_err(|err| ValidationResolverError::Component(err.to_string()))?
            .ok_or(ValidationResolverError::ComponentNotFound(component_id))?
            .schema_variant(ctx)
            .await
            .map_err(|err| ValidationResolverError::Component(err.to_string()))?
            .ok_or(ValidationResolverError::SchemaVariantNotFound)?;
        let context = AttributeReadContext {
            prop_id: None,
            component_id: Some(component_id),
            ..AttributeReadContext::default()
        };
        let rows = ctx
            .txns()
            .await?
            .pg()
            .query(
                FIND_STATUS,
                &[
                    ctx.tenancy(),
                    ctx.visibility(),
                    &context,
                    schema_variant.id(),
                ],
            )
            .await?;

        let mut result = HashMap::new();
        for row in rows {
            let json: Option<serde_json::Value> = row.try_get("object")?;
            let object: Option<FuncBindingReturnValue> =
                serde_json::from_value(json.unwrap_or(serde_json::Value::Null))?;

            let json: Option<serde_json::Value> = row.try_get("validation_prototype_json")?;
            let prototype: Option<ValidationPrototype> =
                serde_json::from_value(json.unwrap_or(serde_json::Value::Null))?;

            let attribute_value_id: AttributeValueId = row.try_get("attribute_value_id")?;

            let key = result
                .entry(attribute_value_id)
                .or_insert(ValidationStatus {
                    attribute_value_id,
                    errors: vec![],
                });

            if let Some(value_json) = object.as_ref().and_then(|o| o.value()) {
                let errors = Vec::<ValidationError>::deserialize(value_json)?;
                key.errors.reserve(errors.len());
                for mut error in errors {
                    error.link = prototype
                        .as_ref()
                        .and_then(|p| p.link())
                        .map(|l| l.to_owned());
                    key.errors.push(error)
                }
            }
        }
        Ok(result.into_values().collect())
    }
}
