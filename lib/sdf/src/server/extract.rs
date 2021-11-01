use axum::{
    async_trait,
    extract::{Extension, FromRequest, RequestParts},
    Json,
};
use hyper::StatusCode;
use serde::Serialize;
use si_data::{
    nats,
    pg::{self, InstrumentedClient},
};

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

pub struct PgConn(pub InstrumentedClient);

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

pub struct PgRwTxn(InstrumentedClient);

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

pub struct PgRoTxn(InstrumentedClient);

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

pub struct Nats(pub nats::NatsConn);

#[async_trait]
impl<P> FromRequest<P> for Nats
where
    P: Send,
{
    type Rejection = (StatusCode, Json<InternalError>);

    async fn from_request(req: &mut RequestParts<P>) -> Result<Self, Self::Rejection> {
        let Extension(nats) = Extension::<nats::NatsConn>::from_request(req)
            .await
            .map_err(internal_error)?;
        Ok(Self(nats))
    }
}

pub struct NatsTxn(nats::NatsConn);

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
    pub async fn start(&mut self) -> Result<nats::NatsTxn, nats::NatsTxnError> {
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
