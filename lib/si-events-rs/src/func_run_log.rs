use chrono::{
    DateTime,
    Utc,
};
use serde::{
    Deserialize,
    Serialize,
};
pub use si_id::FuncRunLogId;

use crate::{
    FuncRunId,
    Tenancy,
};

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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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
