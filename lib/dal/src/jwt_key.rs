use crate::{pk, UserClaim};
use jwt_simple::{
    algorithms::{RS256KeyPair, RS256PublicKey},
    prelude::{JWTClaims, RSAPublicKeyLike, Token},
};
use serde::{Deserialize, Serialize};
use si_data::PgTxn;
use sodiumoxide::crypto::secretbox;
use std::{fs::File, io::prelude::*};
use thiserror::Error;
use tokio::task::JoinError;
use tracing::{info_span, instrument, Instrument};

const JWT_KEY_EXISTS: &str = include_str!("./queries/jwt_key_exists.sql");
const JWT_KEY_GET_LATEST_PRIVATE_KEY: &str =
    include_str!("./queries/jwt_key_get_latest_private_key.sql");
const JWT_KEY_GET_PUBLIC_KEY: &str = include_str!("./queries/jwt_key_get_public_key.sql");

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
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("failure to build signing key from pem: {0}")]
    KeyFromPem(String),
    #[error("failure to extract metadata from bearer token: {0}")]
    Metadata(String),
    #[error("no signing keys - bad news for you!")]
    NoKeys,
    #[error("bad string version of numeric id: {0}")]
    ParseInt(#[from] std::num::ParseIntError),
    #[error("pg error: {0}")]
    Pg(#[from] si_data::PgError),
    #[error("pg pool error: {0}")]
    PgPool(#[from] si_data::PgPoolError),
    #[error("{0}")]
    TaskJoin(#[from] JoinError),
    #[error("failed to build string from utf8: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
    #[error("failure to verify token: {0}")]
    Verify(String),
}

pub type JwtKeyResult<T> = Result<T, JwtKeyError>;

#[derive(Deserialize, Serialize, Debug, Clone, Eq, PartialEq)]
pub struct SiClaims {
    pub user_pk: i64,
    pub billing_account_pk: i64,
}

#[derive(Deserialize, Serialize, Debug, Clone, Eq, PartialEq)]
pub struct ApiClaim {
    pub api_client_pk: i64,
    pub billing_account_pk: i64,
}

pk!(JwtPk);

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct JwtEncrypt {
    pub key: secretbox::Key,
}

impl Default for JwtEncrypt {
    #[cfg(debug_assertions)]
    fn default() -> Self {
        let raw_key = [
            107, 104, 252, 148, 123, 127, 84, 116, 235, 167, 44, 161, 120, 187, 34, 124, 185, 25,
            1, 208, 13, 231, 205, 65, 159, 177, 187, 37, 34, 11, 113, 145,
        ];
        let key = sodiumoxide::crypto::secretbox::Key::from_slice(&raw_key)
            .expect("embedded key is invalid");
        JwtEncrypt { key }
    }

    #[cfg(not(debug_assertions))]
    fn default() -> Self {
        JwtEncrypt {
            key: sodiumoxide::crypto::secretbox::gen_key(),
        }
    }
}

#[tracing::instrument(skip(txn, jwt_id))]
pub async fn get_jwt_validation_key(
    txn: &PgTxn<'_>,
    jwt_id: impl AsRef<str>,
) -> JwtKeyResult<RS256PublicKey> {
    let jwt_id = jwt_id.as_ref();
    let pk: JwtPk = jwt_id.parse::<i64>()?.into();

    let row = txn.query_one(JWT_KEY_GET_PUBLIC_KEY, &[&pk]).await?;
    let key: String = row.try_get("public_key")?;

    tokio::task::spawn_blocking(move || {
        RS256PublicKey::from_pem(&key).map_err(|err| JwtKeyError::KeyFromPem(format!("{}", err)))
    })
    .instrument(info_span!(
        "from_pem",
        code.namespace = "jwt_simple::algorithms::RS256PublicKey"
    ))
    .await?
}

#[tracing::instrument(skip(txn, bearer_token))]
pub async fn validate_bearer_token(
    txn: &PgTxn<'_>,
    bearer_token: impl AsRef<str>,
) -> JwtKeyResult<JWTClaims<UserClaim>> {
    let bearer_token = bearer_token.as_ref();
    let token = if let Some(token) = bearer_token.strip_prefix("Bearer ") {
        token.to_string()
    } else {
        return Err(JwtKeyError::BearerToken);
    };

    let metadata =
        Token::decode_metadata(&token).map_err(|err| JwtKeyError::Metadata(format!("{}", err)))?;
    let key_id = metadata
        .key_id()
        .ok_or(JwtKeyError::Metadata("missing key id".into()))?;
    let public_key = get_jwt_validation_key(txn, key_id).await?;
    let claims = tokio::task::spawn_blocking(move || {
        public_key
            .verify_token::<UserClaim>(&token, None)
            .map_err(|err| JwtKeyError::Verify(format!("{}", err)))
    })
    .instrument(info_span!(
        "verify_token",
        code.namespace = "jwt_simple::algorithms::RSAPublicKeyLike"
    ))
    .await??;
    Ok(claims)
}

#[tracing::instrument(skip(txn, bearer_token))]
pub async fn validate_bearer_token_api_client(
    txn: &PgTxn<'_>,
    bearer_token: impl AsRef<str>,
) -> JwtKeyResult<JWTClaims<ApiClaim>> {
    let bearer_token = bearer_token.as_ref();
    let token = if let Some(token) = bearer_token.strip_prefix("Bearer ") {
        token
    } else {
        return Err(JwtKeyError::BearerToken);
    };

    let metadata =
        Token::decode_metadata(token).map_err(|err| JwtKeyError::Metadata(format!("{}", err)))?;
    let key_id = metadata
        .key_id()
        .ok_or(JwtKeyError::Metadata("missing key id".into()))?;

    let public_key = get_jwt_validation_key(txn, key_id).await?;
    let claims = public_key
        .verify_token::<ApiClaim>(&token, None)
        .map_err(|err| JwtKeyError::Verify(format!("{}", err)))?;
    Ok(claims)
}

#[tracing::instrument(skip(txn, secret_key))]
pub async fn get_jwt_signing_key(
    txn: &PgTxn<'_>,
    secret_key: &secretbox::Key,
) -> JwtKeyResult<RS256KeyPair> {
    let row = txn.query_one(JWT_KEY_GET_LATEST_PRIVATE_KEY, &[]).await?;
    let encrypted_private_key: String = row.try_get("private_key")?;
    let nonce_bytes = row.try_get("nonce")?;
    let pk: JwtPk = row.try_get("pk")?;
    let nonce = secretbox::Nonce::from_slice(nonce_bytes).ok_or(JwtKeyError::BadNonce)?;

    let secret_bytes = base64::decode(&encrypted_private_key)?;
    let key_bytes =
        secretbox::open(&secret_bytes, &nonce, secret_key).map_err(|()| JwtKeyError::Decrypt)?;
    let key = String::from_utf8(key_bytes)?;
    let key_pair =
        RS256KeyPair::from_pem(&key).map_err(|err| JwtKeyError::KeyFromPem(format!("{}", err)))?;
    let key_pair_with_id = key_pair.with_key_id(&format!("{}", pk));
    Ok(key_pair_with_id)
}

#[instrument(skip(txn, public_filename, private_filename, secret_key))]
pub async fn create_jwt_key_if_missing(
    txn: &PgTxn<'_>,
    public_filename: impl AsRef<str>,
    private_filename: impl AsRef<str>,
    secret_key: &secretbox::Key,
) -> JwtKeyResult<()> {
    let rows = txn.query(JWT_KEY_EXISTS, &[]).await?;
    if rows.len() > 0 {
        return Ok(());
    }

    let public_filename = public_filename.as_ref();
    let private_filename = private_filename.as_ref();

    let mut private_file = File::open(private_filename)?;
    let mut private_key = String::new();
    private_file.read_to_string(&mut private_key)?;
    let nonce = secretbox::gen_nonce();
    let encrypted_key = secretbox::seal(private_key.as_bytes(), &nonce, &secret_key);
    let base64_encrypted_key = base64::encode(encrypted_key);

    let mut public_file = File::open(public_filename)?;
    let mut public_key = String::new();
    public_file.read_to_string(&mut public_key)?;

    let _row = txn
        .query_one(
            "SELECT jwt_key_create_v1($1, $2, $3)",
            &[&public_key, &base64_encrypted_key, &nonce.as_ref()],
        )
        .await?;

    Ok(())
}
