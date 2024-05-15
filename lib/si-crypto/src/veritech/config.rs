use serde::{Deserialize, Serialize};
use si_std::CanonicalFile;

/// Configuration for how to load the key for [`CryptoConfig`].
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct VeritechCryptoConfig {
    /// Key file encoded as a base64 string
    pub encryption_key_base64: Option<String>,
    /// Key file on disk
    pub encryption_key_file: Option<CanonicalFile>,
    /// Key file encoded as a base64 string
    pub decryption_key_base64: Option<String>,
    /// Key file on disk
    pub decryption_key_file: Option<CanonicalFile>,
}
