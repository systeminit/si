use crate::data::{NatsTxn, NatsTxnError, PgTxn};
use crate::models::{
    get_jwt_signing_key, list_model, Group, GroupError, JwtKeyError, ListReply, ModelError,
    OrderByDirection, PageToken, Query, SimpleStorable,
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
    #[error("model error: {0}")]
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

    pub async fn list(
        txn: &PgTxn<'_>,
        tenant_id: impl Into<String>,
        query: Option<Query>,
        page_size: Option<u32>,
        order_by: Option<String>,
        order_by_direction: Option<OrderByDirection>,
        page_token: Option<PageToken>,
    ) -> ApiClientResult<ListReply> {
        let tenant_id = tenant_id.into();
        let reply = list_model(
            txn,
            "api_clients",
            tenant_id,
            query,
            page_size,
            order_by,
            order_by_direction,
            page_token,
        )
        .await?;
        Ok(reply)
    }
}
