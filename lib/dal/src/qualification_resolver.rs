use serde::{Deserialize, Serialize};
use si_data::{NatsError, NatsTxn, PgError, PgTxn};
use std::default::Default;
use telemetry::prelude::*;
use thiserror::Error;

use crate::{
    func::{binding::FuncBindingId, FuncId},
    impl_standard_model, pk,
    standard_model::{self, objects_from_rows},
    standard_model_accessor, ComponentId, HistoryActor, HistoryEventError, PropId,
    QualificationPrototypeId, SchemaId, SchemaVariantId, StandardModel, StandardModelError,
    SystemId, Tenancy, Timestamp, Visibility,
};

#[derive(Error, Debug)]
pub enum QualificationResolverError {
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

pub type QualificationResolverResult<T> = Result<T, QualificationResolverError>;

pub const UNSET_ID_VALUE: i64 = -1;
const FIND_FOR_PROTOTYPE: &str =
    include_str!("./queries/qualification_resolver_find_for_prototype.sql");

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct QualificationResolverContext {
    prop_id: PropId,
    component_id: ComponentId,
    schema_id: SchemaId,
    schema_variant_id: SchemaVariantId,
    system_id: SystemId,
}

// Hrm - is this a universal resolver context? -- Adam
impl Default for QualificationResolverContext {
    fn default() -> Self {
        Self::new()
    }
}

impl QualificationResolverContext {
    pub fn new() -> Self {
        QualificationResolverContext {
            prop_id: UNSET_ID_VALUE.into(),
            component_id: UNSET_ID_VALUE.into(),
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

pk!(QualificationResolverPk);
pk!(QualificationResolverId);

// An QualificationResolver joins a `FuncBinding` to the context in which
// its corresponding `FuncBindingResultValue` is consumed.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct QualificationResolver {
    pk: QualificationResolverPk,
    id: QualificationResolverId,
    qualification_prototype_id: QualificationPrototypeId,
    func_id: FuncId,
    func_binding_id: FuncBindingId,
    #[serde(flatten)]
    context: QualificationResolverContext,
    #[serde(flatten)]
    tenancy: Tenancy,
    #[serde(flatten)]
    timestamp: Timestamp,
    #[serde(flatten)]
    visibility: Visibility,
}

impl_standard_model! {
    model: QualificationResolver,
    pk: QualificationResolverPk,
    id: QualificationResolverId,
    table_name: "qualification_resolvers",
    history_event_label_base: "qualification_resolver",
    history_event_message_name: "Qualification Resolver"
}

impl QualificationResolver {
    #[allow(clippy::too_many_arguments)]
    #[tracing::instrument(skip(txn, nats))]
    pub async fn new(
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        tenancy: &Tenancy,
        visibility: &Visibility,
        history_actor: &HistoryActor,
        qualification_prototype_id: QualificationPrototypeId,
        func_id: FuncId,
        func_binding_id: FuncBindingId,
        context: QualificationResolverContext,
    ) -> QualificationResolverResult<Self> {
        let row = txn
            .query_one(
                "SELECT object FROM qualification_resolver_create_v1($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
                &[
                    &tenancy,
                    &visibility,
                    &qualification_prototype_id,
                    &func_id,
                    &func_binding_id,
                    &context.prop_id(),
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
            tenancy,
            visibility,
            history_actor,
            row,
        )
        .await?;
        Ok(object)
    }

    standard_model_accessor!(
        qualification_prototype_id,
        Pk(QualificationPrototypeId),
        QualificationResolverResult
    );
    standard_model_accessor!(func_id, Pk(FuncId), QualificationResolverResult);
    standard_model_accessor!(
        func_binding_id,
        Pk(FuncBindingId),
        QualificationResolverResult
    );

    pub async fn find_for_prototype(
        txn: &PgTxn<'_>,
        tenancy: &Tenancy,
        visibility: &Visibility,
        qualification_prototype_id: &QualificationPrototypeId,
    ) -> QualificationResolverResult<Vec<Self>> {
        let rows = txn
            .query(
                FIND_FOR_PROTOTYPE,
                &[&tenancy, &visibility, qualification_prototype_id],
            )
            .await?;
        let object = objects_from_rows(rows)?;
        Ok(object)
    }
}

#[cfg(test)]
mod test {
    use super::QualificationResolverContext;

    #[test]
    fn context_builder() {
        let mut c = QualificationResolverContext::new();
        c.set_component_id(15.into());
        c.set_prop_id(22.into());
        assert_eq!(c.component_id(), 15.into());
        assert_eq!(c.prop_id(), 22.into());
    }
}
