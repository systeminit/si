use std::collections::HashMap;

use acceptable::{
    RequestId,
    Versioned,
};
use serde::{
    Deserialize,
    Serialize,
};
use si_events::{
    CachedModuleId,
    SchemaId,
};

#[derive(Clone, Debug, Deserialize, Eq, Serialize, PartialEq, Versioned)]
#[serde(rename_all = "camelCase")]
#[acceptable(version = 1)]
// NOTE: **do not modify this datatype--it represents a historically stable, versioned request**
pub struct RebuildSpecificRequestV1 {
    pub id: RequestId,
    pub removed_schema_ids: Vec<SchemaId>,
    pub new_modules: HashMap<CachedModuleId, SchemaId>,
}

#[cfg(test)]
mod tests {
    use super::{
        super::test::*,
        *,
    };

    const SNAPSHOT_NAME: &str = "serialized";
    const VERSION: u64 = 1;

    fn msg() -> RebuildSpecificRequestV1 {
        RebuildSpecificRequestV1 {
            id: "01JQCVVDHXYX6S9YCV773R13MG".parse().unwrap(),
            removed_schema_ids: vec!["01JQCVVDHXYX6S9YCV773R13MG".parse().unwrap()],
            new_modules: HashMap::from_iter([(
                "01JQCVVDHXYX6S9YCV773R13MG".parse().unwrap(),
                "01JQCVVDHXYX6S9YCV773R13MG".parse().unwrap(),
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
