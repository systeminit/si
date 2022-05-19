use serde::{Deserialize, Serialize};

use crate::{CodeGenerated, ComponentView};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QualificationCheckRequest {
    pub execution_id: String,
    pub handler: String,
    pub component: QualificationCheckComponent,
    pub code_base64: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct QualificationCheckComponent {
    pub data: ComponentView,
    pub parents: Vec<ComponentView>,
    pub codes: Vec<CodeGenerated>,
}

// Note: these map 1:1 to the DAL qualificationsubcheck data in the qualification view.
//       perhaps they should live permanently here, and be exported via veritech?
//       for now I'm duplicating, so we can experiement with it.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub enum QualificationSubCheckStatus {
    Success,
    Failure,
    Unknown,
}

impl Default for QualificationSubCheckStatus {
    fn default() -> Self {
        QualificationSubCheckStatus::Unknown
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, Default, PartialEq, Eq)]
pub struct QualificationSubCheck {
    pub description: String,
    pub status: QualificationSubCheckStatus,
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QualificationCheckResultSuccess {
    pub execution_id: String,
    pub qualified: bool,
    pub title: Option<String>,
    pub link: Option<String>,
    pub sub_checks: Vec<QualificationSubCheck>,
    pub message: Option<String>,
    pub timestamp: u64,
}

#[cfg(feature = "server")]
pub(crate) mod server {
    use super::*;

    #[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct LangServerQualificationCheckResultSuccess {
        pub execution_id: String,
        pub qualified: bool,
        pub title: Option<String>,
        pub link: Option<String>,
        pub sub_checks: Vec<QualificationSubCheck>,
        pub message: Option<String>,
    }

    impl From<LangServerQualificationCheckResultSuccess> for QualificationCheckResultSuccess {
        fn from(value: LangServerQualificationCheckResultSuccess) -> Self {
            Self {
                execution_id: value.execution_id,
                qualified: value.qualified,
                title: value.title,
                link: value.link,
                sub_checks: value.sub_checks,
                message: value.message,
                timestamp: crate::server::timestamp(),
            }
        }
    }
}
