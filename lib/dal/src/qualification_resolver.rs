use crate::DalContext;
use serde::{Deserialize, Serialize};
use si_data_nats::NatsError;
use si_data_pg::PgError;
use std::default::Default;
use telemetry::prelude::*;
use thiserror::Error;

use crate::{
    func::{binding::FuncBindingId, FuncId},
    impl_standard_model, pk,
    standard_model::{self, objects_from_rows},
    standard_model_accessor, ComponentId, HistoryEventError, QualificationPrototypeId, SchemaId,
    SchemaVariantId, StandardModel, StandardModelError, Timestamp, Visibility, WriteTenancy,
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

const FIND_FOR_PROTOTYPE: &str =
    include_str!("./queries/qualification_resolver_find_for_prototype.sql");

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct QualificationResolverContext {
    component_id: ComponentId,
    schema_id: SchemaId,
    schema_variant_id: SchemaVariantId,
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
            component_id: ComponentId::NONE,
            schema_id: SchemaId::NONE,
            schema_variant_id: SchemaVariantId::NONE,
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
    tenancy: WriteTenancy,
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
    #[instrument(skip_all)]
    pub async fn new(
        ctx: &DalContext,
        qualification_prototype_id: QualificationPrototypeId,
        func_id: FuncId,
        func_binding_id: FuncBindingId,
        context: QualificationResolverContext,
    ) -> QualificationResolverResult<Self> {
        let row = ctx.txns().pg().query_one(
                "SELECT object FROM qualification_resolver_create_v1($1, $2, $3, $4, $5, $6, $7, $8)",
                &[ctx.write_tenancy(), ctx.visibility(),
                    &qualification_prototype_id,
                    &func_id,
                    &func_binding_id,
                    &context.component_id(),
                    &context.schema_id(),
                    &context.schema_variant_id(),
                ],
            )
            .await?;
        let object = standard_model::finish_create_from_row(ctx, row).await?;
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

    pub async fn find_for_prototype_and_component(
        ctx: &DalContext,
        qualification_prototype_id: &QualificationPrototypeId,
        component_id: &ComponentId,
    ) -> QualificationResolverResult<Vec<Self>> {
        let rows = ctx
            .txns()
            .pg()
            .query(
                FIND_FOR_PROTOTYPE,
                &[
                    ctx.read_tenancy(),
                    ctx.visibility(),
                    qualification_prototype_id,
                    component_id,
                ],
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
        assert_eq!(c.component_id(), 15.into());
    }
}
