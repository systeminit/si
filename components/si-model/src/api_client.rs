use crate::{get_jwt_signing_key, Group, GroupError, JwtKeyError, SimpleStorable};
use blake2::{Blake2b, Digest};
use jwt_simple::algorithms::RSAKeyPairLike;
use jwt_simple::claims::Claims;
use jwt_simple::coarsetime::Duration;
use serde::{Deserialize, Serialize};
use si_data::{NatsTxn, NatsTxnError, PgTxn};
use sodiumoxide::crypto::secretbox;
use thiserror::Error;

const AUTHORIZE_API_CLIENT: &str = include_str!("./queries/authorize_api_client.sql");

#[derive(Error, Debug)]
pub enum ApiClientError {
    #[error("invalid uft-8 string: {0}")]
    Utf8(#[from] std::str::Utf8Error),
    #[error("user is not found")]
    NotFound,
    #[error("API client name already exists")]
    NameExists,
    #[error("API Client is unauthorized")]
    Unauthorized,
    #[error("jwt private key: {0}")]
    JwtKey(#[from] JwtKeyError),
    #[error("jwt signing error: {0}")]
    SignError(String),
    #[error("group: {0}")]
    Group(#[from] GroupError),
    #[error("pg error: {0}")]
    TokioPg(#[from] tokio_postgres::Error),
    #[error("nats txn error: {0}")]
    NatsTxn(#[from] NatsTxnError),
    #[error("serde error: {0}")]
    SerdeJson(#[from] serde_json::Error),
}

pub type ApiClientResult<T> = Result<T, ApiClientError>;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ApiClientKind {
    Cli,
}

impl std::fmt::Display for ApiClientKind {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            &ApiClientKind::Cli => write!(f, "cli"),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ApiClient {
    pub id: String,
    pub name: String,
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
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        secret_key: &secretbox::Key,
        name: impl Into<String>,
        kind: ApiClientKind,
        billing_account_id: impl Into<String>,
    ) -> ApiClientResult<(ApiClient, String)> {
        let name = name.into();
        let billing_account_id = billing_account_id.into();

        let row = txn
            .query_one(
                "SELECT object FROM api_client_create_v1($1, $2, $3)",
                &[&name, &billing_account_id, &kind.to_string()],
            )
            .await?;
        let json: serde_json::Value = row.try_get("object")?;
        nats.publish(&json).await?;
        let object: ApiClient = serde_json::from_value(json)?;

        let signing_key = get_jwt_signing_key(&txn, secret_key).await?;
        let si_claims = ApiClaim {
            api_client_id: object.id.clone(),
            billing_account_id: billing_account_id.clone(),
        };
        let claims = Claims::with_custom_claims(si_claims, Duration::from_days(7300))
            .with_audience("https://app.systeminit.com")
            .with_issuer("https://app.systeminit.com")
            .with_subject(object.id.clone());
        let jwt = signing_key
            .sign(claims)
            .map_err(|e| ApiClientError::SignError(e.to_string()))?;
        let valid_token_hash =
            format!("{:x}", Blake2b::digest(format!("Bearer {}", &jwt).as_ref()));

        txn.execute(
            "SELECT api_client_set_valid_token_hash_v1($1, $2)",
            &[&object.id, &valid_token_hash],
        )
        .await?;

        let mut admin_group = Group::get_administrators_group(&txn, billing_account_id).await?;
        admin_group.api_client_ids.push(object.id.clone());
        admin_group.save(&txn, &nats).await?;

        Ok((object, jwt))
    }

    pub async fn get(txn: &PgTxn<'_>, id: impl AsRef<str>) -> ApiClientResult<ApiClient> {
        let id = id.as_ref();
        let row = txn
            .query_one("SELECT object FROM api_client_get_v1($1)", &[&id])
            .await?;
        let json: serde_json::Value = row.try_get("object")?;
        let object = serde_json::from_value(json)?;
        Ok(object)
    }
}

pub async fn authorize(
    txn: &PgTxn<'_>,
    api_client_id: impl AsRef<str>,
    subject: impl AsRef<str>,
    action: impl AsRef<str>,
) -> ApiClientResult<()> {
    let api_client_id = api_client_id.as_ref();
    let subject = subject.as_ref();
    let action = action.as_ref();
    let _row = txn
        .query_one(AUTHORIZE_API_CLIENT, &[&api_client_id, &subject, &action])
        .await
        .map_err(|_| ApiClientError::Unauthorized)?;
    Ok(())
}

pub async fn authenticate(txn: &PgTxn<'_>, token: impl AsRef<str>) -> ApiClientResult<ApiClaim> {
    let token = token.as_ref();
    let claims = crate::jwt_key::validate_bearer_token_api_client(&txn, token).await?;
    Ok(claims.custom)
}
