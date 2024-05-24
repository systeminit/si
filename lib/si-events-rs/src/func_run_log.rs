use std::str::FromStr;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::{Ulid, ULID_LEN};

use crate::{FuncRunId, Tenancy};

#[derive(Copy, Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct FuncRunLogId(Ulid);

impl FuncRunLogId {
    pub fn new() -> FuncRunLogId {
        FuncRunLogId(Ulid::new())
    }

    pub fn array_to_str<'buf>(&self, buf: &'buf mut [u8; ULID_LEN]) -> &'buf mut str {
        self.0.array_to_str(buf)
    }

    pub fn into_inner(self) -> Ulid {
        self.0
    }
}

impl From<FuncRunLogId> for Ulid {
    fn from(id: FuncRunLogId) -> Self {
        id.0
    }
}

impl Default for FuncRunLogId {
    fn default() -> Self {
        Self::new()
    }
}

impl FromStr for FuncRunLogId {
    type Err = ulid::DecodeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(Ulid::from_str(s)?))
    }
}

impl From<ulid::Ulid> for FuncRunLogId {
    fn from(value: ulid::Ulid) -> Self {
        Self(value)
    }
}

impl std::fmt::Display for FuncRunLogId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// A one-to-one mapping of cyclone's "OutputStream" type.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct OutputLine {
    pub stream: String,
    pub execution_id: String,
    pub level: String,
    pub group: Option<String>,
    pub message: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuncRunLog {
    id: FuncRunLogId,
    tenancy: Tenancy,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    func_run_id: FuncRunId,
    logs: Vec<OutputLine>,
    finalized: bool,
}

impl FuncRunLog {
    pub fn new(func_run_id: FuncRunId, tenancy: Tenancy) -> Self {
        let now = Utc::now();
        Self {
            id: FuncRunLogId::new(),
            tenancy,
            created_at: now,
            updated_at: now,
            func_run_id,
            logs: Vec::new(),
            finalized: false,
        }
    }

    pub fn push_log(&mut self, log: OutputLine) {
        self.logs.push(log);
    }

    pub fn id(&self) -> FuncRunLogId {
        self.id
    }

    pub fn tenancy(&self) -> Tenancy {
        self.tenancy
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    pub fn func_run_id(&self) -> FuncRunId {
        self.func_run_id
    }

    pub fn logs(&self) -> &[OutputLine] {
        self.logs.as_slice()
    }

    pub fn is_finalized(&self) -> bool {
        self.finalized
    }

    pub fn set_finalized(&mut self) {
        self.finalized = true;
    }
}
