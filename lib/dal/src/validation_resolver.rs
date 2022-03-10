use serde::{Deserialize, Serialize};
use si_data::{NatsError, NatsTxn, PgError, PgTxn};
use std::default::Default;
use telemetry::prelude::*;
use thiserror::Error;

use crate::{
    func::{binding::FuncBindingId, binding_return_value::FuncBindingReturnValue, FuncId},
    impl_standard_model, pk,
    standard_model::{self, objects_from_rows},
    standard_model_accessor, ComponentId, HistoryActor, HistoryEventError, Prop, PropId, SchemaId,
    SchemaVariantId, StandardModel, StandardModelError, SystemId, Tenancy, Timestamp,
    ValidationPrototypeId, Visibility,
};

#[derive(Error, Debug)]
pub enum ValidationResolverError {
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
    #[error("invalid prop id")]
    InvalidPropId,
}

pub type ValidationResolverResult<T> = Result<T, ValidationResolverError>;

pub const UNSET_ID_VALUE: i64 = -1;
const FIND_VALUES_FOR_CONTEXT: &str =
    include_str!("./queries/validation_resolver_find_values_for_context.sql");
const FIND_VALUES_FOR_COMPONENT: &str =
    include_str!("./queries/validation_resolver_find_values_for_component.sql");
const FIND_FOR_PROTOTYPE: &str =
    include_str!("./queries/validation_resolver_find_for_prototype.sql");

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct ValidationResolverContext {
    prop_id: PropId,
    component_id: ComponentId,
    schema_id: SchemaId,
    schema_variant_id: SchemaVariantId,
    system_id: SystemId,
}

// Hrm - is this a universal resolver context? -- Adam
impl Default for ValidationResolverContext {
    fn default() -> Self {
        Self::new()
    }
}

impl ValidationResolverContext {
    pub fn new() -> Self {
        ValidationResolverContext {
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

pk!(ValidationResolverPk);
pk!(ValidationResolverId);

// An ValidationResolver joins a `FuncBinding` to the context in which
// its corresponding `FuncBindingResultValue` is consumed.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct ValidationResolver {
    pk: ValidationResolverPk,
    id: ValidationResolverId,
    validation_prototype_id: ValidationPrototypeId,
    func_id: FuncId,
    func_binding_id: FuncBindingId,
    #[serde(flatten)]
    context: ValidationResolverContext,
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
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        tenancy: &Tenancy,
        visibility: &Visibility,
        history_actor: &HistoryActor,
        validation_prototype_id: ValidationPrototypeId,
        func_id: FuncId,
        func_binding_id: FuncBindingId,
        context: ValidationResolverContext,
    ) -> ValidationResolverResult<Self> {
        let row = txn
            .query_one(
                "SELECT object FROM validation_resolver_create_v1($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
                &[
                    &tenancy,
                    &visibility,
                    &validation_prototype_id,
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
        validation_prototype_id,
        Pk(ValidationPrototypeId),
        ValidationResolverResult
    );
    standard_model_accessor!(func_id, Pk(FuncId), ValidationResolverResult);
    standard_model_accessor!(func_binding_id, Pk(FuncBindingId), ValidationResolverResult);

    // #[allow(clippy::too_many_arguments)]
    // #[instrument(skip_all)]
    // pub async fn upsert(
    //     txn: &PgTxn<'_>,
    //     nats: &NatsTxn,
    //     tenancy: &Tenancy,
    //     visibility: &Visibility,
    //     history_actor: &HistoryActor,
    //     func_id: FuncId,
    //     func_binding_id: FuncBindingId,
    //     context: ValidationResolverContext,
    // ) -> ValidationResolverResult<Self> {
    //     let row = txn
    //         .query_one(
    //             "SELECT object FROM validation_resolver_upsert_v1($1, $2, $3, $4, $5, $6, $7, $8, $9)",
    //             &[
    //                 &tenancy,
    //                 &visibility,
    //                 &func_id,
    //                 &func_binding_id,
    //                 &context.prop_id(),
    //                 &context.component_id(),
    //                 &context.schema_id(),
    //                 &context.schema_variant_id(),
    //                 &context.system_id(),
    //             ],
    //         )
    //         .await?;
    //     let object = standard_model::finish_create_from_row(
    //         txn,
    //         nats,
    //         tenancy,
    //         visibility,
    //         history_actor,
    //         row,
    //     )
    //     .await?;
    //     Ok(object)
    // }

    pub async fn list_values_for_component(
        txn: &PgTxn<'_>,
        tenancy: &Tenancy,
        visibility: &Visibility,
        component_id: ComponentId,
        system_id: SystemId,
    ) -> ValidationResolverResult<Vec<(Prop, FuncBindingReturnValue)>> {
        let rows = txn
            .query(
                FIND_VALUES_FOR_COMPONENT,
                &[&tenancy, &visibility, &component_id, &system_id],
            )
            .await?;
        let mut result = Vec::new();
        for row in rows.into_iter() {
            let json: serde_json::Value = row.try_get("object")?;
            let object: FuncBindingReturnValue = serde_json::from_value(json)?;
            let prop_id: PropId = row.try_get("prop_id")?;
            let prop = Prop::get_by_id(txn, tenancy, visibility, &prop_id)
                .await?
                .ok_or(ValidationResolverError::InvalidPropId)?;
            result.push((prop, object));
        }
        Ok(result)
    }

    pub async fn find_values_for_prop_and_component(
        txn: &PgTxn<'_>,
        tenancy: &Tenancy,
        visibility: &Visibility,
        prop_id: PropId,
        component_id: ComponentId,
        system_id: SystemId,
    ) -> ValidationResolverResult<Vec<FuncBindingReturnValue>> {
        let rows = txn
            .query(
                FIND_VALUES_FOR_CONTEXT,
                &[&tenancy, &visibility, &prop_id, &component_id, &system_id],
            )
            .await?;
        let object = objects_from_rows(rows)?;
        Ok(object)
    }

    pub async fn find_for_prototype(
        txn: &PgTxn<'_>,
        tenancy: &Tenancy,
        visibility: &Visibility,
        validation_prototype_id: &ValidationPrototypeId,
    ) -> ValidationResolverResult<Vec<Self>> {
        let rows = txn
            .query(
                FIND_FOR_PROTOTYPE,
                &[&tenancy, &visibility, validation_prototype_id],
            )
            .await?;
        let object = objects_from_rows(rows)?;
        Ok(object)
    }
}

#[cfg(test)]
mod test {
    use super::ValidationResolverContext;

    #[test]
    fn context_builder() {
        let mut c = ValidationResolverContext::new();
        c.set_component_id(15.into());
        c.set_prop_id(22.into());
        assert_eq!(c.component_id(), 15.into());
        assert_eq!(c.prop_id(), 22.into());
    }
}
