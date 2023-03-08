use jwt_simple::{algorithms::RSAKeyPairLike, claims::Claims, reexports::coarsetime::Duration};
use serde::{Deserialize, Serialize};
use si_data_nats::NatsError;
use si_data_pg::PgError;
use sodiumoxide::crypto::pwhash::argon2id13;
use telemetry::prelude::*;
use thiserror::Error;
use tokio::task::JoinError;

use crate::jwt_key::{get_jwt_signing_key, JwtKeyError};
use crate::{
    pk, standard_model_accessor_ro, BillingAccountPk, DalContext, HistoryEvent, HistoryEventError,
    JwtSecretKey, Timestamp, TransactionsError, WorkspacePk,
};

const USER_PASSWORD: &str = include_str!("queries/user/password.sql");
const USER_FIND_BY_EMAIL: &str = include_str!("queries/user/find_by_email.sql");
const USER_GET_BY_PK: &str = include_str!("queries/user/get_by_pk.sql");

#[derive(Error, Debug)]
pub enum UserError {
    #[error("error serializing/deserializing json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("nats txn error: {0}")]
    Nats(#[from] NatsError),
    #[error("history event error: {0}")]
    HistoryEvent(#[from] HistoryEventError),
    #[error("password hashing error; bug!")]
    PasswordHash,
    #[error("failed to join long lived async task; bug!")]
    Join(#[from] JoinError),
    #[error("failed to validate the users password")]
    MismatchedPassword,
    #[error("jwt key: {0}")]
    JwtKey(#[from] JwtKeyError),
    #[error("jwt: {0}")]
    JwtSimple(#[from] jwt_simple::Error),
    #[error(transparent)]
    Transactions(#[from] TransactionsError),
}

pub type UserResult<T> = Result<T, UserError>;

pk!(UserPk);

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct User {
    pk: UserPk,
    name: String,
    email: String,
    #[serde(flatten)]
    timestamp: Timestamp,
}

impl User {
    pub fn pk(&self) -> UserPk {
        self.pk
    }

    standard_model_accessor_ro!(name, String);
    standard_model_accessor_ro!(email, String);

    #[allow(clippy::too_many_arguments)]
    #[instrument(skip_all)]
    pub async fn new(
        ctx: &DalContext,
        name: impl AsRef<str>,
        email: impl AsRef<str>,
        password: impl AsRef<str>,
    ) -> UserResult<Self> {
        let name = name.as_ref();
        let email = email.as_ref();
        let password = password.as_ref();
        let encrypted_password = encrypt_password(password).await?;

        let row = ctx
            .txns()
            .pg()
            .query_one(
                "SELECT object FROM user_create_v1($1, $2, $3)",
                &[&name, &email, &encrypted_password.as_ref()],
            )
            .await?;

        // Inlined `finish_create_from_row`

        let json: serde_json::Value = row.try_get("object")?;
        let object: Self = serde_json::from_value(json)?;

        // HistoryEvent won't be accessible by any tenancy (null tenancy_workspace_pk)
        let _history_event = HistoryEvent::new(
            ctx,
            "user.create".to_owned(),
            "User created".to_owned(),
            &serde_json::json![{ "visibility": ctx.visibility() }],
        )
        .await?;

        Ok(object)
    }

    pub async fn get_by_pk(ctx: &DalContext, pk: UserPk) -> UserResult<Option<User>> {
        let maybe_row = ctx.txns().pg().query_opt(USER_GET_BY_PK, &[&pk]).await?;

        let result = match maybe_row {
            Some(row) => {
                let json: serde_json::Value = row.try_get("object")?;
                Some(serde_json::from_value(json)?)
            }
            None => None,
        };
        Ok(result)
    }

    pub async fn find_by_email(
        ctx: &DalContext,
        email: impl AsRef<str>,
    ) -> UserResult<Option<User>> {
        let email = email.as_ref();
        let maybe_row = ctx
            .txns()
            .pg()
            .query_opt(USER_FIND_BY_EMAIL, &[&email])
            .await?;

        let result = match maybe_row {
            Some(row) => {
                let json: serde_json::Value = row.try_get("object")?;
                Some(serde_json::from_value(json)?)
            }
            None => None,
        };
        Ok(result)
    }

    pub async fn authorize(_ctx: &DalContext, _user_pk: &UserPk) -> UserResult<bool> {
        // TODO(paulo,theo): implement capabilities through auth0
        Ok(true)
    }

    pub async fn login(
        &self,
        ctx: &DalContext,
        jwt_secret_key: &JwtSecretKey,
        workspace_pk: &WorkspacePk,
        password: impl Into<String>,
    ) -> UserResult<String> {
        let row = ctx
            .txns()
            .pg()
            .query_one(USER_PASSWORD, &[&self.pk()])
            .await?;
        let current_password: Vec<u8> = row.try_get("password")?;
        let verified = verify_password(password, current_password).await?;
        if !verified {
            return Err(UserError::MismatchedPassword);
        }
        let user_claim = UserClaim::new(self.pk(), *workspace_pk);

        let claims = Claims::with_custom_claims(user_claim, Duration::from_days(1))
            .with_audience("https://app.systeminit.com")
            .with_issuer("https://app.systeminit.com")
            .with_subject(self.pk());

        let signing_key = get_jwt_signing_key(ctx, jwt_secret_key).await?;
        let jwt = signing_key.sign(claims)?;
        Ok(jwt)
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct UserClaim {
    pub user_pk: UserPk,
    pub workspace_pk: WorkspacePk,
}

impl UserClaim {
    pub fn new(user_pk: UserPk, workspace_pk: WorkspacePk) -> Self {
        UserClaim {
            user_pk,
            workspace_pk,
        }
    }

    pub async fn from_bearer_token(
        ctx: &DalContext,
        token: impl AsRef<str>,
    ) -> UserResult<UserClaim> {
        let claims = crate::jwt_key::validate_bearer_token(ctx, &token).await?;
        Ok(claims.custom)
    }

    pub async fn find_billing_account_pk_for_workspace(
        &self,
        ctx: &DalContext,
    ) -> UserResult<BillingAccountPk> {
        Ok(ctx
            .find_billing_account_pk_for_workspace(self.workspace_pk)
            .await?)
    }
}

#[instrument(skip_all)]
pub async fn encrypt_password(
    password: impl Into<String>,
) -> UserResult<argon2id13::HashedPassword> {
    let password = password.into();
    let password_hash: UserResult<argon2id13::HashedPassword> =
        tokio::task::spawn_blocking(move || {
            let password_hash = argon2id13::pwhash(
                password.as_bytes(),
                argon2id13::OPSLIMIT_INTERACTIVE,
                argon2id13::MEMLIMIT_INTERACTIVE,
            )
            .map_err(|()| UserError::PasswordHash)?;
            Ok(password_hash)
        })
        .await?;
    password_hash
}

#[instrument(skip_all)]
pub async fn verify_password(
    password: impl Into<String>,
    password_hash: impl Into<Vec<u8>>,
) -> UserResult<bool> {
    let password = password.into();
    let password_hash = password_hash.into();
    let result = tokio::task::spawn_blocking(move || {
        let password_hash = password_hash.as_ref();
        let password_bytes = password.as_bytes();
        if let Some(argon_password) = argon2id13::HashedPassword::from_slice(password_hash) {
            argon2id13::pwhash_verify(&argon_password, password_bytes)
        } else {
            false
        }
    })
    .await?;
    Ok(result)
}
