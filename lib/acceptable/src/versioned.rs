use crate::id::RequestId;

pub trait Versioned {
    fn id(&self) -> RequestId;

    fn message_version() -> u64;

    #[inline]
    fn version(&self) -> u64 {
        Self::message_version()
    }
}
