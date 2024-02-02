use si_std::CanonicalFile;
use std::{io::Cursor, path::Path, pin::Pin, sync::Arc};

use base64::{engine::general_purpose, Engine};
use jwt_simple::{
    algorithms::RS256PublicKey,
    prelude::{JWTClaims, RSAPublicKeyLike},
};
use serde::{Deserialize, Serialize};
use si_data_pg::{PgError, PgPoolError};
use telemetry::prelude::*;
use thiserror::Error;
use tokio::{
    fs,
    io::{AsyncRead, AsyncReadExt},
    task::JoinError,
};

use crate::{TransactionsError, UserClaim, UserPk, WorkspacePk};

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
    #[error("error creating jwt from config")]
    FromConfig,
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
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("pg pool error: {0}")]
    PgPool(#[from] PgPoolError),
    #[error("{0}")]
    TaskJoin(#[from] JoinError),
    #[error("failed to convert into PEM format")]
    ToPem,
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error(transparent)]
    UlidDecode(#[from] ulid::DecodeError),
    #[error("failed to build string from utf8: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
    #[error("failure to verify token: {0}")]
    Verify(String),
}

pub type JwtKeyResult<T> = Result<T, JwtKeyError>;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct JwtConfig {
    pub key_file: Option<CanonicalFile>,
    pub key_base64: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone, Eq, PartialEq)]
pub struct SiClaims {
    pub user_pk: UserPk,
    pub workspace_pk: WorkspacePk,
}

#[derive(Clone, Debug)]
pub struct JwtPublicSigningKey {
    inner: Arc<RS256PublicKey>,
}

impl JwtPublicSigningKey {
    pub async fn from_config(config: JwtConfig) -> JwtKeyResult<Self> {
        match (config.key_file, config.key_base64) {
            (Some(path), None) => Self::load(path).await,
            (None, Some(b64_string)) => Self::decode(b64_string).await,
            _ => Err(JwtKeyError::FromConfig),
        }
    }

    #[instrument(level = "debug", skip_all)]
    pub async fn load(path: impl AsRef<Path>) -> JwtKeyResult<Self> {
        trace!(
            path = path.as_ref().to_string_lossy().as_ref(),
            "loading jwt public signing key"
        );
        let mut file = fs::File::open(path).await?;
        Self::from_reader(Pin::new(&mut file)).await
    }

    #[instrument(level = "debug", skip_all)]
    pub async fn decode(key_string: String) -> JwtKeyResult<Self> {
        let buf = general_purpose::STANDARD.decode(key_string)?;

        Self::from_reader(Pin::new(&mut Cursor::new(&buf))).await
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

#[instrument(level = "debug", skip_all)]
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
