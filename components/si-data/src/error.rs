use thiserror::Error;
use tonic;

pub type Result<T> = std::result::Result<T, DataError>;

#[derive(Error, Debug)]
pub enum DataError {
    #[error("cannot decode base64 value")]
    Base64DecodeError(#[from] base64::DecodeError),
    #[error("cannot get an incremented change set entry id")]
    ChangeSetEntryUpdateFailure,
    #[error("couchbase error: {0}")]
    CouchbaseError(#[from] couchbase::CouchbaseError),
    #[error("cannot encode data via cbor")]
    CborEncodeError(#[from] serde_cbor::error::Error),
    #[error("must have an id field")]
    MissingId,
    #[error("must have at least one tenant id")]
    MissingTenantIds,
    #[error("a listed tenant id does not exist: {0}")]
    TenantIdIntegrity(String),
    #[error("an object with this naturalKey ({0}) already exists")]
    NaturalKeyExists(String),
    #[error("this object must have a naturalKey, and it is missing")]
    NaturalKeyMissing,
    #[error("invalid boolean logic field")]
    InvalidBooleanLogic,
    #[error("invalid field type")]
    InvalidFieldType,
    #[error("invalid query options")]
    InvalidDataQueryItems,
    #[error("invalid query comparison option")]
    InvalidDataQueryComparison,
    #[error("invalid order by direction; should be ASC or DESC")]
    InvalidOrderByDirection,
    #[error("invalid order by field")]
    InvalidOrderBy,
    #[error("referential integrity error on {0} for id {1}")]
    ReferentialIntegrity(String, String),
    #[error("failed to parse an integer: {0}")]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("cannot encode data via prost")]
    ProstEncodeError(#[from] prost::EncodeError),
    #[error("cannot decode data via prost")]
    ProstDecodeError(#[from] prost::DecodeError),
    #[error("serde json error: {0}")]
    SerdeJson(#[from] serde_json::error::Error),
    #[error("cannot open a sodium oxide secret box")]
    SodiumOxideOpen,
    #[error("error validating item for insertion: {0}")]
    ValidationError(String),
    #[error("a field was required, and it was absent: {0})")]
    RequiredField(String),
    #[error("only one of property fields '{0}' or '{1}' can be set")]
    MultipleEithersProvided2(String, String),
    #[error("only one of property fields '{0}', '{1}', or '{2}' can be set")]
    MultipleEithersProvided3(String, String, String),
    #[error("neither one of property fields '{0}' nor '{1}' are set and one is required")]
    NeitherEithersProvided2(String, String),
    #[error("neither one of property fields '{0}', '{1}', nor '{2}' are set and one is required")]
    NeitherEithersProvided3(String, String, String),
    #[error("cannot encrypt an empty password")]
    EmptyPassword,
    #[error("failed to hash a password")]
    PasswordHash,
    #[error("UTF-8 String conversion error")]
    Utf8(#[from] std::str::Utf8Error),
    #[error("Missing a scope by tenant id for a list operation")]
    MissingScopeByTenantId,
    #[error("error picking a component: {0}")]
    PickComponent(String),
}

impl From<DataError> for tonic::Status {
    fn from(err: DataError) -> tonic::Status {
        match err {
            DataError::ValidationError(_) => {
                tonic::Status::new(tonic::Code::InvalidArgument, err.to_string())
            }
            _ => tonic::Status::new(tonic::Code::InvalidArgument, err.to_string()),
        }
    }
}

pub fn required_field_err(field: impl Into<String>) -> DataError {
    DataError::RequiredField(field.into())
}
