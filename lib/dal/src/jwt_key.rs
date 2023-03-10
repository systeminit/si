use std::{fs::File, io::prelude::*, path::Path, pin::Pin, str::FromStr};

use jwt_simple::{
    algorithms::{RS256KeyPair, RS256PublicKey},
    prelude::{JWTClaims, RSAPublicKeyLike, Token},
};
use serde::{Deserialize, Serialize};
use si_data_pg::{InstrumentedTransaction, PgError, PgPoolError};
use sodiumoxide::crypto::secretbox;
use telemetry::prelude::*;
use thiserror::Error;
use tokio::{
    fs,
    io::{AsyncRead, AsyncReadExt, AsyncWriteExt},
    task::JoinError,
};

use crate::{pk, DalContext, UserClaim, UserPk, WorkspacePk};

const JWT_KEY_EXISTS: &str = include_str!("queries/jwt_key/exists.sql");
const JWT_KEY_GET_LATEST_PRIVATE_KEY: &str =
    include_str!("queries/jwt_key/get_latest_private_key.sql");
const JWT_KEY_GET_PUBLIC_KEY: &str = include_str!("queries/jwt_key/get_public_key.sql");

#[derive(Error, Debug)]
pub enum JwtKeyError {
    #[error(transparent)]
    UlidDecode(#[from] ulid::DecodeError),
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
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("pg pool error: {0}")]
    PgPool(#[from] PgPoolError),
    #[error("{0}")]
    TaskJoin(#[from] JoinError),
    #[error("failed to convert into PEM format")]
    ToPem,
    #[error("failed to build string from utf8: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
    #[error("failure to verify token: {0}")]
    Verify(String),
}

pub type JwtKeyResult<T> = Result<T, JwtKeyError>;

pk!(ApiClientPk);

#[derive(Deserialize, Serialize, Debug, Clone, Eq, PartialEq)]
pub struct SiClaims {
    pub user_pk: UserPk,
    pub workspace_pk: WorkspacePk,
}

#[derive(Deserialize, Serialize, Debug, Clone, Eq, PartialEq)]
pub struct ApiClaim {
    pub api_client_pk: ApiClientPk,
    pub workspace_pk: WorkspacePk,
}

pk!(JwtPk);

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct JwtSecretKey {
    pub key: secretbox::Key,
}

impl Default for JwtSecretKey {
    #[cfg(debug_assertions)]
    fn default() -> Self {
        let raw_key = [
            107, 104, 252, 148, 123, 127, 84, 116, 235, 167, 44, 161, 120, 187, 34, 124, 185, 25,
            1, 208, 13, 231, 205, 65, 159, 177, 187, 37, 34, 11, 113, 145,
        ];
        let key = sodiumoxide::crypto::secretbox::Key::from_slice(&raw_key)
            .expect("embedded key is invalid");
        Self { key }
    }

    #[cfg(not(debug_assertions))]
    fn default() -> Self {
        Self {
            key: sodiumoxide::crypto::secretbox::gen_key(),
        }
    }
}

impl JwtSecretKey {
    pub async fn create(path: impl AsRef<Path>) -> JwtKeyResult<Self> {
        let mut file = fs::File::create(path).await?;
        let key = secretbox::gen_key();
        file.write_all(&key.0).await?;

        Ok(Self { key })
    }

    pub async fn load(path: impl AsRef<Path>) -> JwtKeyResult<Self> {
        trace!(
            path = path.as_ref().to_string_lossy().as_ref(),
            "loading jwt secret key"
        );
        let mut file = fs::File::open(path).await?;
        Self::from_reader(Pin::new(&mut file)).await
    }

    pub async fn from_reader(mut reader: Pin<&mut impl AsyncRead>) -> JwtKeyResult<Self> {
        let mut buf: Vec<u8> = Vec::with_capacity(secretbox::KEYBYTES);
        reader.read_to_end(&mut buf).await?;
        let key = secretbox::Key::from_slice(&buf).ok_or(JwtKeyError::JwtEncryptionKeyParse)?;

        Ok(Self { key })
    }
}

#[instrument(skip_all)]
pub async fn get_jwt_validation_key(
    ctx: &DalContext,
    jwt_id: impl AsRef<str>,
) -> JwtKeyResult<RS256PublicKey> {
    // let jwt_id = jwt_id.as_ref();
    // let pk = JwtPk::from_str(jwt_id)?;

    // let row = ctx
    //     .txns()
    //     .pg()
    //     .query_one(JWT_KEY_GET_PUBLIC_KEY, &[&pk])
    //     .await?;
    // let key: String = row.try_get("public_key")?;

    // TODO: this is the dev public key, we'll need to toggle to the prod public key
    // and we want to load this in a much smarter way...
    let key = "-----BEGIN PUBLIC KEY-----
MIICIjANBgkqhkiG9w0BAQEFAAOCAg8AMIICCgKCAgEA26alH+FYeuFfrLedINll
EOCxMwDE8Y/cp3dMgoCBxB6pE1wn7uPkUispjjcsYFGKsmJ6pmrirQ6k65A3teeg
QqBCVDoWkco6GFvdp9lhySFXoZ9SEo0DQWvqe/o+YzHRToq+KTrEFoY+SWJYGiJS
yuwrm09YelG5QRA3E6ajGbbRNzd1XpvvSm0gI0OHUL8v6ZnFZeWVIDKqne/BXy/C
NYTZEGKZr5hroxBqqpze3CqhCAAN9rfTtQxvKNOzd0lgy4Cu8X+RBKm+unKgDPhY
SqA6wKcu4T5asMd4NdZ1r5g1onhQNm5ouxtKq35Mlh+hbRgw9/QMYEYMKggDYCvn
AorwPyCXjGtgCBT0KVsaZBTBRf5uZzWV6D5mjcMHjJoFpC41VOceio3/NCGTqu1M
j+TdmI+toprQqAU/OG0eXlDS7HNxyqKhbwDnBnI8gedQ0rhHHkyK0wnX+4H04c43
5UyHxdbqJbcdSbssUDqYmGk0vcN6u72/YrQwT0GfVYBCBGQn+cpN8P3nT+k533nb
w6zMZwg3ztrMZO1cV/xpiDUTxxV5iWN/HoiSSZ1PCK9Byc/NnLIeqL8vO2RHa0J/
OZk+wfML7+4H53lowRr0zAmkMn2u1Wxda9oGSUezsIvyIDWnOruM/DtIOEQnkIEg
08nljy29cVMh5/26Oga3qysCAwEAAQ==
-----END PUBLIC KEY-----";

    tokio::task::spawn_blocking(move || {
        RS256PublicKey::from_pem(&key).map_err(|err| JwtKeyError::KeyFromPem(format!("{err}")))
    })
    .instrument(trace_span!(
        "from_pem",
        code.namespace = "jwt_simple::algorithms::RS256PublicKey"
    ))
    .await?
}

#[instrument(skip_all)]
pub async fn validate_bearer_token(
    ctx: &DalContext,
    bearer_token: impl AsRef<str>,
) -> JwtKeyResult<JWTClaims<UserClaim>> {
    let bearer_token = bearer_token.as_ref();
    let token = if let Some(token) = bearer_token.strip_prefix("Bearer ") {
        token.to_string()
    } else {
        return Err(JwtKeyError::BearerToken);
    };

    let metadata =
        Token::decode_metadata(&token).map_err(|err| JwtKeyError::Metadata(format!("{err}")))?;
    let key_id = metadata
        .key_id()
        .ok_or_else(|| JwtKeyError::Metadata("missing key id".into()))?;
    let public_key = get_jwt_validation_key(ctx, key_id).await?;
    let claims = tokio::task::spawn_blocking(move || {
        public_key
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

#[instrument(skip_all)]
pub async fn validate_bearer_token_api_client(
    ctx: &DalContext,
    bearer_token: impl AsRef<str>,
) -> JwtKeyResult<JWTClaims<ApiClaim>> {
    let bearer_token = bearer_token.as_ref();
    let token = if let Some(token) = bearer_token.strip_prefix("Bearer ") {
        token
    } else {
        return Err(JwtKeyError::BearerToken);
    };

    let metadata =
        Token::decode_metadata(token).map_err(|err| JwtKeyError::Metadata(format!("{err}")))?;
    let key_id = metadata
        .key_id()
        .ok_or_else(|| JwtKeyError::Metadata("missing key id".into()))?;

    let public_key = get_jwt_validation_key(ctx, key_id).await?;
    let claims = public_key
        .verify_token::<ApiClaim>(token, None)
        .map_err(|err| JwtKeyError::Verify(format!("{err}")))?;
    Ok(claims)
}

#[instrument(skip_all)]
pub async fn get_jwt_signing_key(
    ctx: &DalContext,
    jwt_secret_key: &JwtSecretKey,
) -> JwtKeyResult<RS256KeyPair> {
    let row = ctx
        .txns()
        .pg()
        .query_one(JWT_KEY_GET_LATEST_PRIVATE_KEY, &[])
        .await?;
    let encrypted_private_key: String = row.try_get("private_key")?;
    let nonce_bytes = row.try_get("nonce")?;
    let pk: JwtPk = row.try_get("pk")?;
    let nonce = secretbox::Nonce::from_slice(nonce_bytes).ok_or(JwtKeyError::BadNonce)?;

    let secret_bytes = base64::decode(encrypted_private_key)?;
    let key_bytes = secretbox::open(&secret_bytes, &nonce, &jwt_secret_key.key)
        .map_err(|()| JwtKeyError::Decrypt)?;
    let key = String::from_utf8(key_bytes)?;
    let key_pair =
        RS256KeyPair::from_pem(&key).map_err(|err| JwtKeyError::KeyFromPem(format!("{err}")))?;
    let key_pair_with_id = key_pair.with_key_id(&format!("{pk}"));
    Ok(key_pair_with_id)
}

#[instrument(skip_all)]
pub async fn jwt_key_exists(txn: &InstrumentedTransaction<'_>) -> JwtKeyResult<bool> {
    let rows = txn.query(JWT_KEY_EXISTS, &[]).await?;
    Ok(!rows.is_empty())
}

#[instrument(skip_all)]
pub async fn install_new_jwt_key(
    txn: &InstrumentedTransaction<'_>,
    jwt_secret_key: &JwtSecretKey,
) -> JwtKeyResult<()> {
    // NOTE(fnichol): It's a little unclear to me what a good "molulus bits" value would be, this
    // seems to correspond to the key length, and generating longer keys, unsurprisingly takes much
    // longer
    info!("generating new RS256 key pair for signing JWT, this may take a while");
    let keypair = tokio::task::spawn_blocking(move || {
        RS256KeyPair::generate(2048).map_err(|err| {
            warn!(error = ?err, "failed to generate keypair");
            JwtKeyError::GenerateKeyPair
        })
    })
    .instrument(info_span!(
        "generate",
        code.namespace = "jwt_simple::algorithms::rsa::RS256KeyPair"
    ))
    .await??;
    debug!("finished generating new RS256 key pair");

    let private_key_pem = keypair.to_pem().map_err(|_| JwtKeyError::ToPem)?;
    let public_key_pem = keypair
        .public_key()
        .to_pem()
        .map_err(|_| JwtKeyError::ToPem)?;

    let nonce = secretbox::gen_nonce();
    let encrypted_key = secretbox::seal(private_key_pem.as_bytes(), &nonce, &jwt_secret_key.key);
    let base64_encrypted_key = base64::encode(encrypted_key);

    let _row = txn
        .query_one(
            "SELECT jwt_key_create_v1($1, $2, $3)",
            &[&public_key_pem, &base64_encrypted_key, &nonce.as_ref()],
        )
        .await?;

    Ok(())
}

#[instrument(skip_all)]
pub async fn create_jwt_key_if_missing(
    txn: &InstrumentedTransaction<'_>,
    public_filename: impl AsRef<str>,
    private_filename: impl AsRef<str>,
    secret_key: &secretbox::Key,
) -> JwtKeyResult<()> {
    if jwt_key_exists(txn).await? {
        return Ok(());
    }

    let public_filename = public_filename.as_ref();
    let private_filename = private_filename.as_ref();

    let mut private_file = File::open(private_filename)?;
    let mut private_key = String::new();
    private_file.read_to_string(&mut private_key)?;
    let nonce = secretbox::gen_nonce();
    let encrypted_key = secretbox::seal(private_key.as_bytes(), &nonce, secret_key);
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
