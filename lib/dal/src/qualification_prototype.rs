use std::default::Default;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use si_data_nats::NatsError;
use si_data_pg::PgError;
use telemetry::prelude::*;

use crate::{
    func::FuncId,
    impl_prototype_list_for_func, impl_standard_model, pk,
    prototype_context::{HasPrototypeContext, PrototypeContext},
    standard_model::{self, objects_from_rows},
    standard_model_accessor, ComponentId, DalContext, HistoryEventError, PrototypeListForFuncError,
    SchemaId, SchemaVariantId, StandardModel, StandardModelError, Timestamp, Visibility,
    WriteTenancy,
};

#[derive(Error, Debug)]
pub enum QualificationPrototypeError {
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
    #[error("prototype list for func: {0}")]
    PrototypeListForFunc(#[from] PrototypeListForFuncError),
}

pub type QualificationPrototypeResult<T> = Result<T, QualificationPrototypeError>;

const FIND_FOR_CONTEXT: &str =
    include_str!("./queries/qualification_prototype_find_for_context.sql");

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct QualificationPrototypeContext {
    component_id: ComponentId,
    schema_id: SchemaId,
    schema_variant_id: SchemaVariantId,
}

// Hrm - is this a universal resolver context? -- Adam
impl Default for QualificationPrototypeContext {
    fn default() -> Self {
        Self::new()
    }
}

impl PrototypeContext for QualificationPrototypeContext {
    fn component_id(&self) -> ComponentId {
        self.component_id
    }

    fn set_component_id(&mut self, component_id: ComponentId) {
        self.component_id = component_id;
    }

    fn schema_id(&self) -> SchemaId {
        self.schema_id
    }

    fn set_schema_id(&mut self, schema_id: SchemaId) {
        self.schema_id = schema_id;
    }
    fn schema_variant_id(&self) -> SchemaVariantId {
        self.schema_variant_id
    }

    fn set_schema_variant_id(&mut self, schema_variant_id: SchemaVariantId) {
        self.schema_variant_id = schema_variant_id;
    }
}

impl QualificationPrototypeContext {
    pub fn new() -> Self {
        Self {
            component_id: ComponentId::NONE,
            schema_id: SchemaId::NONE,
            schema_variant_id: SchemaVariantId::NONE,
        }
    }
}

pk!(QualificationPrototypePk);
pk!(QualificationPrototypeId);

// An QualificationPrototype joins a `Func` to the context in which
// the component that is created with it can use to generate a QualificationResolver.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct QualificationPrototype {
    pk: QualificationPrototypePk,
    id: QualificationPrototypeId,
    func_id: FuncId,
    component_id: ComponentId,
    schema_id: SchemaId,
    schema_variant_id: SchemaVariantId,
    #[serde(flatten)]
    tenancy: WriteTenancy,
    #[serde(flatten)]
    timestamp: Timestamp,
    #[serde(flatten)]
    visibility: Visibility,
}

impl_standard_model! {
    model: QualificationPrototype,
    pk: QualificationPrototypePk,
    id: QualificationPrototypeId,
    table_name: "qualification_prototypes",
    history_event_label_base: "qualification_prototype",
    history_event_message_name: "Qualification Prototype"
}

impl HasPrototypeContext<QualificationPrototypeContext> for QualificationPrototype {
    fn context(&self) -> QualificationPrototypeContext {
        let mut context = QualificationPrototypeContext::new();
        context.set_component_id(self.component_id);
        context.set_schema_id(self.schema_id);
        context.set_schema_variant_id(self.schema_variant_id);

        context
    }

    fn new_context() -> QualificationPrototypeContext {
        QualificationPrototypeContext::new()
    }
}

impl QualificationPrototype {
    #[allow(clippy::too_many_arguments)]
    #[instrument(skip_all)]
    pub async fn new(
        ctx: &DalContext,
        func_id: FuncId,
        context: QualificationPrototypeContext,
    ) -> QualificationPrototypeResult<Self> {
        let row = ctx
            .txns()
            .pg()
            .query_one(
                "SELECT object FROM qualification_prototype_create_v1($1, $2, $3, $4, $5, $6)",
                &[
                    ctx.write_tenancy(),
                    ctx.visibility(),
                    &func_id,
                    &context.component_id(),
                    &context.schema_id(),
                    &context.schema_variant_id(),
                ],
            )
            .await?;
        let object = standard_model::finish_create_from_row(ctx, row).await?;
        Ok(object)
    }

    standard_model_accessor!(func_id, Pk(FuncId), QualificationPrototypeResult);
    standard_model_accessor!(schema_id, Pk(SchemaId), QualificationPrototypeResult);
    standard_model_accessor!(
        schema_variant_id,
        Pk(SchemaVariantId),
        QualificationPrototypeResult
    );
    standard_model_accessor!(component_id, Pk(ComponentId), QualificationPrototypeResult);

    #[allow(clippy::too_many_arguments)]
    pub async fn find_for_component(
        ctx: &DalContext,
        component_id: ComponentId,
        schema_id: SchemaId,
        schema_variant_id: SchemaVariantId,
    ) -> QualificationPrototypeResult<Vec<Self>> {
        let rows = ctx
            .txns()
            .pg()
            .query(
                FIND_FOR_CONTEXT,
                &[
                    ctx.read_tenancy(),
                    ctx.visibility(),
                    &component_id,
                    &schema_variant_id,
                    &schema_id,
                ],
            )
            .await?;

        Ok(objects_from_rows(rows)?)
    }
}

impl_prototype_list_for_func! {model: QualificationPrototype}
