use crate::WriteTenancy;
use jwt_simple::{algorithms::RSAKeyPairLike, claims::Claims, reexports::coarsetime::Duration};
use serde::{Deserialize, Serialize};
use si_data_nats::NatsError;
use si_data_pg::PgError;
use sodiumoxide::crypto::pwhash::argon2id13;
use telemetry::prelude::*;
use thiserror::Error;
use tokio::task::JoinError;

use crate::jwt_key::{get_jwt_signing_key, JwtKeyError};
use crate::standard_model::option_object_from_row;
use crate::{
    impl_standard_model, pk, standard_model, standard_model_accessor, standard_model_belongs_to,
    BillingAccount, BillingAccountId, DalContext, HistoryEventError, JwtSecretKey, StandardModel,
    StandardModelError, Timestamp, Visibility,
};

const USER_PASSWORD: &str = include_str!("./queries/user_password.sql");
const USER_FIND_BY_EMAIL: &str = include_str!("queries/user_find_by_email.sql");
const AUTHORIZE_USER: &str = include_str!("./queries/authorize_user.sql");

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
    #[error("standard model error: {0}")]
    StandardModelError(#[from] StandardModelError),
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
}

pub type UserResult<T> = Result<T, UserError>;

pk!(UserPk);
pk!(UserId);

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct User {
    pk: UserPk,
    id: UserId,
    name: String,
    email: String,
    #[serde(flatten)]
    tenancy: WriteTenancy,
    #[serde(flatten)]
    timestamp: Timestamp,
    #[serde(flatten)]
    visibility: Visibility,
}

impl_standard_model! {
    model: User,
    pk: UserPk,
    id: UserId,
    table_name: "users",
    history_event_label_base: "user",
    history_event_message_name: "User"
}

impl User {
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
                "SELECT object FROM user_create_v1($1, $2, $3, $4, $5)",
                &[
                    ctx.write_tenancy(),
                    ctx.visibility(),
                    &name,
                    &email,
                    &encrypted_password.as_ref(),
                ],
            )
            .await?;
        let object = standard_model::finish_create_from_row(ctx, row).await?;
        Ok(object)
    }

    standard_model_accessor!(name, String, UserResult);
    standard_model_accessor!(email, String, UserResult);
    standard_model_belongs_to!(
        lookup_fn: billing_account,
        set_fn: set_billing_account,
        unset_fn: unset_billing_account,
        table: "user_belongs_to_billing_account",
        model_table: "billing_accounts",
        belongs_to_id: BillingAccountId,
        returns: BillingAccount,
        result: UserResult,
    );

    pub async fn find_by_email(
        ctx: &DalContext,
        email: impl AsRef<str>,
    ) -> UserResult<Option<User>> {
        let email = email.as_ref();
        let maybe_row = ctx
            .txns()
            .pg()
            .query_opt(
                USER_FIND_BY_EMAIL,
                &[&email, ctx.read_tenancy(), ctx.visibility()],
            )
            .await?;
        let result = option_object_from_row(maybe_row)?;
        Ok(result)
    }

    pub async fn authorize(ctx: &DalContext, user_id: &UserId) -> UserResult<bool> {
        let _row = ctx
            .txns()
            .pg()
            .query_one(
                AUTHORIZE_USER,
                &[ctx.read_tenancy(), ctx.visibility(), &user_id],
            )
            .await?;
        Ok(true)
    }

    pub async fn login(
        &self,
        ctx: &DalContext,
        jwt_secret_key: &JwtSecretKey,
        billing_account_id: &BillingAccountId,
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
        let user_claim = UserClaim::new(*self.id(), *billing_account_id);

        let claims = Claims::with_custom_claims(user_claim, Duration::from_days(1))
            .with_audience("https://app.systeminit.com")
            .with_issuer("https://app.systeminit.com")
            .with_subject(*self.id());

        let signing_key = get_jwt_signing_key(ctx, jwt_secret_key).await?;
        let jwt = signing_key.sign(claims)?;
        Ok(jwt)
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct UserClaim {
    pub user_id: UserId,
    pub billing_account_id: BillingAccountId,
}

impl UserClaim {
    pub fn new(user_id: UserId, billing_account_id: BillingAccountId) -> Self {
        UserClaim {
            user_id,
            billing_account_id,
        }
    }

    pub async fn from_bearer_token(
        ctx: &DalContext,
        token: impl AsRef<str>,
    ) -> UserResult<UserClaim> {
        let claims = crate::jwt_key::validate_bearer_token(ctx, &token).await?;
        Ok(claims.custom)
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
