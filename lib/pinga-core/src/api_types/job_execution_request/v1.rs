use naxum_api_types::RequestId;
use serde::{
    Deserialize,
    Serialize,
};
use si_id::{
    ActionId,
    AttributeValueId,
    ChangeSetId,
    WorkspacePk,
};
use strum::{
    AsRefStr,
    Display,
    EnumDiscriminants,
    IntoStaticStr,
};

#[derive(Clone, Debug, Deserialize, Eq, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
// NOTE: **do not modify this datatype--it represents a historically stable, versioned request**
pub struct JobExecutionRequestV1 {
    pub id: RequestId,
    pub workspace_id: WorkspacePk,
    pub change_set_id: ChangeSetId,
    pub args: JobArgsV1,
    pub is_job_blocking: bool,
}

#[remain::sorted]
#[derive(
    AsRefStr,
    Clone,
    Debug,
    Deserialize,
    Display,
    EnumDiscriminants,
    Eq,
    Hash,
    IntoStaticStr,
    PartialEq,
    Serialize,
)]
#[serde(rename_all = "camelCase")]
#[strum_discriminants(derive(Hash))]
// NOTE: **do not modify this datatype--it represents a historically stable, versioned request**
pub enum JobArgsV1 {
    #[serde(rename_all = "camelCase")]
    Action {
        action_id: ActionId,
    },
    DependentValuesUpdate,
    #[serde(rename_all = "camelCase")]
    Validation {
        attribute_value_ids: Vec<AttributeValueId>,
    },
}

#[cfg(test)]
mod tests {
    use super::{
        super::{
            JobExecutionRequestVersionsDiscriminants,
            JobExecutionRequestVersionsDiscriminants::*,
            test::*,
        },
        *,
    };

    const VERSION: JobExecutionRequestVersionsDiscriminants = V1;

    fn msg_action() -> JobExecutionRequestV1 {
        JobExecutionRequestV1 {
            id: "01JQCVVDHXYX6S9YCV773R13MG".parse().unwrap(),
            workspace_id: "01JVRJNMKGRDDSVQ5Y72R0R9F8".parse().unwrap(),
            change_set_id: "01JVRJSRHT964D53MWMTXYBXVK".parse().unwrap(),
            args: JobArgsV1::Action {
                action_id: "01JVX336AZEFB82369PQGMCTM4".parse().unwrap(),
            },
            is_job_blocking: true,
        }
    }

    fn msg_dependent_values_update() -> JobExecutionRequestV1 {
        JobExecutionRequestV1 {
            id: "01JQCVVDHXYX6S9YCV773R13MG".parse().unwrap(),
            workspace_id: "01JVRJNMKGRDDSVQ5Y72R0R9F8".parse().unwrap(),
            change_set_id: "01JVRJSRHT964D53MWMTXYBXVK".parse().unwrap(),
            args: JobArgsV1::DependentValuesUpdate,
            is_job_blocking: true,
        }
    }

    fn msg_validation() -> JobExecutionRequestV1 {
        JobExecutionRequestV1 {
            id: "01JQCVVDHXYX6S9YCV773R13MG".parse().unwrap(),
            workspace_id: "01JVRJNMKGRDDSVQ5Y72R0R9F8".parse().unwrap(),
            change_set_id: "01JVRJSRHT964D53MWMTXYBXVK".parse().unwrap(),
            args: JobArgsV1::Validation {
                attribute_value_ids: vec![
                    "01JVX3710QXN05Q6QEZ0FRHW5T".parse().unwrap(),
                    "01JVX37HB74AFWTDGEMTBQY5NM".parse().unwrap(),
                ],
            },
            is_job_blocking: true,
        }
    }

    #[test]
    fn serialize_action() {
        assert_serialize("serialized-action", VERSION, msg_action());
    }

    #[test]
    fn serialize_dependent_values_update() {
        assert_serialize(
            "serialized-dependent-values-update",
            VERSION,
            msg_dependent_values_update(),
        );
    }

    #[test]
    fn serialize_validation() {
        assert_serialize("serialized-validation", VERSION, msg_validation());
    }

    #[test]
    fn deserialize_action() {
        assert_deserialize("serialized-action", VERSION, msg_action());
    }

    #[test]
    fn deserialize_dependent_values_update() {
        assert_deserialize(
            "serialized-dependent-values-update",
            VERSION,
            msg_dependent_values_update(),
        );
    }

    #[test]
    fn deserialize_validation() {
        assert_deserialize("serialized-validation", VERSION, msg_validation());
    }
}
