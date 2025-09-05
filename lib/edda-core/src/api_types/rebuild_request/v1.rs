use acceptable::{
    RequestId,
    Versioned,
};
use serde::{
    Deserialize,
    Serialize,
};

#[derive(Clone, Debug, Deserialize, Eq, Serialize, PartialEq, Versioned)]
#[serde(rename_all = "camelCase")]
#[acceptable(version = 1)]
// NOTE: **do not modify this datatype--it represents a historically stable, versioned request**
pub struct RebuildRequestV1 {
    pub id: RequestId,
}

#[cfg(test)]
mod tests {
    use super::{
        super::test::*,
        *,
    };

    const SNAPSHOT_NAME: &str = "serialized";
    const VERSION: u64 = 1;

    fn msg() -> RebuildRequestV1 {
        RebuildRequestV1 {
            id: "01JQCVVDHXYX6S9YCV773R13MG".parse().unwrap(),
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
