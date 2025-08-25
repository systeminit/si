use crate::id::RequestId;

/// A unique version of a message type.
///
/// Note: all versions of a message type are tracked via a type that implements `AllVersions`.
pub trait Versioned {
    /// A unique identifier representing a single message in a system.
    fn id(&self) -> RequestId;

    /// The message version for this type.
    fn message_version() -> u64;

    /// The message version for this message.
    #[inline]
    fn version(&self) -> u64 {
        Self::message_version()
    }
}
