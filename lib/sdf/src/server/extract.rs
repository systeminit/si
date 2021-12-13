use axum::extract::Query;
use axum::{
    async_trait,
    extract::{Extension, FromRequest, RequestParts},
    Json,
};
use dal::{ChangeSetPk, EditSessionPk, StandardModel, User, Workspace, WorkspaceId};
use dal::{Tenancy, UserClaim, Visibility};
use hyper::StatusCode;
use serde::Serialize;
use si_data::{nats, pg};
use std::collections::HashMap;

pub struct PgPool(pub pg::PgPool);

#[async_trait]
impl<P> FromRequest<P> for PgPool
where
    P: Send,
{
    type Rejection = (StatusCode, Json<InternalError>);

    async fn from_request(req: &mut RequestParts<P>) -> Result<Self, Self::Rejection> {
        let Extension(pg_pool) = Extension::<pg::PgPool>::from_request(req)
            .await
            .map_err(internal_error)?;
        Ok(Self(pg_pool))
    }
}

pub struct PgConn(pub pg::InstrumentedClient);

#[async_trait]
impl<P> FromRequest<P> for PgConn
where
    P: Send,
{
    type Rejection = (StatusCode, Json<InternalError>);

    async fn from_request(req: &mut RequestParts<P>) -> Result<Self, Self::Rejection> {
        let PgPool(pg_pool) = PgPool::from_request(req).await?;
        let pg_conn = pg_pool.get().await.map_err(internal_error)?;
        Ok(Self(pg_conn))
    }
}

pub struct PgRwTxn(pg::InstrumentedClient);

#[async_trait]
impl<P> FromRequest<P> for PgRwTxn
where
    P: Send,
{
    type Rejection = (StatusCode, Json<InternalError>);

    async fn from_request(req: &mut RequestParts<P>) -> Result<Self, Self::Rejection> {
        let PgConn(pg_conn) = PgConn::from_request(req).await?;
        Ok(Self(pg_conn))
    }
}

impl PgRwTxn {
    pub async fn start(&mut self) -> Result<pg::PgTxn<'_>, pg::Error> {
        self.0.transaction().await
    }
}

pub struct PgRoTxn(pg::InstrumentedClient);

#[async_trait]
impl<P> FromRequest<P> for PgRoTxn
where
    P: Send,
{
    type Rejection = (StatusCode, Json<InternalError>);

    async fn from_request(req: &mut RequestParts<P>) -> Result<Self, Self::Rejection> {
        let PgConn(pg_conn) = PgConn::from_request(req).await?;
        Ok(Self(pg_conn))
    }
}

impl PgRoTxn {
    pub async fn start(&mut self) -> Result<pg::PgTxn<'_>, pg::Error> {
        self.0.build_transaction().read_only(true).start().await
    }
}

pub struct Nats(pub nats::Client);

#[async_trait]
impl<P> FromRequest<P> for Nats
where
    P: Send,
{
    type Rejection = (StatusCode, Json<InternalError>);

    async fn from_request(req: &mut RequestParts<P>) -> Result<Self, Self::Rejection> {
        let Extension(nats) = Extension::<nats::Client>::from_request(req)
            .await
            .map_err(internal_error)?;
        Ok(Self(nats))
    }
}

pub struct NatsTxn(nats::Client);

#[async_trait]
impl<P> FromRequest<P> for NatsTxn
where
    P: Send,
{
    type Rejection = (StatusCode, Json<InternalError>);

    async fn from_request(req: &mut RequestParts<P>) -> Result<Self, Self::Rejection> {
        let Nats(nats) = Nats::from_request(req).await?;
        Ok(Self(nats))
    }
}

impl NatsTxn {
    pub async fn start(&mut self) -> Result<nats::NatsTxn, nats::Error> {
        Ok(self.0.transaction())
    }
}

#[derive(Debug, Serialize)]
pub struct InternalError {
    error: String,
}

fn internal_error(err: impl std::error::Error) -> (StatusCode, Json<InternalError>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(InternalError {
            error: err.to_string(),
        }),
    )
}

pub struct JwtSigningKey(crate::JwtSigningKey);

#[async_trait]
impl<P> FromRequest<P> for JwtSigningKey
where
    P: Send,
{
    type Rejection = (StatusCode, Json<InternalError>);

    async fn from_request(req: &mut RequestParts<P>) -> Result<Self, Self::Rejection> {
        let Extension(key) = Extension::<crate::server::config::JwtSigningKey>::from_request(req)
            .await
            .map_err(internal_error)?;
        Ok(Self(key))
    }
}

impl JwtSigningKey {
    pub fn key(&self) -> &sodiumoxide::crypto::secretbox::Key {
        &self.0.key
    }
}

pub struct Authorization(pub UserClaim);

#[async_trait]
impl<P> FromRequest<P> for Authorization
where
    P: Send,
{
    type Rejection = (StatusCode, Json<serde_json::Value>);

    async fn from_request(req: &mut RequestParts<P>) -> Result<Self, Self::Rejection> {
        let error_response = (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({
                "error": {
                    "message": "unauthorized",
                    "code": 42,
                    "statusCode": 401,
                },
            })),
        );

        let mut ro_txn = PgRoTxn::from_request(req)
            .await
            .map_err(|_| error_response.clone())?;
        let txn = ro_txn.start().await.map_err(|_| error_response.clone())?;

        let headers = req.headers().ok_or_else(|| error_response.clone())?;
        let authorization_header_value = headers
            .get("Authorization")
            .ok_or_else(|| error_response.clone())?;
        let authorization = authorization_header_value
            .to_str()
            .map_err(|_| error_response.clone())?;
        let claim = UserClaim::from_bearer_token(&txn, authorization)
            .await
            .map_err(|_| error_response.clone())?;
        let tenancy = Tenancy::new_billing_account(vec![claim.billing_account_id]);
        let visibility = Visibility::new_head(false);
        User::authorize(&txn, &tenancy, &visibility, &claim.user_id)
            .await
            .map_err(|_| error_response.clone())?;
        txn.commit().await.map_err(|_| error_response.clone())?;

        Ok(Self(claim))
    }
}

pub struct WsAuthorization(pub UserClaim);

#[async_trait]
impl<P> FromRequest<P> for WsAuthorization
where
    P: Send,
{
    type Rejection = (StatusCode, Json<serde_json::Value>);

    async fn from_request(req: &mut RequestParts<P>) -> Result<Self, Self::Rejection> {
        let error_response = (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({
                "error": {
                    "message": "unauthorized",
                    "code": 42,
                    "statusCode": 401,
                },
            })),
        );

        let mut ro_txn = PgRoTxn::from_request(req)
            .await
            .map_err(|_| error_response.clone())?;
        let txn = ro_txn.start().await.map_err(|_| error_response.clone())?;

        let query: Query<HashMap<String, String>> = Query::from_request(req)
            .await
            .map_err(|_| error_response.clone())?;
        let authorization = query.get("token").ok_or_else(|| error_response.clone())?;

        let claim = UserClaim::from_bearer_token(&txn, authorization)
            .await
            .map_err(|_| error_response.clone())?;
        let tenancy = Tenancy::new_billing_account(vec![claim.billing_account_id]);
        let visibility = Visibility::new_head(false);
        User::authorize(&txn, &tenancy, &visibility, &claim.user_id)
            .await
            .map_err(|_| error_response.clone())?;
        txn.commit().await.map_err(|_| error_response.clone())?;

        Ok(Self(claim))
    }
}

pub struct QueryVisibility(pub Visibility);

#[async_trait]
impl<P> FromRequest<P> for QueryVisibility
where
    P: Send,
{
    type Rejection = (StatusCode, Json<serde_json::Value>);

    async fn from_request(req: &mut RequestParts<P>) -> Result<Self, Self::Rejection> {
        let error_response = (
            StatusCode::NOT_ACCEPTABLE,
            Json(serde_json::json!({
                "error": {
                    "message": "bad or missing visibility",
                    "code": 42,
                    "statusCode": 406,
                },
            })),
        );

        let query: Query<HashMap<String, String>> = Query::from_request(req)
            .await
            .map_err(|_| error_response.clone())?;
        let change_set_pk_string = query
            .get("visibility_change_set_pk")
            .ok_or_else(|| error_response.clone())?;
        let change_set_pk: ChangeSetPk = change_set_pk_string
            .parse()
            .map_err(|_| error_response.clone())?;
        let edit_session_pk_string = query
            .get("visibility_edit_session_pk")
            .ok_or_else(|| error_response.clone())?;
        let edit_session_pk: EditSessionPk = edit_session_pk_string
            .parse()
            .map_err(|_| error_response.clone())?;
        let deleted_string = query
            .get("visibility_deleted")
            .ok_or_else(|| error_response.clone())?;
        let deleted = match deleted_string.as_ref() {
            "0" => true,
            "1" => false,
            _ => return Err(error_response.clone()),
        };
        let visibility = Visibility::new(change_set_pk, edit_session_pk, deleted);
        Ok(Self(visibility))
    }
}

pub struct QueryWorkspaceTenancy(pub Tenancy);

#[async_trait]
impl<P> FromRequest<P> for QueryWorkspaceTenancy
where
    P: Send,
{
    type Rejection = (StatusCode, Json<serde_json::Value>);

    async fn from_request(req: &mut RequestParts<P>) -> Result<Self, Self::Rejection> {
        let error_response = (
            StatusCode::NOT_ACCEPTABLE,
            Json(serde_json::json!({
                "error": {
                    "message": "bad or missing workspace id",
                    "code": 42,
                    "statusCode": 406,
                },
            })),
        );

        let query: Query<HashMap<String, String>> = Query::from_request(req)
            .await
            .map_err(|_| error_response.clone())?;
        let workspace_id_string = query
            .get("workspaceId")
            .ok_or_else(|| error_response.clone())?;

        let QueryVisibility(visibility) =
            QueryVisibility::from_request(req).await.map_err(|e| {
                dbg!(e);
                error_response.clone()
            })?;
        let mut ro_txn = PgRoTxn::from_request(req)
            .await
            .map_err(|_| error_response.clone())?;
        let txn = ro_txn.start().await.map_err(|_| error_response.clone())?;

        let Authorization(claim) = Authorization::from_request(req)
            .await
            .map_err(|_| error_response.clone())?;
        let billing_account_tenancy = Tenancy::new_billing_account(vec![claim.billing_account_id]);

        let workspace_id: WorkspaceId = workspace_id_string
            .parse()
            .map_err(|_| error_response.clone())?;

        let _workspace =
            Workspace::get_by_id(&txn, &billing_account_tenancy, &visibility, &workspace_id)
                .await
                .map_err(|_| error_response.clone())?;

        let tenancy = Tenancy::new_workspace(vec![workspace_id]);

        Ok(Self(tenancy))
    }
}
