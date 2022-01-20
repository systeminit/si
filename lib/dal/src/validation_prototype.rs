use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use si_data::{NatsError, NatsTxn, PgError, PgTxn};
use std::default::Default;
use telemetry::prelude::*;
use thiserror::Error;

use crate::{
    func::FuncId,
    impl_standard_model, pk,
    standard_model::{self, objects_from_rows},
    standard_model_accessor, HistoryActor, HistoryEventError, PropId, SchemaId, SchemaVariantId,
    StandardModel, StandardModelError, SystemId, Tenancy, Timestamp, Visibility,
};

#[derive(Error, Debug)]
pub enum ValidationPrototypeError {
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
}

pub type ValidationPrototypeResult<T> = Result<T, ValidationPrototypeError>;

pub const UNSET_ID_VALUE: i64 = -1;
const FIND_FOR_CONTEXT: &str = include_str!("./queries/validation_prototype_find_for_context.sql");

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct ValidationPrototypeContext {
    prop_id: PropId,
    schema_id: SchemaId,
    schema_variant_id: SchemaVariantId,
    system_id: SystemId,
}

// Hrm - is this a universal resolver context? -- Adam
impl Default for ValidationPrototypeContext {
    fn default() -> Self {
        Self::new()
    }
}

impl ValidationPrototypeContext {
    pub fn new() -> Self {
        Self {
            prop_id: UNSET_ID_VALUE.into(),
            schema_id: UNSET_ID_VALUE.into(),
            schema_variant_id: UNSET_ID_VALUE.into(),
            system_id: UNSET_ID_VALUE.into(),
        }
    }

    pub fn prop_id(&self) -> PropId {
        self.prop_id
    }

    pub fn set_prop_id(&mut self, prop_id: PropId) {
        self.prop_id = prop_id;
    }

    pub fn schema_id(&self) -> SchemaId {
        self.schema_id
    }

    pub fn set_schema_id(&mut self, schema_id: SchemaId) {
        self.schema_id = schema_id;
    }

    pub fn schema_variant_id(&self) -> SchemaVariantId {
        self.schema_variant_id
    }

    pub fn set_schema_variant_id(&mut self, schema_variant_id: SchemaVariantId) {
        self.schema_variant_id = schema_variant_id;
    }

    pub fn system_id(&self) -> SystemId {
        self.system_id
    }

    pub fn set_system_id(&mut self, system_id: SystemId) {
        self.system_id = system_id;
    }
}

pk!(ValidationPrototypePk);
pk!(ValidationPrototypeId);

// An ValidationPrototype joins a `Func` to the context in which
// the component that is created with it can use to generate a ValidationResolver.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct ValidationPrototype {
    pk: ValidationPrototypePk,
    id: ValidationPrototypeId,
    func_id: FuncId,
    args: serde_json::Value,
    #[serde(flatten)]
    context: ValidationPrototypeContext,
    #[serde(flatten)]
    tenancy: Tenancy,
    #[serde(flatten)]
    timestamp: Timestamp,
    #[serde(flatten)]
    visibility: Visibility,
}

impl_standard_model! {
    model: ValidationPrototype,
    pk: ValidationPrototypePk,
    id: ValidationPrototypeId,
    table_name: "validation_prototypes",
    history_event_label_base: "validation_prototype",
    history_event_message_name: "Validation Prototype"
}

impl ValidationPrototype {
    #[allow(clippy::too_many_arguments)]
    #[tracing::instrument(skip(txn, nats))]
    pub async fn new(
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        tenancy: &Tenancy,
        visibility: &Visibility,
        history_actor: &HistoryActor,
        func_id: FuncId,
        args: serde_json::Value,
        context: ValidationPrototypeContext,
    ) -> ValidationPrototypeResult<Self> {
        let row = txn
            .query_one(
                "SELECT object FROM validation_prototype_create_v1($1, $2, $3, $4, $5, $6, $7, $8)",
                &[
                    &tenancy,
                    &visibility,
                    &func_id,
                    &args,
                    &context.prop_id(),
                    &context.schema_id(),
                    &context.schema_variant_id(),
                    &context.system_id(),
                ],
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

    standard_model_accessor!(func_id, Pk(FuncId), ValidationPrototypeResult);
    standard_model_accessor!(args, Json<JsonValue>, ValidationPrototypeResult);

    #[allow(clippy::too_many_arguments)]
    pub async fn find_for_prop(
        txn: &PgTxn<'_>,
        tenancy: &Tenancy,
        visibility: &Visibility,
        prop_id: PropId,
        system_id: SystemId,
    ) -> ValidationPrototypeResult<Vec<Self>> {
        let rows = txn
            .query(
                FIND_FOR_CONTEXT,
                &[&tenancy, &visibility, &prop_id, &system_id],
            )
            .await?;
        let object = objects_from_rows(rows)?;
        Ok(object)
    }
}

#[cfg(test)]
mod test {
    use super::ValidationPrototypeContext;

    #[test]
    fn context_builder() {
        let mut c = ValidationPrototypeContext::new();
        c.set_prop_id(22.into());
        assert_eq!(c.prop_id(), 22.into());
    }
}
