use thiserror::Error;

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
    #[error("invalid query comparison option")]
    InvalidQueryComparison,
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
}
