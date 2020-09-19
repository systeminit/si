use serde::{Deserialize, Serialize};
use sodiumoxide::crypto::pwhash::argon2id13;
use thiserror::Error;

use std::collections::HashMap;

use crate::data::Db;
use crate::models::{
    check_secondary_key, generate_id, get_model, insert_model, ModelError, SiStorableError,
    SimpleStorable,
};

#[derive(Error, Debug)]
pub enum UserError {
    #[error("a user with this email already exists in this billing account")]
    EmailExists,
    #[error("si_storable error: {0}")]
    SiStorable(#[from] SiStorableError),
    #[error("error in core model functions: {0}")]
    Model(#[from] ModelError),
    #[error("invalid uft-8 string: {0}")]
    Utf8(#[from] std::str::Utf8Error),
    #[error("error generating password hash")]
    PasswordHash,
    #[error("user is not found")]
    NotFound,
    #[error("database error: {0}")]
    Data(#[from] si_data::DataError),
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

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: String,
    pub name: String,
    pub email: String,
    pub si_storable: SimpleStorable,
}

impl User {
    pub async fn new(
        db: &Db,
        name: impl Into<String>,
        email: impl Into<String>,
        billing_account_id: impl Into<String>,
        password: impl Into<String>,
    ) -> UserResult<User> {
        let name = name.into();
        let email = email.into();
        let billing_account_id = billing_account_id.into();

        if check_secondary_key(db, &billing_account_id, "user", "name", &name).await? {
            return Err(UserError::EmailExists);
        }

        let id = generate_id("user");
        let si_storable = SimpleStorable::new(&id, "user", &billing_account_id);
        let object = User {
            id: id.clone(),
            name,
            email,
            si_storable,
        };
        insert_model(db, &object.id, &object).await?;

        let _user_password = UserPassword::new(db, id, password, billing_account_id).await?;

        Ok(object)
    }

    pub async fn get_by_email(
        db: &Db,
        email: impl AsRef<str>,
        billing_account_id: impl AsRef<str>,
    ) -> UserResult<User> {
        let email = email.as_ref();
        let billing_account_id = billing_account_id.as_ref();

        let query = format!(
            "SELECT a.*
               FROM `{bucket}` AS a
               WHERE a.siStorable.typeName = \"user\"
                 AND a.siStorable.billingAccountId = $billing_account_id
                 AND a.email = $email
               LIMIT 1",
            bucket = db.bucket_name,
        );
        let mut named_params: HashMap<String, serde_json::Value> = HashMap::new();
        named_params.insert("email".into(), serde_json::json![email]);
        named_params.insert(
            "billing_account_id".into(),
            serde_json::json![billing_account_id],
        );
        let mut results: Vec<User> = db.query(query, Some(named_params)).await?;
        if let Some(user) = results.pop() {
            Ok(user)
        } else {
            Err(UserError::NotFound)
        }
    }

    pub async fn verify(&self, db: &Db, password: impl AsRef<str>) -> UserResult<bool> {
        UserPassword::verify(db, &self.id, &self.si_storable.billing_account_id, password).await
    }

    pub async fn get(
        db: &Db,
        user_id: impl AsRef<str>,
        billing_account_id: impl AsRef<str>,
    ) -> UserResult<User> {
        let id = user_id.as_ref();
        let billing_account_id = billing_account_id.as_ref();
        let object: User = get_model(db, id, billing_account_id).await?;
        Ok(object)
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UserPassword {
    pub id: String,
    pub user_id: String,
    pub password_hash: String,
    pub si_storable: SimpleStorable,
}

impl UserPassword {
    pub async fn new(
        db: &Db,
        user_id: impl Into<String>,
        password: impl Into<String>,
        billing_account_id: impl Into<String>,
    ) -> UserResult<UserPassword> {
        let user_id = user_id.into();
        let password = password.into();
        let billing_account_id = billing_account_id.into();
        let password_hash = encrypt_password(password)?;
        let id = format!("{}:userPassword", &user_id);
        let si_storable = SimpleStorable::new(&id, "userPassword", billing_account_id);
        let object = UserPassword {
            id,
            user_id,
            password_hash,
            si_storable,
        };
        insert_model(db, &object.id, &object).await?;

        Ok(object)
    }

    pub async fn get(
        db: &Db,
        user_id: impl AsRef<str>,
        billing_account_id: impl AsRef<str>,
    ) -> UserResult<UserPassword> {
        let id = format!("{}:userPassword", user_id.as_ref());
        let billing_account_id = billing_account_id.as_ref();
        let object: UserPassword = get_model(db, id, billing_account_id).await?;
        Ok(object)
    }

    pub async fn verify(
        db: &Db,
        user_id: impl AsRef<str>,
        billing_account_id: impl AsRef<str>,
        password: impl AsRef<str>,
    ) -> UserResult<bool> {
        let user_id = user_id.as_ref();
        let billing_account_id = billing_account_id.as_ref();
        let password = password.as_ref();
        let up = UserPassword::get(db, user_id, billing_account_id).await?;
        Ok(verify_password(password, up.password_hash))
    }
}

pub fn encrypt_password(password: impl Into<String>) -> UserResult<String> {
    let password = password.into();
    let password_hash = argon2id13::pwhash(
        password.as_bytes(),
        argon2id13::OPSLIMIT_INTERACTIVE,
        argon2id13::MEMLIMIT_INTERACTIVE,
    )
    .map_err(|()| UserError::PasswordHash)?;
    let password_hash_str = std::str::from_utf8(password_hash.as_ref())?;
    Ok(password_hash_str.to_string())
}

pub fn verify_password(password: &str, password_hash: impl AsRef<str>) -> bool {
    let password_hash = password_hash.as_ref();
    let password_bytes = password.as_bytes();
    if let Some(argon_password) = argon2id13::HashedPassword::from_slice(password_hash.as_bytes()) {
        if argon2id13::pwhash_verify(&argon_password, password_bytes) {
            true
        } else {
            false
        }
    } else {
        false
    }
}
