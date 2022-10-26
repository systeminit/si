use std::default::Default;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use si_data::PgError;
use si_data_nats::NatsError;
use telemetry::prelude::*;

use crate::{
    func::FuncId,
    impl_prototype_list_for_func, impl_standard_model, pk,
    prototype_context::{HasPrototypeContext, PrototypeContext},
    standard_model::{self, objects_from_rows, TypeHint},
    standard_model_accessor, ComponentId, DalContext, HistoryEvent, HistoryEventError,
    PrototypeListForFuncError, SchemaId, SchemaVariantId, StandardModel, StandardModelError,
    SystemId, Timestamp, Visibility, WriteTenancy,
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
    system_id: SystemId,
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

    fn system_id(&self) -> SystemId {
        self.system_id
    }

    fn set_system_id(&mut self, system_id: SystemId) {
        self.system_id = system_id;
    }
}

impl QualificationPrototypeContext {
    pub fn new() -> Self {
        Self {
            component_id: ComponentId::NONE,
            schema_id: SchemaId::NONE,
            schema_variant_id: SchemaVariantId::NONE,
            system_id: SystemId::NONE,
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
    system_id: SystemId,
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
        context.set_system_id(self.system_id);

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
                "SELECT object FROM qualification_prototype_create_v1($1, $2, $3, $4, $5, $6, $7)",
                &[
                    ctx.write_tenancy(),
                    ctx.visibility(),
                    &func_id,
                    &context.component_id(),
                    &context.schema_id(),
                    &context.schema_variant_id(),
                    &context.system_id(),
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

    standard_model_accessor!(system_id, Pk(SystemId), QualificationPrototypeResult);

    pub async fn set_id(
        &mut self,
        ctx: &DalContext,
        id: &QualificationPrototypeId,
    ) -> QualificationPrototypeResult<()> {
        let updated_at = standard_model::update(
            ctx,
            Self::table_name(),
            "id",
            self.id(),
            id,
            TypeHint::BigInt,
        )
        .await?;
        let _history_event = HistoryEvent::new(
            ctx,
            &Self::history_event_label(vec!["updated"]),
            &Self::history_event_message("updated"),
            &serde_json::json![{
                "pk": self.pk,
                "field": "id",
                "value": id,
            }],
        )
        .await?;
        self.timestamp.updated_at = updated_at;
        self.id = *id;
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn find_for_component(
        ctx: &DalContext,
        component_id: ComponentId,
        schema_id: SchemaId,
        schema_variant_id: SchemaVariantId,
        system_id: SystemId,
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
                    &system_id,
                    &schema_variant_id,
                    &schema_id,
                ],
            )
            .await?;

        Ok(objects_from_rows(rows)?)
    }
}

impl_prototype_list_for_func! {model: QualificationPrototype}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn context_builder() {
        let mut c = QualificationPrototypeContext::new();
        c.set_component_id(22.into());
        assert_eq!(c.component_id(), 22.into());
    }
}
