use axum::{
    async_trait,
    extract::{Extension, FromRequest, RequestParts},
    Json,
};
use dal::UserClaim;
use hyper::StatusCode;
use serde::Serialize;
use si_data::{nats, pg};

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
            Json(
                serde_json::json!({ "error": { "message": "unauthorized", "code": 42, "statusCode": 401 } }),
            ),
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
            .map_err(|_| error_response)?;
        Ok(Self(claim))
    }
}
