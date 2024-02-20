use serde::{Deserialize, Serialize};
use si_data_nats::NatsError;
use si_data_pg::PgError;
use telemetry::prelude::*;
use thiserror::Error;
use veritech_client::OutputStream;

use crate::{
    impl_standard_model, pk, schema::variant::SchemaVariantError, standard_model,
    standard_model_accessor, ComponentId, HistoryEventError, PropId, StandardModel,
    StandardModelError, Tenancy, Timestamp, Visibility,
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

pk!(ValidationResolverPk);
pk!(ValidationResolverId);

#[remain::sorted]
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum ValidationStatus {
    Error,
    Failure,
    Success,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct ValidationOutput {
    pub status: ValidationStatus,
    pub message: String,
    pub logs: Vec<OutputStream>,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct ValidationResolver {
    pk: ValidationResolverPk,
    id: ValidationResolverId,
    prop_id: PropId,
    component_id: ComponentId,
    key: Option<String>,
    value: serde_json::Value,
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
    pub async fn upsert(
        ctx: &DalContext,
        prop_id: PropId,
        component_id: ComponentId,
        key: Option<&str>,
        value: &ValidationOutput,
    ) -> ValidationResolverResult<Self> {
        let row = ctx
            .txns()
            .await?
            .pg()
            .query_one(
                "SELECT object FROM validation_resolver_upsert_v2($1, $2, $3, $4, $5, $6)",
                &[
                    ctx.tenancy(),
                    ctx.visibility(),
                    &prop_id,
                    &component_id,
                    &key,
                    &serde_json::to_value(value)?,
                ],
            )
            .await?;
        let object = standard_model::finish_create_from_row(ctx, row).await?;
        Ok(object)
    }

    pub fn value(&self) -> ValidationResolverResult<ValidationOutput> {
        Ok(ValidationOutput::deserialize(&self.value)?)
    }

    standard_model_accessor!(prop_id, Pk(PropId), ValidationResolverResult);
    standard_model_accessor!(component_id, Pk(ComponentId), ValidationResolverResult);
    standard_model_accessor!(key, Option<String>, ValidationResolverResult);
}
