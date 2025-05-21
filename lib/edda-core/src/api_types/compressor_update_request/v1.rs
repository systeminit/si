use std::collections::HashMap;

use naxum_api_types::RequestId;
use serde::{
    Deserialize,
    Serialize,
};
use si_events::{
    WorkspaceSnapshotAddress,
    workspace_snapshot::Change,
};
use si_id::EntityId;

#[derive(Clone, Debug, Deserialize, Eq, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
// NOTE: **do not modify this datatype--it represents a historically stable, versioned request**
pub struct CompressorUpdateRequestV1 {
    pub id: RequestId,
    pub previous_ids: Vec<RequestId>,
    pub from_snapshot_address: WorkspaceSnapshotAddress,
    pub to_snapshot_address: WorkspaceSnapshotAddress,
    pub changes: HashMap<EntityId, Change>,
}

#[cfg(test)]
mod tests {
    use si_events::{
        merkle_tree_hash::MerkleTreeHash,
        workspace_snapshot::EntityKind,
    };

    use super::{
        super::{
            CompressorUpdateRequestVersionsDiscriminants,
            CompressorUpdateRequestVersionsDiscriminants::*,
            test::*,
        },
        *,
    };

    const SNAPSHOT_NAME: &str = "serialized";
    const VERSION: CompressorUpdateRequestVersionsDiscriminants = V1;

    fn msg() -> CompressorUpdateRequestV1 {
        CompressorUpdateRequestV1 {
            id: "01JQCVVDHXYX6S9YCV773R13MG".parse().unwrap(),
            previous_ids: Vec::new(),
            from_snapshot_address: WorkspaceSnapshotAddress::new(b"super-snapshot"),
            to_snapshot_address: WorkspaceSnapshotAddress::new(b"super-snapshot"),
            changes: HashMap::from_iter([(
                "01JQCVVDHXYX6S9YCV773R13MG".parse().unwrap(),
                Change {
                    entity_id: "01JQCVVDHXYX6S9YCV773R13MG".parse().unwrap(),
                    entity_kind: EntityKind::Component,
                    merkle_tree_hash: MerkleTreeHash::new(b"todd-howard"),
                },
            )]),
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
