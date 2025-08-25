pub trait AllVersions {
    fn version(&self) -> u64;

    fn all_versions() -> &'static [u64];
}
