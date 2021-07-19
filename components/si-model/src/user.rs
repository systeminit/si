use crate::{JwtKeyError, SimpleStorable};
use serde::{Deserialize, Serialize};
use si_data::{NatsTxn, NatsTxnError, PgTxn};
use sodiumoxide::crypto::pwhash::argon2id13;
use thiserror::Error;
use tracing::warn;

const AUTHORIZE_USER: &str = include_str!("./queries/authorize_user.sql");
const USER_GET_BY_EMAIL: &str = include_str!("./queries/user_get_by_email.sql");
const USER_VERIFY: &str = include_str!("./queries/user_verify.sql");

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SiClaims {
    pub user_id: String,
    pub billing_account_id: String,
}

#[derive(Error, Debug)]
pub enum UserError {
    #[error("a user with this email already exists in this billing account")]
    EmailExists,
    #[error("jwt key error: {0}")]
    JwtKey(#[from] JwtKeyError),
    #[error("nats txn error: {0}")]
    NatsTxn(#[from] NatsTxnError),
    #[error("user is not found")]
    NotFound,
    #[error("error generating password hash")]
    PasswordHash,
    #[error("pg error: {0}")]
    Pg(#[from] si_data::PgError),
    #[error("serde error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("user is unauthorized")]
    Unauthorized,
    #[error("invalid uft-8 string: {0}")]
    Utf8(#[from] std::str::Utf8Error),
}

pub type UserResult<T> = Result<T, UserError>;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LoginRequest {
    pub billing_account_name: String,
    pub email: String,
    pub password: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LoginReply {
    pub user: User,
    pub jwt: String,
}

#[derive(Deserialize, Serialize, Debug, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: String,
    pub name: String,
    pub email: String,
    pub si_storable: SimpleStorable,
}

impl User {
    pub async fn new(
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        name: impl Into<String>,
        email: impl Into<String>,
        password: impl Into<String>,
        billing_account_id: impl Into<String>,
    ) -> UserResult<User> {
        let name = name.into();
        let email = email.into();
        let password_hash = encrypt_password(password)?;
        let billing_account_id = billing_account_id.into();

        let row = txn
            .query_one(
                "SELECT object FROM user_create_v1($1, $2, $3, $4)",
                &[&name, &email, &password_hash.as_ref(), &billing_account_id],
            )
            .await?;
        let json: serde_json::Value = row.try_get("object")?;
        nats.publish(&json).await?;
        let object: User = serde_json::from_value(json)?;

        Ok(object)
    }

    pub async fn get_by_email(
        txn: &PgTxn<'_>,
        email: impl AsRef<str>,
        billing_account_id: impl AsRef<str>,
    ) -> UserResult<User> {
        let email = email.as_ref();
        let billing_account_id = billing_account_id.as_ref();

        let row = txn
            .query_one(USER_GET_BY_EMAIL, &[&email, &billing_account_id])
            .await?;
        let json: serde_json::Value = row.try_get("object")?;
        let user: User = serde_json::from_value(json)?;
        Ok(user)
    }

    pub async fn verify(&self, txn: &PgTxn<'_>, password: impl AsRef<str>) -> UserResult<bool> {
        let password = password.as_ref();
        let row = txn.query_one(USER_VERIFY, &[&self.id]).await?;
        let current_password: Vec<u8> = row.try_get("password")?;
        let result = verify_password(password, &current_password);
        Ok(result)
    }

    pub async fn get(txn: &PgTxn<'_>, user_id: impl AsRef<str>) -> UserResult<User> {
        let id = user_id.as_ref();
        let row = txn
            .query_one("SELECT object FROM user_get_v1($1)", &[&id])
            .await?;
        let json: serde_json::Value = row.try_get("object")?;
        let object = serde_json::from_value(json)?;
        Ok(object)
    }

    pub async fn save(&self, txn: &PgTxn<'_>, nats: &NatsTxn) -> UserResult<User> {
        let json = serde_json::to_value(self)?;
        let row = txn
            .query_one("SELECT object FROM user_save_v1($1)", &[&json])
            .await?;
        let updated_result: serde_json::Value = row.try_get("object")?;
        nats.publish(&updated_result).await?;
        let updated = serde_json::from_value(updated_result)?;
        Ok(updated)
    }
}

pub fn encrypt_password(password: impl Into<String>) -> UserResult<argon2id13::HashedPassword> {
    let password = password.into();
    let password_hash = argon2id13::pwhash(
        password.as_bytes(),
        argon2id13::OPSLIMIT_INTERACTIVE,
        argon2id13::MEMLIMIT_INTERACTIVE,
    )
    .map_err(|()| UserError::PasswordHash)?;
    //let password_hash_str = std::str::from_utf8(password_hash.as_ref())?;
    Ok(password_hash)
}

pub fn verify_password(password: &str, password_hash: &[u8]) -> bool {
    let password_hash = password_hash.as_ref();
    let password_bytes = password.as_bytes();
    if let Some(argon_password) = argon2id13::HashedPassword::from_slice(password_hash) {
        if argon2id13::pwhash_verify(&argon_password, password_bytes) {
            true
        } else {
            false
        }
    } else {
        false
    }
}

#[tracing::instrument(skip(txn, token))]
pub async fn authenticate(txn: &PgTxn<'_>, token: impl AsRef<str>) -> UserResult<SiClaims> {
    let token = token.as_ref();
    let claims = crate::jwt_key::validate_bearer_token(&txn, token).await?;
    Ok(claims.custom)
}

#[tracing::instrument(
    skip(txn, user_id, subject, action),
    fields(
        enduser.id = user_id.as_ref(),
        authn.subject = subject.as_ref(),
        authn.action = action.as_ref(),
    )
)]
pub async fn authorize(
    txn: &PgTxn<'_>,
    user_id: impl AsRef<str>,
    subject: impl AsRef<str>,
    action: impl AsRef<str>,
) -> UserResult<()> {
    txn.query_opt(
        AUTHORIZE_USER,
        &[&user_id.as_ref(), &subject.as_ref(), &action.as_ref()],
    )
    .await?
    .ok_or_else(|| {
        warn!("user is not authorized");
        UserError::Unauthorized
    })
    .map(|_| ())
}
