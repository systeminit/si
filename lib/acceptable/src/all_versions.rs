/// Marks a type as a container over all possible versions of a message type.
pub trait AllVersions {
    /// The current version of the message type.
    fn version(&self) -> u64;

    /// A list of all possible versions that this type supports.
    fn all_versions() -> &'static [u64];
}
