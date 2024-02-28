use std::default::Default;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use si_data_nats::NatsError;
use si_data_pg::PgError;
use telemetry::prelude::*;

use crate::{
    pk, ComponentId, FuncId, HistoryEventError, SchemaVariantId, StandardModelError,
    TransactionsError, WsEventError,
};

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

pk!(AuthenticationPrototypeId);

// An ActionPrototype joins a `FuncId` to a `SchemaVariantId` with a `ActionKind` and `name`
// This only exists for deserialization of the import data
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct AuthenticationPrototype {
    pub id: AuthenticationPrototypeId,
    pub func_id: FuncId,
    pub schema_variant_id: SchemaVariantId,
}
