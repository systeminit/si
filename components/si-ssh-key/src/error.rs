use base64;
use config::ConfigError;
use couchbase::error::CouchbaseError;
use prost;
use serde_cbor;
use tonic;
use tracing::{event, Level};

use std::{error, fmt, num, result, string};

pub type Result<T> = result::Result<T, Error>;
pub type TonicResult<T> = result::Result<T, tonic::Status>;

#[derive(Debug)]
pub enum Error {
    Base64DecodeError(base64::DecodeError),
    CborEncodeError(serde_cbor::error::Error),
    ComponentNotFound,
    ConfigError(ConfigError),
    CouchbaseError(CouchbaseError),
    FromUtf8Error(string::FromUtf8Error),
    InvalidQueryComparison,
    InvalidBooleanLogic,
    InvalidFieldType,
    InvalidOrderByDirection,
    InvalidKeyType,
    InvalidKeyFormat,
    InvalidTenant,
    IoError(std::io::Error),
    OrderBy,
    ParseIntError(num::ParseIntError),
    ProstEncodeError(prost::EncodeError),
    ProstDecodeError(prost::DecodeError),
    StringError,
    SodiumOxideInit,
    SodiumOxideOpen,
    SshKeyGenError(i32, String, String),
    TonicError(tonic::transport::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let msg = match &*self {
            Error::Base64DecodeError(e) => format!("base64 decode error: {}", e),
            Error::CborEncodeError(e) => format!("CBOR encode error: {}", e),
            Error::ComponentNotFound => format!("Component not found during entity creation; more constraints?"),
            Error::ConfigError(e) => format!("Configuration error: {}", e),
            Error::CouchbaseError(e) => format!("Couchbase error: {}", e),
            Error::FromUtf8Error(e) => format!("Error converting a string to utf-8: {}", e),
            Error::InvalidBooleanLogic => format!("Invalid boolean logic integer"),
            Error::InvalidQueryComparison => format!("Invalid query comparison integer"),
            Error::InvalidFieldType => format!("Invalid field type integer"),
            Error::InvalidKeyType => format!("Invalid key type integer"),
            Error::InvalidKeyFormat => format!("Invalid key format integer"),
            Error::InvalidOrderByDirection => format!("Invalid order by direction"),
            Error::InvalidTenant => format!("Invalid tenant specified for entity"),
            Error::IoError(e) => format!("IO error: {}", e),
            Error::OrderBy => format!("Invalid order_by option"),
            Error::ParseIntError(e) => format!("Failed to parse a string to an integer; this is probably a bad field type setting: {}", e),
            Error::ProstEncodeError(e) => format!("Prost encoding error: {}", e),
            Error::ProstDecodeError(e) => format!("Prost encoding error: {}", e),
            Error::StringError => format!("String conversion error"),
            Error::SodiumOxideInit => format!("Initialization error for sodiumoxide"),
            Error::SodiumOxideOpen => format!("Failed to decrypt a secret box"),
            Error::SshKeyGenError(exit_code, stdout, stderr) => format!("Failed to generate SSH Key.\nCode: {}\n***STDOUT***\n{}\n***STDERR***\n{}", exit_code, stdout, stderr),
            Error::TonicError(e) => format!("Tonic error: {}", e),
        };
        write!(f, "{}", msg)
    }
}

impl error::Error for Error {}

impl From<string::FromUtf8Error> for Error {
    fn from(err: string::FromUtf8Error) -> Error {
        Error::FromUtf8Error(err)
    }
}

impl From<num::ParseIntError> for Error {
    fn from(err: num::ParseIntError) -> Error {
        Error::ParseIntError(err)
    }
}

impl From<base64::DecodeError> for Error {
    fn from(err: base64::DecodeError) -> Error {
        Error::Base64DecodeError(err)
    }
}

impl From<prost::EncodeError> for Error {
    fn from(err: prost::EncodeError) -> Error {
        Error::ProstEncodeError(err)
    }
}

impl From<prost::DecodeError> for Error {
    fn from(err: prost::DecodeError) -> Error {
        Error::ProstDecodeError(err)
    }
}

impl From<serde_cbor::error::Error> for Error {
    fn from(err: serde_cbor::error::Error) -> Error {
        Error::CborEncodeError(err)
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error::IoError(err)
    }
}

impl From<tonic::transport::Error> for Error {
    fn from(err: tonic::transport::Error) -> Error {
        Error::TonicError(err)
    }
}

impl From<ConfigError> for Error {
    fn from(err: ConfigError) -> Error {
        Error::ConfigError(err)
    }
}

impl From<CouchbaseError> for Error {
    fn from(err: CouchbaseError) -> Error {
        Error::CouchbaseError(err)
    }
}

impl From<Error> for tonic::Status {
    fn from(err: Error) -> tonic::Status {
        match err {
            Error::CouchbaseError(CouchbaseError::KeyDoesNotExist) => {
                event!(Level::DEBUG, "no_id");
                tonic::Status::new(tonic::Code::NotFound, "Item not found")
            }
            Error::InvalidTenant => {
                tonic::Status::new(tonic::Code::InvalidArgument, format!("{}", err))
            }
            Error::ComponentNotFound => {
                tonic::Status::new(tonic::Code::InvalidArgument, format!("{}", err))
            }
            Error::OrderBy => tonic::Status::new(tonic::Code::InvalidArgument, format!("{}", err)),
            _ => tonic::Status::new(tonic::Code::Unknown, format!("{}", err)),
        }
    }
}
