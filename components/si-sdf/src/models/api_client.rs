use super::upsert_model;
use crate::data::{Connection, Db};
use crate::models::{
    check_secondary_key, generate_id, get_model, insert_model, Group, GroupError, JwtKeyError,
    JwtKeyPrivate, ModelError, SiStorableError, SimpleStorable,
};
use blake2::{Blake2b, Digest};
use jwt_simple::algorithms::RSAKeyPairLike;
use jwt_simple::claims::Claims;
use jwt_simple::coarsetime::Duration;
use serde::{Deserialize, Serialize};
use sodiumoxide::crypto::secretbox;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApiClientError {
    #[error("si_storable error: {0}")]
    SiStorable(#[from] SiStorableError),
    #[error("error in core model functions: {0}")]
    Model(#[from] ModelError),
    #[error("invalid uft-8 string: {0}")]
    Utf8(#[from] std::str::Utf8Error),
    #[error("user is not found")]
    NotFound,
    #[error("database error: {0}")]
    Data(#[from] crate::data::DataError),
    #[error("API client name already exists")]
    NameExists,
    #[error("jwt private key: {0}")]
    JwtKey(#[from] JwtKeyError),
    #[error("jwt signing error: {0}")]
    SignError(String),
    #[error("group: {0}")]
    Group(#[from] GroupError),
}

pub type ApiClientResult<T> = Result<T, ApiClientError>;

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum ApiClientKind {
    Cli,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ApiClient {
    pub id: String,
    pub name: String,
    pub valid_token_hash: String,
    pub kind: ApiClientKind,
    pub si_storable: SimpleStorable,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CreateRequest {
    pub name: String,
    pub kind: ApiClientKind,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CreateReply {
    pub api_client: ApiClient,
    pub token: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ApiClaim {
    pub api_client_id: String,
    pub billing_account_id: String,
}

impl ApiClient {
    pub async fn new(
        db: &Db,
        nats: &Connection,
        secret_key: secretbox::Key,
        name: impl Into<String>,
        kind: ApiClientKind,
        billing_account_id: impl Into<String>,
    ) -> ApiClientResult<(ApiClient, String)> {
        let name = name.into();
        let billing_account_id = billing_account_id.into();

        if check_secondary_key(db, &billing_account_id, "apiClient", "name", &name).await? {
            return Err(ApiClientError::NameExists);
        }

        let id = generate_id("apiClient");
        let si_storable = SimpleStorable::new(&id, "apiClient", &billing_account_id);

        let signing_key = JwtKeyPrivate::get_jwt_signing_key(&db, &secret_key).await?;
        let si_claims = ApiClaim {
            api_client_id: id.clone(),
            billing_account_id: billing_account_id.clone(),
        };
        let claims = Claims::with_custom_claims(si_claims, Duration::from_days(7300))
            .with_audience("https://app.systeminit.com")
            .with_issuer("https://app.systeminit.com")
            .with_subject(id.clone());
        let jwt = signing_key
            .sign(claims)
            .map_err(|e| ApiClientError::SignError(e.to_string()))?;
        let valid_token_hash =
            format!("{:x}", Blake2b::digest(format!("Bearer {}", &jwt).as_ref()));

        let object = ApiClient {
            id: id.clone(),
            name,
            si_storable,
            valid_token_hash,
            kind,
        };
        insert_model(db, nats, &object.id, &object).await?;

        let mut admin_group = Group::get_administrators_group(&db, billing_account_id).await?;
        admin_group.api_client_ids.push(object.id.clone());
        upsert_model(&db, &nats, &admin_group.id, &admin_group).await?;

        Ok((object, jwt))
    }

    pub async fn get(
        db: &Db,
        user_id: impl AsRef<str>,
        billing_account_id: impl AsRef<str>,
    ) -> ApiClientResult<ApiClient> {
        let id = user_id.as_ref();
        let billing_account_id = billing_account_id.as_ref();
        let object: ApiClient = get_model(db, id, billing_account_id).await?;
        Ok(object)
    }
}
