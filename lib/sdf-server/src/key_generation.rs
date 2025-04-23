use std::path::Path;

use si_crypto::{
    SymmetricCryptoError,
    SymmetricCryptoService,
    VeritechKeyPair,
    VeritechKeyPairError,
};
use telemetry::prelude::*;

#[instrument(name = "sdf.util.generate_veritech_key_pair", level = "info", skip_all)]
pub async fn generate_veritech_key_pair(
    secret_key_path: impl AsRef<Path>,
    public_key_path: impl AsRef<Path>,
) -> Result<(), VeritechKeyPairError> {
    VeritechKeyPair::create_and_write_files(secret_key_path, public_key_path).await
}

#[instrument(name = "sdf.util.generate_symmetric_key", level = "info", skip_all)]
pub async fn generate_symmetric_key(
    symmetric_key_path: impl AsRef<Path>,
) -> Result<(), SymmetricCryptoError> {
    SymmetricCryptoService::generate_key()
        .save(symmetric_key_path.as_ref())
        .await
}
