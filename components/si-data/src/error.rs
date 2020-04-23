use thiserror::Error;
use tonic;

pub type Result<T> = std::result::Result<T, DataError>;

#[derive(Error, Debug)]
pub enum DataError {
    #[error("cannot decode base64 value")]
    Base64DecodeError(#[from] base64::DecodeError),
    #[error("couchbase error: {0}")]
    CouchbaseError(#[from] couchbase::CouchbaseError),
    #[error("cannot encode data via cbor")]
    CborEncodeError(#[from] serde_cbor::error::Error),
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
    #[error("cannot open a sodium oxide secret box")]
    SodiumOxideOpen,
    #[error("error validating item for insertion: {0}")]
    ValidationError(String),
    #[error("a protobuf field was required, and it was absent: {0})")]
    RequiredField(String),
    #[error("cannot encrypt an empty password")]
    EmptyPassword,
    #[error("failed to hash a password")]
    PasswordHash,
    #[error("UTF-8 String conversion error")]
    Utf8(#[from] std::str::Utf8Error),
    #[error("Missing a scope by tenant id for a list operation")]
    MissingScopeByTenantId,
}

impl From<DataError> for tonic::Status {
    fn from(err: DataError) -> tonic::Status {
        match err {
            DataError::ValidationError(_) => {
                tonic::Status::new(tonic::Code::InvalidArgument, err.to_string())
            }
            _ => tonic::Status::new(tonic::Code::Unknown, err.to_string()),
        }
    }
}
