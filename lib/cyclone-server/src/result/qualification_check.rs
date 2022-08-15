use cyclone_core::{QualificationCheckResultSuccess, QualificationSubCheck};
use serde::{Deserialize, Serialize};

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
            timestamp: crate::timestamp(),
        }
    }
}
