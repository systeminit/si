use naxum_api_types::RequestId;
use serde::{Deserialize, Serialize};
use si_events::{WorkspaceSnapshotAddress, change_batch::ChangeBatchAddress};

#[derive(Clone, Debug, Deserialize, Eq, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
// NOTE: **do not modify this datatype--it represents a historically stable, versioned request**
pub struct UpdateRequestV1 {
    pub id: RequestId,
    pub from_snapshot_address: WorkspaceSnapshotAddress,
    pub to_snapshot_address: WorkspaceSnapshotAddress,
    pub change_batch_address: ChangeBatchAddress,
}

#[cfg(test)]
mod tests {
    use super::{
        super::{
            UpdateRequestVersionsDiscriminants, UpdateRequestVersionsDiscriminants::*, test::*,
        },
        *,
    };

    const SNAPSHOT_NAME: &str = "serialized";
    const VERSION: UpdateRequestVersionsDiscriminants = V1;

    fn msg() -> UpdateRequestV1 {
        UpdateRequestV1 {
            id: "01JQCVVDHXYX6S9YCV773R13MG".parse().unwrap(),
            from_snapshot_address: WorkspaceSnapshotAddress::new(b"super-snapshot"),
            to_snapshot_address: WorkspaceSnapshotAddress::new(b"super-snapshot"),
            change_batch_address: ChangeBatchAddress::new(b"super-snapshot"),
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
