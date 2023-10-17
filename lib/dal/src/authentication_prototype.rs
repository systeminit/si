use std::default::Default;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use si_data_nats::NatsError;
use si_data_pg::PgError;
use telemetry::prelude::*;

use crate::authentication_prototype::AuthenticationPrototypeError::AuthAlreadySet;
use crate::{
    impl_standard_model, pk, standard_model, standard_model_accessor, ComponentId, DalContext,
    FuncId, HistoryEventError, SchemaVariantId, StandardModel, StandardModelError, Tenancy,
    Timestamp, TransactionsError, Visibility, WsEventError,
};

const FIND_FOR_CONTEXT: &str =
    include_str!("./queries/authentication_prototype/find_for_context.sql");
const FIND_FOR_FUNC: &str = include_str!("./queries/authentication_prototype/find_for_func.sql");
const FIND_FOR_CONTEXT_AND_FUNC: &str =
    include_str!("./queries/authentication_prototype/find_for_context_and_func.sql");

#[remain::sorted]
#[derive(Error, Debug)]
pub enum AuthenticationPrototypeError {
    #[error("this schema variant({0}) already has an authentication function")]
    AuthAlreadySet(SchemaVariantId),
    #[error("component error: {0}")]
    Component(String),
    #[error("component not found: {0}")]
    ComponentNotFound(ComponentId),
    #[error("action Func {0} not found for ActionPrototype {1}")]
    FuncNotFound(FuncId, AuthenticationPrototypeId),
    #[error("history event error: {0}")]
    HistoryEvent(#[from] HistoryEventError),
    #[error("nats txn error: {0}")]
    Nats(#[from] NatsError),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("schema not found")]
    SchemaNotFound,
    #[error("schema variant not found")]
    SchemaVariantNotFound,
    #[error("error serializing/deserializing json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("standard model error: {0}")]
    StandardModelError(#[from] StandardModelError),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error(transparent)]
    WsEvent(#[from] WsEventError),
}

pub type AuthenticationPrototypeResult<T> = Result<T, AuthenticationPrototypeError>;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Copy)]
pub struct AuthenticationPrototypeContext {
    pub schema_variant_id: SchemaVariantId,
}

// Hrm - is this a universal resolver context? -- Adam
impl Default for AuthenticationPrototypeContext {
    fn default() -> Self {
        Self::new(SchemaVariantId::NONE)
    }
}

impl AuthenticationPrototypeContext {
    pub fn new(schema_variant_id: SchemaVariantId) -> Self {
        Self { schema_variant_id }
    }

    pub fn schema_variant_id(&self) -> SchemaVariantId {
        self.schema_variant_id
    }

    pub fn set_schema_variant_id(&mut self, schema_variant_id: SchemaVariantId) {
        self.schema_variant_id = schema_variant_id;
    }
}

pk!(AuthenticationPrototypePk);
pk!(AuthenticationPrototypeId);

// An ActionPrototype joins a `FuncId` to a `SchemaVariantId` with a `ActionKind` and `name`
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct AuthenticationPrototype {
    pk: AuthenticationPrototypePk,
    id: AuthenticationPrototypeId,
    func_id: FuncId,
    schema_variant_id: SchemaVariantId,
    #[serde(flatten)]
    tenancy: Tenancy,
    #[serde(flatten)]
    timestamp: Timestamp,
    #[serde(flatten)]
    visibility: Visibility,
}

impl_standard_model! {
    model: AuthenticationPrototype,
    pk: AuthenticationPrototypePk,
    id: AuthenticationPrototypeId,
    table_name: "authentication_prototypes",
    history_event_label_base: "authentication_prototypes",
    history_event_message_name: "Authentication Prototype"
}

impl AuthenticationPrototype {
    #[allow(clippy::too_many_arguments)]
    #[instrument(skip_all)]
    pub async fn new(
        ctx: &DalContext,
        func_id: FuncId,
        context: AuthenticationPrototypeContext,
    ) -> AuthenticationPrototypeResult<Self> {
        if !Self::find_for_context(ctx, context).await?.is_empty() {
            return Err(AuthAlreadySet(context.schema_variant_id));
        }

        let row = ctx
            .txns()
            .await?
            .pg()
            .query_one(
                "SELECT object FROM authentication_prototype_create_v1($1, $2, $3, $4)",
                &[
                    ctx.tenancy(),
                    ctx.visibility(),
                    &func_id,
                    &context.schema_variant_id(),
                ],
            )
            .await?;
        let object = standard_model::finish_create_from_row(ctx, row).await?;
        Ok(object)
    }

    pub async fn find_for_context(
        ctx: &DalContext,
        context: AuthenticationPrototypeContext,
    ) -> AuthenticationPrototypeResult<Vec<Self>> {
        let rows = ctx
            .txns()
            .await?
            .pg()
            .query(
                FIND_FOR_CONTEXT,
                &[
                    ctx.tenancy(),
                    ctx.visibility(),
                    &context.schema_variant_id(),
                ],
            )
            .await?;

        Ok(standard_model::objects_from_rows(rows)?)
    }

    pub async fn find_for_func(
        ctx: &DalContext,
        func_id: FuncId,
    ) -> AuthenticationPrototypeResult<Vec<Self>> {
        let rows = ctx
            .txns()
            .await?
            .pg()
            .query(FIND_FOR_FUNC, &[ctx.tenancy(), ctx.visibility(), &func_id])
            .await?;

        Ok(standard_model::objects_from_rows(rows)?)
    }

    pub async fn find_for_context_and_func(
        ctx: &DalContext,
        context: &AuthenticationPrototypeContext,
        func_id: FuncId,
    ) -> AuthenticationPrototypeResult<Vec<Self>> {
        let rows = ctx
            .txns()
            .await?
            .pg()
            .query(
                FIND_FOR_CONTEXT_AND_FUNC,
                &[
                    ctx.tenancy(),
                    ctx.visibility(),
                    &context.schema_variant_id(),
                    &func_id,
                ],
            )
            .await?;

        Ok(standard_model::objects_from_rows(rows)?)
    }

    standard_model_accessor!(
        schema_variant_id,
        Pk(SchemaVariantId),
        AuthenticationPrototypeResult
    );
    standard_model_accessor!(func_id, Pk(FuncId), AuthenticationPrototypeResult);
}
