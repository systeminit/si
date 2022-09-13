use std::default::Default;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use si_data::{NatsError, PgError};
use telemetry::prelude::*;

use crate::{
    func::FuncId,
    impl_standard_model, pk,
    standard_model::{self, objects_from_rows, TypeHint},
    standard_model_accessor, ComponentId, DalContext, HistoryEvent, HistoryEventError, SchemaId,
    SchemaVariantId, StandardModel, StandardModelError, SystemId, Timestamp, Visibility,
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
}

pub type QualificationPrototypeResult<T> = Result<T, QualificationPrototypeError>;

const FIND_FOR_CONTEXT: &str =
    include_str!("./queries/qualification_prototype_find_for_context.sql");
const FIND_FOR_FUNC: &str = include_str!("./queries/qualification_prototype_find_for_func.sql");

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

impl QualificationPrototypeContext {
    pub fn new() -> Self {
        Self {
            component_id: ComponentId::NONE,
            schema_id: SchemaId::NONE,
            schema_variant_id: SchemaVariantId::NONE,
            system_id: SystemId::NONE,
        }
    }

    pub fn new_for_context_field(context_field: QualificationPrototypeContextField) -> Self {
        match context_field {
            QualificationPrototypeContextField::Schema(schema_id) => {
                QualificationPrototypeContext {
                    component_id: ComponentId::NONE,
                    schema_id,
                    schema_variant_id: SchemaVariantId::NONE,
                    system_id: SystemId::NONE,
                }
            }
            QualificationPrototypeContextField::System(system_id) => {
                QualificationPrototypeContext {
                    component_id: ComponentId::NONE,
                    schema_id: SchemaId::NONE,
                    schema_variant_id: SchemaVariantId::NONE,
                    system_id,
                }
            }
            QualificationPrototypeContextField::SchemaVariant(schema_variant_id) => {
                QualificationPrototypeContext {
                    component_id: ComponentId::NONE,
                    schema_id: SchemaId::NONE,
                    schema_variant_id,
                    system_id: SystemId::NONE,
                }
            }
            QualificationPrototypeContextField::Component(component_id) => {
                QualificationPrototypeContext {
                    component_id,
                    schema_id: SchemaId::NONE,
                    schema_variant_id: SchemaVariantId::NONE,
                    system_id: SystemId::NONE,
                }
            }
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

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum QualificationPrototypeContextField {
    Component(ComponentId),
    Schema(SchemaId),
    SchemaVariant(SchemaVariantId),
    System(SystemId),
}

impl From<ComponentId> for QualificationPrototypeContextField {
    fn from(component_id: ComponentId) -> Self {
        QualificationPrototypeContextField::Component(component_id)
    }
}

impl From<SchemaId> for QualificationPrototypeContextField {
    fn from(schema_id: SchemaId) -> Self {
        QualificationPrototypeContextField::Schema(schema_id)
    }
}

impl From<SchemaVariantId> for QualificationPrototypeContextField {
    fn from(schema_variant_id: SchemaVariantId) -> Self {
        QualificationPrototypeContextField::SchemaVariant(schema_variant_id)
    }
}

impl From<SystemId> for QualificationPrototypeContextField {
    fn from(system_id: SystemId) -> Self {
        QualificationPrototypeContextField::System(system_id)
    }
}

impl_standard_model! {
    model: QualificationPrototype,
    pk: QualificationPrototypePk,
    id: QualificationPrototypeId,
    table_name: "qualification_prototypes",
    history_event_label_base: "qualification_prototype",
    history_event_message_name: "Qualification Prototype"
}

impl QualificationPrototype {
    #[allow(clippy::too_many_arguments)]
    #[instrument(skip_all)]
    pub async fn new(
        ctx: &DalContext<'_, '_, '_>,
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
        ctx: &DalContext<'_, '_, '_>,
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
        ctx: &DalContext<'_, '_, '_>,
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

    pub async fn find_for_func(
        ctx: &DalContext<'_, '_, '_>,
        func_id: &FuncId,
    ) -> QualificationPrototypeResult<Vec<Self>> {
        let rows = ctx
            .txns()
            .pg()
            .query(
                FIND_FOR_FUNC,
                &[ctx.read_tenancy(), ctx.visibility(), func_id],
            )
            .await?;
        let object = objects_from_rows(rows)?;
        Ok(object)
    }

    pub fn context(&self) -> QualificationPrototypeContext {
        let mut context = QualificationPrototypeContext::new();
        context.set_component_id(self.component_id);
        context.set_schema_id(self.schema_id);
        context.set_schema_variant_id(self.schema_variant_id);
        context.set_system_id(self.system_id);

        context
    }

    async fn create_missing_prototypes(
        ctx: &DalContext<'_, '_, '_>,
        func_id: &FuncId,
        existing_for_field: &[QualificationPrototypeContextField],
        desired_for_field: &[QualificationPrototypeContextField],
    ) -> QualificationPrototypeResult<Vec<Self>> {
        let mut new_protos = vec![];

        for desired in desired_for_field {
            if existing_for_field.contains(desired) {
                continue;
            }

            new_protos.push(
                QualificationPrototype::new(
                    ctx,
                    *func_id,
                    QualificationPrototypeContext::new_for_context_field(*desired),
                )
                .await?,
            );
        }

        Ok(new_protos)
    }

    /// Given a list of `QualificationPrototypeContextField`s (specifically here only
    /// `SchemaVariantId` and `ComponentId` context fields, make them the only context fields for
    /// which we have a prototype connected to the given function by deleting any that are not
    /// in the list and creating any that do not currently exist.
    pub async fn associate_prototypes_with_func_and_objects<
        T: Into<QualificationPrototypeContextField> + Copy,
    >(
        ctx: &DalContext<'_, '_, '_>,
        func_id: &FuncId,
        prototype_context_field_ids: &[T],
    ) -> QualificationPrototypeResult<Vec<Self>> {
        let mut existing_field_ids = vec![];
        let prototype_context_field_ids: Vec<QualificationPrototypeContextField> =
            prototype_context_field_ids
                .iter()
                .map(|field| (*field).into())
                .collect();

        for proto in Self::find_for_func(ctx, func_id).await? {
            let component_id = proto.component_id();
            let schema_variant_id = proto.schema_variant_id();

            if component_id.is_none() && schema_variant_id.is_none() {
                continue;
            }

            if component_id.is_some() && !prototype_context_field_ids.contains(&component_id.into())
                || schema_variant_id.is_some()
                    && !prototype_context_field_ids.contains(&schema_variant_id.into())
            {
                proto.delete(ctx).await?;
                continue;
            } else if component_id.is_some() {
                existing_field_ids.push(component_id.into());
            } else if schema_variant_id.is_some() {
                existing_field_ids.push(schema_variant_id.into());
            }
        }

        Self::create_missing_prototypes(
            ctx,
            func_id,
            &existing_field_ids,
            &prototype_context_field_ids,
        )
        .await
    }
}

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
