use naxum_api_types::RequestId;
use serde::{
    Deserialize,
    Serialize,
};

#[derive(Clone, Debug, Deserialize, Eq, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
// NOTE: **do not modify this datatype--it represents a historically stable, versioned request**
pub struct CompressorRebuildRequestV1 {
    pub id: RequestId,
    pub previous_ids: Vec<RequestId>,
}

#[cfg(test)]
mod tests {
    use super::{
        super::{
            CompressorRebuildRequestVersionsDiscriminants,
            CompressorRebuildRequestVersionsDiscriminants::*,
            test::*,
        },
        *,
    };

    const SNAPSHOT_NAME: &str = "serialized";
    const VERSION: CompressorRebuildRequestVersionsDiscriminants = V1;

    fn msg() -> CompressorRebuildRequestV1 {
        CompressorRebuildRequestV1 {
            id: "01JQCVVDHXYX6S9YCV773R13MG".parse().unwrap(),
            previous_ids: Vec::new(),
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
