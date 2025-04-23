use serde::{Deserialize, Serialize};
use si_tls::{CertificateSource, KeySource};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct AuthConfig {
    pub client_cert: Option<CertificateSource>,
    pub client_key: Option<KeySource>,
}
