use cyclone_core::{CodeGenerated, CodeGenerationResultSuccess};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LangServerCodeGenerationResultSuccess {
    pub execution_id: String,
    pub data: CodeGenerated,
}

impl From<LangServerCodeGenerationResultSuccess> for CodeGenerationResultSuccess {
    fn from(value: LangServerCodeGenerationResultSuccess) -> Self {
        Self {
            execution_id: value.execution_id,
            data: value.data,
            timestamp: crate::timestamp(),
        }
    }
}
