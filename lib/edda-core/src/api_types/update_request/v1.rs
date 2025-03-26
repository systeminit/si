use naxum_api_types::RequestId;
use serde::{Deserialize, Serialize};
use si_events::{ChangeSetId, WorkspacePk, WorkspaceSnapshotAddress};

/// A request to update materialized views in a Frigg store.
#[derive(Clone, Debug, Deserialize, Eq, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
// NOTE: **do not modify this datatype--it represents a historically stable, versioned request**
pub struct UpdateRequestV1 {
    /// A unique ID for this request.
    pub id: RequestId,
    /// The workspace ID.
    pub workspace_id: WorkspacePk,
    /// The change set ID.
    pub change_set_id: ChangeSetId,
    /// The workspace snapshot from which to compute changes.
    ///
    /// - A `Some(addr)` denotes a request coming from the result of processing a Rebaser request
    /// - A `None` would be a request to compute all missing materialized views for the change set,
    ///   and is likely from another serivce such as SDF
    pub from_snapshot_address: Option<WorkspaceSnapshotAddress>,
}

#[cfg(test)]
mod tests {
    use super::{
        super::{
            test::*, UpdateRequestVersionsDiscriminants, UpdateRequestVersionsDiscriminants::*,
        },
        *,
    };

    const SNAPSHOT_NAME: &str = "serialized";
    const VERSION: UpdateRequestVersionsDiscriminants = V1;

    fn msg() -> UpdateRequestV1 {
        UpdateRequestV1 {
            id: "01JQCVVDHXYX6S9YCV773R13MG".parse().unwrap(),
            workspace_id: "01JQCVZPRBGEXQ0CY905F23CE8".parse().unwrap(),
            change_set_id: "01JQCVZYFTZF6BM6NVJBACXZMW".parse().unwrap(),
            from_snapshot_address: Some(WorkspaceSnapshotAddress::new(b"super-snapshot")),
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
