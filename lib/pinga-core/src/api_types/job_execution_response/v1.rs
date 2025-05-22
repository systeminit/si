use naxum_api_types::RequestId;
use serde::{
    Deserialize,
    Serialize,
};
use si_id::{
    ChangeSetId,
    WorkspacePk,
};
use strum::{
    AsRefStr,
    Display,
    EnumIter,
    EnumString,
};

#[derive(Clone, Debug, Deserialize, Eq, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
// NOTE: **do not modify this datatype--it represents a historically stable, versioned request**
pub struct JobExecutionResponseV1 {
    pub id: RequestId,
    pub workspace_id: WorkspacePk,
    pub change_set_id: ChangeSetId,
    pub result: JobExecutionResultV1,
}

#[remain::sorted]
#[derive(
    AsRefStr, Clone, Debug, Deserialize, Display, EnumIter, EnumString, Eq, PartialEq, Serialize,
)]
#[serde(rename_all = "PascalCase")]
// NOTE: **do not modify this datatype--it represents a historically stable, versioned request**
pub enum JobExecutionResultV1 {
    Err { message: String },
    Ok,
}

#[cfg(test)]
mod tests {
    use super::{
        super::{
            JobExecutionResponseVersionsDiscriminants,
            JobExecutionResponseVersionsDiscriminants::*,
            test::*,
        },
        *,
    };

    const VERSION: JobExecutionResponseVersionsDiscriminants = V1;

    fn msg_success() -> JobExecutionResponseV1 {
        JobExecutionResponseV1 {
            id: "01JVWW3987YMPQP4P4TF0AJGTW".parse().unwrap(),
            workspace_id: "01JVWW42D20TT4W6E12EX2HZHF".parse().unwrap(),
            change_set_id: "01JVWW4VJTSE7KW5PAC1NDHGNC".parse().unwrap(),
            result: JobExecutionResultV1::Ok,
        }
    }

    fn msg_failure() -> JobExecutionResponseV1 {
        JobExecutionResponseV1 {
            id: "01JVWWKN64WJJ6H0YFS9SP0J62".parse().unwrap(),
            workspace_id: "01JVWWKVYN0C5R44CYVWQBV2N7".parse().unwrap(),
            change_set_id: "01JVWWM228PDFDE00NA7KGC5EX".parse().unwrap(),
            result: JobExecutionResultV1::Err {
                message: "failed to execute job, too many gerbils".to_string(),
            },
        }
    }

    #[test]
    fn serialize_success() {
        assert_serialize("serialized-success", VERSION, msg_success());
    }

    #[test]
    fn serialize_failure() {
        assert_serialize("serialized-failure", VERSION, msg_failure());
    }

    #[test]
    fn deserialize_success() {
        assert_deserialize("serialized-success", VERSION, msg_success());
    }

    #[test]
    fn deserialize_failure() {
        assert_deserialize("serialized-failure", VERSION, msg_failure());
    }
}
