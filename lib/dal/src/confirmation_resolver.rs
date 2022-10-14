use crate::DalContext;
use serde::{Deserialize, Serialize};
use si_data::PgError;
use std::default::Default;
use telemetry::prelude::*;
use thiserror::Error;

use crate::{
    func::{binding::FuncBindingId, FuncId},
    impl_standard_model, pk, standard_model, standard_model_accessor, standard_model_many_to_many,
    ActionPrototype, ActionPrototypeId, ComponentId, ConfirmationPrototypeId, HistoryEventError,
    SchemaId, SchemaVariantId, StandardModel, StandardModelError, SystemId, Timestamp, Visibility,
    WriteTenancy,
};

#[derive(Error, Debug)]
pub enum ConfirmationResolverError {
    #[error(transparent)]
    Pg(#[from] PgError),
    #[error(transparent)]
    HistoryEvent(#[from] HistoryEventError),
    #[error(transparent)]
    StandardModel(#[from] StandardModelError),
}

pub type ConfirmationResolverResult<T> = Result<T, ConfirmationResolverError>;

const FIND_FOR_PROTOTYPE: &str =
    include_str!("./queries/confirmation_resolver_find_for_prototype.sql");

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct ConfirmationResolverContext {
    pub component_id: ComponentId,
    pub schema_id: SchemaId,
    pub schema_variant_id: SchemaVariantId,
    pub system_id: SystemId,
}

// Hrm - is this a universal resolver context? -- Adam
impl Default for ConfirmationResolverContext {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfirmationResolverContext {
    pub fn new() -> Self {
        ConfirmationResolverContext {
            component_id: ComponentId::NONE,
            schema_id: SchemaId::NONE,
            schema_variant_id: SchemaVariantId::NONE,
            system_id: SystemId::NONE,
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

pk!(ConfirmationResolverPk);
pk!(ConfirmationResolverId);

/// A [`ConfirmationResolver`] joins a [`FuncBinding`](crate::FuncBinding) to the
/// [`ConfirmationResolverContext`] in which its corresponding
/// [`FuncBindingReturnValue`](crate::FuncBindingReturnValue) is consumed.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct ConfirmationResolver {
    pk: ConfirmationResolverPk,
    id: ConfirmationResolverId,
    confirmation_prototype_id: ConfirmationPrototypeId,
    success: bool,
    message: Option<String>,
    func_id: FuncId,
    func_binding_id: FuncBindingId,
    #[serde(flatten)]
    context: ConfirmationResolverContext,
    #[serde(flatten)]
    tenancy: WriteTenancy,
    #[serde(flatten)]
    timestamp: Timestamp,
    #[serde(flatten)]
    visibility: Visibility,
}

impl_standard_model! {
    model: ConfirmationResolver,
    pk: ConfirmationResolverPk,
    id: ConfirmationResolverId,
    table_name: "confirmation_resolvers",
    history_event_label_base: "confirmation_resolver",
    history_event_message_name: "Confirmation Resolver"
}

impl ConfirmationResolver {
    #[allow(clippy::too_many_arguments)]
    #[instrument(skip_all)]
    pub async fn new(
        ctx: &DalContext,
        confirmation_prototype_id: ConfirmationPrototypeId,
        success: bool,
        message: Option<&str>,
        recommended_actions: Vec<ActionPrototype>,
        func_id: FuncId,
        func_binding_id: FuncBindingId,
        context: ConfirmationResolverContext,
    ) -> ConfirmationResolverResult<Self> {
        let row = ctx.txns().pg().query_one(
                "SELECT object FROM confirmation_resolver_create_v1($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)",
                &[ctx.write_tenancy(), ctx.visibility(),
                    &confirmation_prototype_id,
                    &success,
                    &message,
                    &func_id,
                    &func_binding_id,
                    &context.component_id(),
                    &context.schema_id(),
                    &context.schema_variant_id(),
                    &context.system_id(),
                ],
            )
            .await?;

        let object: Self = standard_model::finish_create_from_row(ctx, row).await?;

        for recommended_action in recommended_actions {
            object
                .add_recommended_action(ctx, recommended_action.id())
                .await?;
        }

        Ok(object)
    }

    pub async fn find_for_prototype(
        ctx: &DalContext,
        workflow_prototype_id: &ConfirmationPrototypeId,
        context: ConfirmationResolverContext,
    ) -> ConfirmationResolverResult<Vec<Self>> {
        let rows = ctx
            .txns()
            .pg()
            .query(
                FIND_FOR_PROTOTYPE,
                &[
                    ctx.read_tenancy(),
                    ctx.visibility(),
                    workflow_prototype_id,
                    &context.component_id(),
                    &context.schema_id(),
                    &context.schema_variant_id(),
                    &context.system_id(),
                ],
            )
            .await?;
        let object = standard_model::objects_from_rows(rows)?;
        Ok(object)
    }

    standard_model_accessor!(
        confirmation_prototype_id,
        Pk(ConfirmationPrototypeId),
        ConfirmationResolverResult
    );
    standard_model_accessor!(func_id, Pk(FuncId), ConfirmationResolverResult);
    standard_model_accessor!(
        func_binding_id,
        Pk(FuncBindingId),
        ConfirmationResolverResult
    );

    standard_model_accessor!(success, bool, ConfirmationResolverResult);
    standard_model_accessor!(message, Option<String>, ConfirmationResolverResult);
    standard_model_many_to_many!(
        lookup_fn: recommended_actions,
        associate_fn: add_recommended_action,
        disassociate_fn: remove_recommended_action,
        disassociate_all_fn: remove_all_recommended_actions,
        table_name: "confirmation_resolvers_many_to_many_action_prototypes",
        left_table: "confirmation_resolvers",
        left_id: ConfirmationResolverId,
        right_table: "action_prototypes",
        right_id: ActionPrototypeId,
        which_table_is_this: "left",
        returns: ActionPrototype,
        result: ConfirmationResolverResult,
    );

    pub fn context(&self) -> ConfirmationResolverContext {
        self.context.clone()
    }
}

#[cfg(test)]
mod test {
    use super::ConfirmationResolverContext;

    #[test]
    fn context_builder() {
        let mut c = ConfirmationResolverContext::new();
        c.set_component_id(15.into());
        assert_eq!(c.component_id(), 15.into());
    }
}
