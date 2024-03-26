/// This key uses a [`blake3::Hash`] wrapper, [`si_hash::Hash`], in order to get de/ser and display
/// implementation benefits.
pub type EncryptedSecretKey = si_hash::Hash;
