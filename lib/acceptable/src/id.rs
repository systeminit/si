use std::{
    fmt,
    str,
};

use serde::{
    Deserialize,
    Serialize,
};
use ulid::Ulid;

/// A unique identifier representing a single request to a system.
///
/// This value may be used for request message de-duplication, idempotency checks, response
/// metadata, etc.
#[derive(Copy, Clone, Eq, Debug, Deserialize, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct RequestId(Ulid);

impl RequestId {
    /// Length of a string-encoded ID in bytes.
    pub const ID_LEN: usize = ulid::ULID_LEN;

    /// Generates a new key which is virtually guaranteed to be unique.
    pub fn generate() -> Self {
        Self(Ulid::new())
    }

    /// Calls [`Self::generate`].
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self::generate()
    }

    /// Converts type into inner [`Ulid`].
    pub fn into_inner(self) -> Ulid {
        self.0
    }

    /// Creates a Crockford Base32 encoded string that represents this Ulid.
    pub fn array_to_str<'buf>(&self, buf: &'buf mut [u8; Self::ID_LEN]) -> &'buf mut str {
        self.0.array_to_str(buf)
    }

    /// Converts the ID into a byte array.
    pub fn array_to_str_buf() -> [u8; Self::ID_LEN] {
        [0; Self::ID_LEN]
    }

    /// Constructs a new instance of Self from the given raw identifier.
    ///
    /// This function is typically used to consume ownership of the specified identifier.
    pub fn from_raw_id(value: Ulid) -> Self {
        Self(value)
    }

    /// Extracts the raw identifier.
    ///
    /// This function is typically used to borrow an owned identifier.
    pub fn as_raw_id(&self) -> Ulid {
        self.0
    }

    /// Consumes this object, returning the raw underlying identifier.
    ///
    /// This function is typically used to transfer ownership of the underlying identifier
    /// to the caller."
    pub fn into_raw_id(self) -> Ulid {
        self.0
    }
}

impl fmt::Display for RequestId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.to_string())
    }
}

impl From<RequestId> for String {
    fn from(value: RequestId) -> Self {
        value.to_string()
    }
}

impl<'a> From<&'a RequestId> for Ulid {
    fn from(value: &'a RequestId) -> Self {
        value.0
    }
}

impl From<RequestId> for Ulid {
    fn from(value: RequestId) -> Self {
        value.0
    }
}

impl str::FromStr for RequestId {
    type Err = ulid::DecodeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(Ulid::from_string(s)?))
    }
}
