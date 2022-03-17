use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use si_data::{NatsError, NatsTxn, PgError, PgTxn};
use std::default::Default;
use telemetry::prelude::*;
use thiserror::Error;

use crate::{
    func::FuncId, impl_standard_model, pk, standard_model, standard_model_accessor, ComponentId,
    HistoryActor, HistoryEventError, ReadTenancy, SchemaId, SchemaVariantId, StandardModel,
    StandardModelError, SystemId, Tenancy, Timestamp, Visibility, WriteTenancy,
};

#[derive(Error, Debug)]
pub enum ResourcePrototypeError {
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
    #[error("component not found: {0}")]
    ComponentNotFound(ComponentId),
    #[error("component error: {0}")]
    Component(String),
    #[error("schema not found")]
    SchemaNotFound,
    #[error("schema variant not found")]
    SchemaVariantNotFound,
}

pub type ResourcePrototypeResult<T> = Result<T, ResourcePrototypeError>;

pub const UNSET_ID_VALUE: i64 = -1;
const GET_FOR_CONTEXT: &str = include_str!("./queries/resource_prototype_get_for_context.sql");

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct ResourcePrototypeContext {
    component_id: ComponentId,
    schema_id: SchemaId,
    schema_variant_id: SchemaVariantId,
    system_id: SystemId,
}

// Hrm - is this a universal resolver context? -- Adam
impl Default for ResourcePrototypeContext {
    fn default() -> Self {
        Self::new()
    }
}

impl ResourcePrototypeContext {
    pub fn new() -> Self {
        Self {
            component_id: UNSET_ID_VALUE.into(),
            schema_id: UNSET_ID_VALUE.into(),
            schema_variant_id: UNSET_ID_VALUE.into(),
            system_id: UNSET_ID_VALUE.into(),
        }
    }

    pub fn component_id(&self) -> ComponentId {
        self.component_id
    }

    pub fn set_component_id(&mut self, component_id: ComponentId) {
        self.component_id = component_id;
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

pk!(ResourcePrototypePk);
pk!(ResourcePrototypeId);

// An ResourcePrototype joins a `Func` to the context in which
// the component that is created with it can use to generate a ResourceResolver.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct ResourcePrototype {
    pk: ResourcePrototypePk,
    id: ResourcePrototypeId,
    func_id: FuncId,
    args: serde_json::Value,
    #[serde(flatten)]
    context: ResourcePrototypeContext,
    #[serde(flatten)]
    tenancy: Tenancy,
    #[serde(flatten)]
    timestamp: Timestamp,
    #[serde(flatten)]
    visibility: Visibility,
}

impl_standard_model! {
    model: ResourcePrototype,
    pk: ResourcePrototypePk,
    id: ResourcePrototypeId,
    table_name: "resource_prototypes",
    history_event_label_base: "resource_prototype",
    history_event_message_name: "Resource Prototype"
}

impl ResourcePrototype {
    #[allow(clippy::too_many_arguments)]
    #[instrument(skip_all)]
    pub async fn new(
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        write_tenancy: &WriteTenancy,
        visibility: &Visibility,
        history_actor: &HistoryActor,
        func_id: FuncId,
        args: serde_json::Value,
        context: ResourcePrototypeContext,
    ) -> ResourcePrototypeResult<Self> {
        let row = txn
            .query_one(
                "SELECT object FROM resource_prototype_create_v1($1, $2, $3, $4, $5, $6, $7, $8)",
                &[
                    write_tenancy,
                    &visibility,
                    &func_id,
                    &args,
                    &context.component_id(),
                    &context.schema_id(),
                    &context.schema_variant_id(),
                    &context.system_id(),
                ],
            )
            .await?;
        let object = standard_model::finish_create_from_row(
            txn,
            nats,
            &write_tenancy.into(),
            visibility,
            history_actor,
            row,
        )
        .await?;
        Ok(object)
    }

    standard_model_accessor!(func_id, Pk(FuncId), ResourcePrototypeResult);
    standard_model_accessor!(args, Json<JsonValue>, ResourcePrototypeResult);

    #[allow(clippy::too_many_arguments)]
    pub async fn get_for_component(
        txn: &PgTxn<'_>,
        read_tenancy: &ReadTenancy,
        visibility: &Visibility,
        component_id: ComponentId,
        schema_id: SchemaId,
        schema_variant_id: SchemaVariantId,
        system_id: SystemId,
    ) -> ResourcePrototypeResult<Option<Self>> {
        let row = txn
            .query_opt(
                GET_FOR_CONTEXT,
                &[
                    read_tenancy,
                    &visibility,
                    &component_id,
                    &system_id,
                    &schema_variant_id,
                    &schema_id,
                ],
            )
            .await?;
        let object = standard_model::option_object_from_row(row)?;
        Ok(object)
    }
}

#[cfg(test)]
mod test {
    use super::ResourcePrototypeContext;

    #[test]
    fn context_builder() {
        let mut c = ResourcePrototypeContext::new();
        c.set_component_id(22.into());
        assert_eq!(c.component_id(), 22.into());
    }
}
