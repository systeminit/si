use serde::{Deserialize, Serialize};

use crate::ComponentView;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeGenerationRequest {
    pub execution_id: String,
    pub handler: String,
    pub component: ComponentView,
    pub code_base64: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CodeGenerated {
    pub format: String,
    pub code: String,
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeGenerationResultSuccess {
    pub execution_id: String,
    pub data: CodeGenerated,
    pub timestamp: u64,
}

#[cfg(feature = "server")]
pub(crate) mod server {
    use super::*;

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
                timestamp: crate::server::timestamp(),
            }
        }
    }
}
