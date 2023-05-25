use jwt_simple::prelude::{JWTClaims, RS256PublicKey, RSAPublicKeyLike};
use std::{path::Path, pin::Pin, sync::Arc};
use telemetry::prelude::*;
use thiserror::Error;

use tokio::{
    fs,
    io::{AsyncRead, AsyncReadExt},
    task::JoinError,
};

use crate::extract::UserClaim;

#[derive(Clone, Debug)]
pub struct JwtPublicSigningKey {
    inner: Arc<RS256PublicKey>,
}
impl JwtPublicSigningKey {
    #[instrument(level = "debug", skip_all)]
    pub async fn load(path: impl AsRef<Path>) -> JwtKeyResult<Self> {
        trace!(
            path = path.as_ref().to_string_lossy().as_ref(),
            "loading jwt public signing key"
        );
        let mut file = fs::File::open(path).await?;
        Self::from_reader(Pin::new(&mut file)).await
    }

    async fn from_reader(mut reader: Pin<&mut impl AsyncRead>) -> JwtKeyResult<Self> {
        let mut public_key_string = String::new();
        reader.read_to_string(&mut public_key_string).await?;

        let inner = tokio::task::spawn_blocking(move || {
            RS256PublicKey::from_pem(&public_key_string)
                .map_err(|err| JwtKeyError::KeyFromPem(format!("{err}")))
        })
        .instrument(trace_span!(
            "from_pem",
            code.namespace = "jwt_simple::algorithms::RS256PublicKey"
        ))
        .await??;

        Ok(Self {
            inner: Arc::new(inner),
        })
    }
}

pub type JwtKeyResult<T> = Result<T, JwtKeyError>;

#[remain::sorted]
#[derive(Error, Debug)]
pub enum JwtKeyError {
    #[error("bad nonce bytes")]
    BadNonce,
    #[error("failed to decode base64 string: {0}")]
    Base64Decode(#[from] base64::DecodeError),
    #[error("invalid bearer token")]
    BearerToken,
    #[error("failed to decrypt secret data")]
    Decrypt,
    #[error("error generating new keypair")]
    GenerateKeyPair,
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("failed to load jwt encryption key from bytes")]
    JwtEncryptionKeyParse,
    #[error("failure to build signing key from pem: {0}")]
    KeyFromPem(String),
    #[error("failure to extract metadata from bearer token: {0}")]
    Metadata(String),
    #[error("no signing keys - bad news for you!")]
    NoKeys,
    #[error("{0}")]
    TaskJoin(#[from] JoinError),
    #[error("failed to convert into PEM format")]
    ToPem,
    #[error(transparent)]
    UlidDecode(#[from] ulid::DecodeError),
    #[error("failed to build string from utf8: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
    #[error("failure to verify token: {0}")]
    Verify(String),
}

#[instrument(skip_all)]
pub async fn validate_bearer_token(
    public_key: JwtPublicSigningKey,
    bearer_token: impl AsRef<str>,
) -> JwtKeyResult<JWTClaims<UserClaim>> {
    let bearer_token = bearer_token.as_ref();
    let token = if let Some(token) = bearer_token.strip_prefix("Bearer ") {
        token.to_string()
    } else {
        return Err(JwtKeyError::BearerToken);
    };

    let claims = tokio::task::spawn_blocking(move || {
        public_key
            .inner
            .verify_token::<UserClaim>(&token, None)
            .map_err(|err| JwtKeyError::Verify(format!("{err}")))
    })
    .instrument(trace_span!(
        "verfy_token",
        code.namespace = "jwt_simple::algorithms::RSAPublicKeyLike"
    ))
    .await??;
    Ok(claims)
}
