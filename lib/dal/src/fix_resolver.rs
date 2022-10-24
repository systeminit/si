use crate::DalContext;
use serde::{Deserialize, Serialize};
use si_data::PgError;
use std::default::Default;
use telemetry::prelude::*;
use thiserror::Error;

use crate::{
    impl_standard_model, pk, standard_model, standard_model_accessor, ComponentId,
    ConfirmationResolverId, HistoryEventError, SchemaId, SchemaVariantId, StandardModel,
    StandardModelError, SystemId, Timestamp, Visibility, WorkflowPrototypeId, WriteTenancy,
};

#[derive(Error, Debug)]
pub enum FixResolverError {
    #[error(transparent)]
    Pg(#[from] PgError),
    #[error(transparent)]
    HistoryEvent(#[from] HistoryEventError),
    #[error(transparent)]
    StandardModel(#[from] StandardModelError),
}

pub type FixResolverResult<T> = Result<T, FixResolverError>;

const FIND_FOR_CONFIRMATION: &str =
    include_str!("./queries/fix_resolver_find_for_confirmation.sql");

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct FixResolverContext {
    pub component_id: ComponentId,
    pub schema_id: SchemaId,
    pub schema_variant_id: SchemaVariantId,
    pub system_id: SystemId,
}

// Hrm - is this a universal resolver context? -- Adam
impl Default for FixResolverContext {
    fn default() -> Self {
        Self::new()
    }
}

impl FixResolverContext {
    pub fn new() -> Self {
        FixResolverContext {
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

pk!(FixResolverPk);
pk!(FixResolverId);

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct FixResolver {
    pk: FixResolverPk,
    id: FixResolverId,
    workflow_prototype_id: WorkflowPrototypeId,
    confirmation_resolver_id: ConfirmationResolverId,
    #[serde(flatten)]
    context: FixResolverContext,
    #[serde(flatten)]
    tenancy: WriteTenancy,
    #[serde(flatten)]
    timestamp: Timestamp,
    #[serde(flatten)]
    visibility: Visibility,
}

impl_standard_model! {
    model: FixResolver,
    pk: FixResolverPk,
    id: FixResolverId,
    table_name: "fix_resolvers",
    history_event_label_base: "fix_resolver",
    history_event_message_name: "Fix Resolver"
}

impl FixResolver {
    #[allow(clippy::too_many_arguments)]
    #[instrument(skip_all)]
    pub async fn new(
        ctx: &DalContext,
        workflow_prototype_id: WorkflowPrototypeId,
        confirmation_resolver_id: ConfirmationResolverId,
        context: FixResolverContext,
    ) -> FixResolverResult<Self> {
        let row = ctx
            .txns()
            .pg()
            .query_one(
                "SELECT object FROM fix_resolver_create_v1($1, $2, $3, $4, $5, $6, $7, $8)",
                &[
                    ctx.write_tenancy(),
                    ctx.visibility(),
                    &workflow_prototype_id,
                    &confirmation_resolver_id,
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

    pub async fn find_for_confirmation(
        ctx: &DalContext,
        confirmation_resolver_id: ConfirmationResolverId,
        context: FixResolverContext,
    ) -> FixResolverResult<Option<Self>> {
        let row = ctx
            .txns()
            .pg()
            .query_opt(
                FIND_FOR_CONFIRMATION,
                &[
                    ctx.read_tenancy(),
                    ctx.visibility(),
                    &confirmation_resolver_id,
                    &context.component_id(),
                    &context.schema_id(),
                    &context.schema_variant_id(),
                    &context.system_id(),
                ],
            )
            .await?;
        let object = standard_model::option_object_from_row(row)?;
        Ok(object)
    }

    pub async fn upsert(
        ctx: &DalContext,
        workflow_prototype_id: WorkflowPrototypeId,
        confirmation_resolver_id: ConfirmationResolverId,
        context: FixResolverContext,
    ) -> FixResolverResult<Self> {
        if let Some(mut resolver) =
            Self::find_for_confirmation(ctx, confirmation_resolver_id, context.clone()).await?
        {
            resolver
                .set_workflow_prototype_id(ctx, workflow_prototype_id)
                .await?;
            Ok(resolver)
        } else {
            Ok(Self::new(
                ctx,
                workflow_prototype_id,
                confirmation_resolver_id,
                context,
            )
            .await?)
        }
    }

    standard_model_accessor!(
        workflow_prototype_id,
        Pk(WorkflowPrototypeId),
        FixResolverResult
    );
    standard_model_accessor!(
        confirmation_resolver_id,
        Pk(ConfirmationResolverId),
        FixResolverResult
    );

    pub fn context(&self) -> FixResolverContext {
        self.context.clone()
    }
}

#[cfg(test)]
mod test {
    use super::FixResolverContext;

    #[test]
    fn context_builder() {
        let mut c = FixResolverContext::new();
        c.set_component_id(15.into());
        assert_eq!(c.component_id(), 15.into());
    }
}
