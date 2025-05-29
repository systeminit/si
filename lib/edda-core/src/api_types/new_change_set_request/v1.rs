use naxum_api_types::RequestId;
use serde::{
    Deserialize,
    Serialize,
};
use si_events::{
    ChangeSetId,
    WorkspaceSnapshotAddress,
};

#[derive(Clone, Debug, Deserialize, Eq, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
// NOTE: **do not modify this datatype--it represents a historically stable, versioned request**
pub struct NewChangeSetRequestV1 {
    pub id: RequestId,
    pub base_change_set_id: ChangeSetId,
    pub new_change_set_id: ChangeSetId,
    pub to_snapshot_address: WorkspaceSnapshotAddress,
}

#[cfg(test)]
mod tests {
    use super::{
        super::{
            NewChangeSetRequestVersionsDiscriminants,
            NewChangeSetRequestVersionsDiscriminants::*,
            test::*,
        },
        *,
    };

    const SNAPSHOT_NAME: &str = "serialized";
    const VERSION: NewChangeSetRequestVersionsDiscriminants = V1;

    fn msg() -> NewChangeSetRequestV1 {
        NewChangeSetRequestV1 {
            id: "01JQCVVDHXYX6S9YCV773R13MG".parse().unwrap(),
            to_snapshot_address: WorkspaceSnapshotAddress::new(b"super-snapshot"),
            new_change_set_id: "01JQCVVDHXYX6S9YCV773R13MG".parse().unwrap(),
            base_change_set_id: "01JQCVVDHXYX6S9YCV773R13MG".parse().unwrap(),
        }
    }

    #[test]
    fn serialize() {
        assert_serialize(SNAPSHOT_NAME, VERSION, msg());
    }

    #[test]
    fn deserialize() {
        assert_deserialize(SNAPSHOT_NAME, VERSION, msg());
    }
}
