use chrono::{DateTime, Utc};
use std::str::FromStr;

use crate::ulid::Ulid;
use postgres_types::FromSql;
use postgres_types::ToSql;
use serde::Deserialize;
use serde::Serialize;

use crate::ContentHash;
use crate::UserPk;

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Hash, Clone)]
pub struct FuncExecutionKey {
    action_id: Ulid,
    changeset_id: Ulid,
    component_id: Ulid,
    pub func_execution_id: Option<ContentHash>,
    pub message_id: Option<ContentHash>,
    prototype_id: Ulid,
}

impl FuncExecutionKey {
    pub fn new(
        action_id: Ulid,
        changeset_id: Ulid,
        component_id: Ulid,
        prototype_id: Ulid,
    ) -> Self {
        Self {
            action_id,
            changeset_id,
            component_id,
            func_execution_id: None,
            message_id: None,
            prototype_id,
        }
    }

    pub fn action_id(&self) -> &Ulid {
        &self.action_id
    }

    pub fn component_id(&self) -> &Ulid {
        &self.component_id
    }

    pub fn func_execution_id(&self) -> Option<&ContentHash> {
        self.func_execution_id.as_ref()
    }

    pub fn message_id(&self) -> Option<&ContentHash> {
        self.message_id.as_ref()
    }

    pub fn prototype_id(&self) -> &Ulid {
        &self.prototype_id
    }
}

#[remain::sorted]
#[derive(
    Deserialize, Serialize, Debug, strum::EnumString, strum::Display, Eq, PartialEq, ToSql, Clone,
)]
pub enum FuncExecutionState {
    Create,
    Dispatch,
    Failure,
    Run,
    Start,
    Success,
}

impl<'a> postgres_types::FromSql<'a> for FuncExecutionState {
    fn from_sql(
        ty: &postgres_types::Type,
        raw: &'a [u8],
    ) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        let value: &str = FromSql::from_sql(ty, raw)?;
        Ok(FuncExecutionState::from_str(value)?)
    }

    fn accepts(ty: &postgres_types::Type) -> bool {
        ty == &postgres_types::Type::TEXT
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct FuncExecution {
    name: String,

    state: FuncExecutionState,
    started_at: Option<DateTime<Utc>>,

    finished_at: Option<DateTime<Utc>>,
    created_by: Option<UserPk>,
}

impl FuncExecution {
    pub fn new(name: String, state: FuncExecutionState) -> Self {
        Self {
            name,
            state,
            started_at: None,
            finished_at: None,
            created_by: None,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Eq, PartialEq, ToSql)]
pub struct FuncExecutionMessage {
    message: String,
}

impl FuncExecutionMessage {
    pub fn new(message: String) -> Self {
        Self { message }
    }
    pub fn message(&self) -> &str {
        &self.message
    }
}
