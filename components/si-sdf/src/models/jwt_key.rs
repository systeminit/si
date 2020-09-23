use jwt_simple::algorithms::{RS256KeyPair, RS256PublicKey};
use jwt_simple::prelude::JWTClaims;
use jwt_simple::prelude::RSAPublicKeyLike;
use jwt_simple::prelude::Token;
//use jwt_simple::prelude::*;
use serde::{Deserialize, Serialize};
use sodiumoxide::crypto::secretbox;
use thiserror::Error;

use std::fs::File;
use std::io::prelude::*;

use crate::data::{Connection, Db};
use crate::handlers::users::SiClaims;
use crate::models::{
    generate_id, insert_model, MinimalStorable, ModelError, UpdateClock, UpdateClockError,
};

#[derive(Error, Debug)]
pub enum JwtKeyError {
    #[error("database error: {0}")]
    Data(#[from] crate::data::DataError),
    #[error("error in core model functions: {0}")]
    Model(#[from] ModelError),
    #[error("failed to decrypt secret data")]
    Decrypt,
    #[error("failed to build string from utf8: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
    #[error("failed to decode base64 string: {0}")]
    Base64Decode(#[from] base64::DecodeError),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("update clock error: {0}")]
    UpdateClock(#[from] UpdateClockError),
    #[error("no signing keys - bad news for you!")]
    NoKeys,
    #[error("failure to build signing key from pem: {0}")]
    KeyFromPem(String),
    #[error("failure to extract metadata from bearer token: {0}")]
    Metadata(String),
    #[error("failure to verify token: {0}")]
    Verify(String),
    #[error("invalid bearer token")]
    BearerToken,
}

pub type JwtKeyResult<T> = Result<T, JwtKeyError>;

#[derive(Deserialize, Serialize, Debug)]
pub enum JwtKeyType {
    RS256,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct JwtKeyPublic {
    pub id: String,
    pub key_type: JwtKeyType,
    pub key: String,
    pub si_storable: MinimalStorable,
    pub update_clock: UpdateClock,
}

impl JwtKeyPublic {
    pub async fn new(
        db: &Db,
        nats: &Connection,
        id: impl AsRef<str>,
        key_type: JwtKeyType,
        key: impl Into<String>,
    ) -> JwtKeyResult<JwtKeyPublic> {
        let key = key.into();
        let id = id.as_ref();
        let object_id = format!("{}:jwtKeyPublic", id);
        let si_storable = MinimalStorable::new(&object_id, "jwtKeyPublic");
        let update_clock = UpdateClock::create_or_update(db, "jwtKey", 0).await?;
        let object = JwtKeyPublic {
            id: object_id,
            key_type,
            key,
            si_storable,
            update_clock,
        };
        insert_model(db, nats, &object.id, &object).await?;
        Ok(object)
    }

    pub async fn get_jwt_validation_key(
        db: &Db,
        id: impl AsRef<str>,
    ) -> JwtKeyResult<RS256PublicKey> {
        let id = id.as_ref();
        let jwt_key_public: JwtKeyPublic = db.get(format!("{}:jwtKeyPublic", id)).await?;
        RS256PublicKey::from_pem(&jwt_key_public.key)
            .map_err(|err| JwtKeyError::KeyFromPem(format!("{}", err)))
    }

    pub async fn validate_bearer_token(
        db: &Db,
        bearer_token: impl AsRef<str>,
    ) -> JwtKeyResult<JWTClaims<SiClaims>> {
        let bearer_token = bearer_token.as_ref();
        let token = if let Some(token) = bearer_token.strip_prefix("Bearer ") {
            token
        } else {
            return Err(JwtKeyError::BearerToken);
        };

        let metadata = Token::decode_metadata(token)
            .map_err(|err| JwtKeyError::Metadata(format!("{}", err)))?;
        let key_id = metadata
            .key_id()
            .ok_or(JwtKeyError::Metadata("missing key id".into()))?;
        let public_key = JwtKeyPublic::get_jwt_validation_key(&db, key_id).await?;
        let claims = public_key
            .verify_token::<SiClaims>(&token, None)
            .map_err(|err| JwtKeyError::Verify(format!("{}", err)))?;
        Ok(claims)
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct JwtKeyPrivate {
    pub id: String,
    pub key_type: JwtKeyType,
    pub nonce: secretbox::Nonce,
    pub key: String,
    pub si_storable: MinimalStorable,
    pub update_clock: UpdateClock,
}

impl JwtKeyPrivate {
    pub async fn new(
        db: &Db,
        nats: &Connection,
        id: impl AsRef<str>,
        key_type: JwtKeyType,
        key: impl Into<String>,
        secret_key: &secretbox::Key,
    ) -> JwtKeyResult<JwtKeyPrivate> {
        let key = key.into();
        let id = id.as_ref();
        let nonce = secretbox::gen_nonce();

        let encrypted_key = secretbox::seal(key.as_bytes(), &nonce, &secret_key);
        let base64_encrypted_key = base64::encode(encrypted_key);

        let object_id = format!("{}", id);
        let si_storable = MinimalStorable::new(&object_id, "jwtKeyPrivate");
        let update_clock = UpdateClock::create_or_update(db, "jwtKey", 0).await?;
        let object = JwtKeyPrivate {
            id: object_id,
            nonce,
            key_type,
            key: base64_encrypted_key,
            si_storable,
            update_clock,
        };
        insert_model(db, nats, &object.id, &object).await?;
        Ok(object)
    }

    pub fn decrypt(&mut self, secret_key: &secretbox::Key) -> JwtKeyResult<()> {
        let secret_bytes = base64::decode(&self.key)?;
        let key = secretbox::open(&secret_bytes, &self.nonce, secret_key)
            .map_err(|()| JwtKeyError::Decrypt)?;
        self.key = String::from_utf8(key)?;
        Ok(())
    }

    pub fn as_signing_key(&self) -> JwtKeyResult<RS256KeyPair> {
        let key_pair = RS256KeyPair::from_pem(&self.key)
            .map_err(|err| JwtKeyError::KeyFromPem(format!("{}", err)))?;
        let key_pair_with_id = key_pair.with_key_id(&self.id);
        Ok(key_pair_with_id)
    }

    pub async fn get_jwt_signing_key(
        db: &Db,
        secret_key: &secretbox::Key,
    ) -> JwtKeyResult<RS256KeyPair> {
        let query = format!(
            "SELECT a.*
               FROM `{bucket}` AS a
               WHERE a.siStorable.typeName = \"jwtKeyPrivate\"
               ORDER BY a.updateClock.epoch DESC, a.updateClock.update_count DESC
               LIMIT 1",
            bucket = db.bucket_name,
        );
        let mut results: Vec<JwtKeyPrivate> = db.query(query, None).await?;
        if let Some(mut jwt_key_private) = results.pop() {
            jwt_key_private.decrypt(secret_key)?;
            Ok(jwt_key_private.as_signing_key()?)
        } else {
            Err(JwtKeyError::NoKeys)
        }
    }
}

pub async fn create_if_missing(
    db: &Db,
    nats: &Connection,
    public_filename: impl AsRef<str>,
    private_filename: impl AsRef<str>,
    secret_key: &secretbox::Key,
) -> JwtKeyResult<()> {
    let query = format!(
        "SELECT a.*
               FROM `{bucket}` AS a
               WHERE a.siStorable.typeName = \"jwtKeyPrivate\"
               LIMIT 1",
        bucket = db.bucket_name,
    );
    let results: Vec<JwtKeyPrivate> = db.query(query, None).await?;
    if results.len() != 0 {
        return Ok(());
    }

    let id = generate_id("jwtKey");

    let public_filename = public_filename.as_ref();
    let private_filename = private_filename.as_ref();

    let mut private_file = File::open(private_filename)?;
    let mut private_key = String::new();
    private_file.read_to_string(&mut private_key)?;
    let _jwt_key_private =
        JwtKeyPrivate::new(db, nats, &id, JwtKeyType::RS256, private_key, secret_key).await?;

    let mut public_file = File::open(public_filename)?;
    let mut public_key = String::new();
    public_file.read_to_string(&mut public_key)?;
    let _jwt_key_public = JwtKeyPublic::new(db, nats, &id, JwtKeyType::RS256, public_key).await?;

    Ok(())
}
