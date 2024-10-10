use std::ops;

use spicedb_grpc::authzed::api::v1;

/// ZedToken is used to provide causality metadata between Write and Check requests.
///
/// See the authzed.api.v1.Consistency message for more information.
#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ZedToken(String);

impl ops::Deref for ZedToken {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.0.as_str()
    }
}

impl From<v1::ZedToken> for ZedToken {
    fn from(value: v1::ZedToken) -> Self {
        Self(value.token)
    }
}

#[derive(Clone, Debug)]
pub struct ReadSchemaResponse {
    /// schema_text is the textual form of the current schema in the system
    pub schema_text: String,

    /// read_at is the ZedToken at which the schema was read.
    read_at: Option<ZedToken>,
}

impl ReadSchemaResponse {
    pub fn schema_text(&self) -> &str {
        &self.schema_text
    }

    pub fn read_at(&self) -> Option<&ZedToken> {
        self.read_at.as_ref()
    }
}

impl From<v1::ReadSchemaResponse> for ReadSchemaResponse {
    fn from(value: v1::ReadSchemaResponse) -> Self {
        Self {
            schema_text: value.schema_text,
            read_at: value.read_at.map(|value| value.into()),
        }
    }
}
