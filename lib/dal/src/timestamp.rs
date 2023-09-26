use chrono::{DateTime, Utc};
use postgres_types::{FromSql, Type};
use serde::{Deserialize, Serialize};
use std::error::Error;
use thiserror::Error;

#[remain::sorted]
#[derive(Error, Debug)]
pub enum TimestampError {}

pub type TimestampResult<T> = Result<T, TimestampError>;

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone, Copy)]
pub struct Timestamp {
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Timestamp {
    pub fn now() -> Self {
        let now = Utc::now();
        Self {
            created_at: now,
            updated_at: now,
        }
    }
}
