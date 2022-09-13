use cyclone_core::CommandRunResultSuccess;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LangServerCommandRunResultSuccess {
    pub execution_id: String,
    // Collects the error if the function throws
    #[serde(default)]
    pub error: Option<String>,
}

impl From<LangServerCommandRunResultSuccess> for CommandRunResultSuccess {
    fn from(value: LangServerCommandRunResultSuccess) -> Self {
        Self {
            execution_id: value.execution_id,
            error: value.error,
        }
    }
}
