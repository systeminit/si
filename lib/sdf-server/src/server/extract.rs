use std::{collections::HashMap, fmt, sync::Arc};

use axum::{
    async_trait,
    extract::{Extension, FromRequest, Query, RequestParts},
    Json,
};
use dal::{
    context::{self, DalContextBuilder, ServicesContext},
    RequestContext, User, UserClaim,
};
use hyper::StatusCode;
use si_data_pg::{InstrumentedClient, InstrumentedTransaction, PgError};
use veritech_client::Client as VeritechClient;

pub struct AccessBuilder(pub context::AccessBuilder);

#[async_trait]
impl<P> FromRequest<P> for AccessBuilder
where
    P: Send,
{
    type Rejection = (StatusCode, Json<serde_json::Value>);

    async fn from_request(req: &mut RequestParts<P>) -> Result<Self, Self::Rejection> {
        let Authorization(claim) = Authorization::from_request(req).await?;
        let Tenancy(tenancy) = tenancy_from_claim(&claim).await?;

        Ok(Self(context::AccessBuilder::new(
            tenancy,
            dal::HistoryActor::from(claim.user_pk),
        )))
    }
}

pub struct HandlerContext(pub DalContextBuilder);

#[async_trait]
impl<P> FromRequest<P> for HandlerContext
where
    P: Send,
{
    type Rejection = (StatusCode, Json<serde_json::Value>);

    async fn from_request(req: &mut RequestParts<P>) -> Result<Self, Self::Rejection> {
        let Extension(services_context) = Extension::<ServicesContext>::from_request(req)
            .await
            .map_err(internal_error)?;
        let builder = services_context.into_builder();
        Ok(Self(builder))
    }
}

pub struct PgPool(pub si_data_pg::PgPool);

#[async_trait]
impl<P> FromRequest<P> for PgPool
where
    P: Send,
{
    type Rejection = (StatusCode, Json<serde_json::Value>);

    async fn from_request(req: &mut RequestParts<P>) -> Result<Self, Self::Rejection> {
        let Extension(pg_pool) = Extension::<si_data_pg::PgPool>::from_request(req)
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
    type Rejection = (StatusCode, Json<serde_json::Value>);

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
    type Rejection = (StatusCode, Json<serde_json::Value>);

    async fn from_request(req: &mut RequestParts<P>) -> Result<Self, Self::Rejection> {
        let PgConn(pg_conn) = PgConn::from_request(req).await?;
        Ok(Self(pg_conn))
    }
}

impl PgRwTxn {
    pub async fn start(&mut self) -> Result<InstrumentedTransaction<'_>, PgError> {
        self.0.transaction().await
    }
}

#[derive(Debug)]
pub struct PgRoTxn(InstrumentedClient);

#[async_trait]
impl<P> FromRequest<P> for PgRoTxn
where
    P: Send,
{
    type Rejection = (StatusCode, Json<serde_json::Value>);

    async fn from_request(req: &mut RequestParts<P>) -> Result<Self, Self::Rejection> {
        let PgConn(pg_conn) = PgConn::from_request(req).await?;
        Ok(Self(pg_conn))
    }
}

impl PgRoTxn {
    pub async fn start(&mut self) -> Result<InstrumentedTransaction<'_>, PgError> {
        self.0.build_transaction().read_only(true).start().await
    }
}

pub struct Nats(pub si_data_nats::NatsClient);

#[async_trait]
impl<P> FromRequest<P> for Nats
where
    P: Send,
{
    type Rejection = (StatusCode, Json<serde_json::Value>);

    async fn from_request(req: &mut RequestParts<P>) -> Result<Self, Self::Rejection> {
        let Extension(nats) = Extension::<si_data_nats::NatsClient>::from_request(req)
            .await
            .map_err(internal_error)?;
        Ok(Self(nats))
    }
}

pub struct NatsTxn(si_data_nats::NatsClient);

#[async_trait]
impl<P> FromRequest<P> for NatsTxn
where
    P: Send,
{
    type Rejection = (StatusCode, Json<serde_json::Value>);

    async fn from_request(req: &mut RequestParts<P>) -> Result<Self, Self::Rejection> {
        let Nats(nats) = Nats::from_request(req).await?;
        Ok(Self(nats))
    }
}

impl NatsTxn {
    pub async fn start(&mut self) -> Result<si_data_nats::NatsTxn, si_data_nats::NatsError> {
        Ok(self.0.transaction())
    }
}

pub struct Veritech(pub VeritechClient);

#[async_trait]
impl<P> FromRequest<P> for Veritech
where
    P: Send,
{
    type Rejection = (StatusCode, Json<serde_json::Value>);

    async fn from_request(req: &mut RequestParts<P>) -> Result<Self, Self::Rejection> {
        let Extension(veritech) = Extension::<VeritechClient>::from_request(req)
            .await
            .map_err(internal_error)?;
        Ok(Self(veritech))
    }
}

pub struct EncryptionKey(pub Arc<veritech_client::EncryptionKey>);

#[async_trait]
impl<P> FromRequest<P> for EncryptionKey
where
    P: Send,
{
    type Rejection = (StatusCode, Json<serde_json::Value>);

    async fn from_request(req: &mut RequestParts<P>) -> Result<Self, Self::Rejection> {
        let Extension(encryption_key) =
            Extension::<Arc<veritech_client::EncryptionKey>>::from_request(req)
                .await
                .map_err(internal_error)?;
        Ok(Self(encryption_key))
    }
}

pub struct SignupSecret(pub super::routes::SignupSecret);

#[async_trait]
impl<P> FromRequest<P> for SignupSecret
where
    P: Send,
{
    type Rejection = (StatusCode, Json<serde_json::Value>);

    async fn from_request(req: &mut RequestParts<P>) -> Result<Self, Self::Rejection> {
        let Extension(signup_secret) = Extension::<super::routes::SignupSecret>::from_request(req)
            .await
            .map_err(internal_error)?;
        Ok(Self(signup_secret))
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
        let HandlerContext(builder) = HandlerContext::from_request(req).await?;
        let mut ctx = builder
            .build(RequestContext::default())
            .await
            .map_err(internal_error)?;

        let headers = req.headers();
        let authorization_header_value = headers
            .get("Authorization")
            .ok_or_else(unauthorized_error)?;
        let authorization = authorization_header_value
            .to_str()
            .map_err(internal_error)?;
        let claim = UserClaim::from_bearer_token(&ctx, authorization)
            .await
            .map_err(|_| unauthorized_error())?;
        ctx.update_tenancy(dal::Tenancy::new(claim.workspace_pk));

        User::authorize(&ctx, &claim.user_pk)
            .await
            .map_err(|_| unauthorized_error())?;

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
        let HandlerContext(builder) = HandlerContext::from_request(req).await?;
        let mut ctx = builder
            .build(RequestContext::default())
            .await
            .map_err(internal_error)?;

        let query: Query<HashMap<String, String>> = Query::from_request(req)
            .await
            .map_err(|_| unauthorized_error())?;
        let authorization = query.get("token").ok_or_else(unauthorized_error)?;

        let claim = UserClaim::from_bearer_token(&ctx, authorization)
            .await
            .map_err(|_| unauthorized_error())?;
        ctx.update_tenancy(dal::Tenancy::new(claim.workspace_pk));

        User::authorize(&ctx, &claim.user_pk)
            .await
            .map_err(|_| unauthorized_error())?;

        Ok(Self(claim))
    }
}

pub struct HistoryActor(pub dal::HistoryActor);

#[async_trait]
impl<P> FromRequest<P> for HistoryActor
where
    P: Send,
{
    type Rejection = (StatusCode, Json<serde_json::Value>);

    async fn from_request(req: &mut RequestParts<P>) -> Result<Self, Self::Rejection> {
        let Authorization(claim) = Authorization::from_request(req).await?;
        Ok(Self(dal::HistoryActor::from(claim.user_pk)))
    }
}

pub struct Tenancy(pub dal::Tenancy);

#[async_trait]
impl<P> FromRequest<P> for Tenancy
where
    P: Send,
{
    type Rejection = (StatusCode, Json<serde_json::Value>);

    async fn from_request(req: &mut RequestParts<P>) -> Result<Self, Self::Rejection> {
        let Authorization(claim) = Authorization::from_request(req).await?;
        tenancy_from_claim(&claim).await
    }
}

async fn tenancy_from_claim(
    claim: &UserClaim,
) -> Result<Tenancy, (StatusCode, Json<serde_json::Value>)> {
    Ok(Tenancy(dal::Tenancy::new(claim.workspace_pk)))
}

fn internal_error(message: impl fmt::Display) -> (StatusCode, Json<serde_json::Value>) {
    let status_code = StatusCode::INTERNAL_SERVER_ERROR;
    (
        status_code,
        Json(serde_json::json!({
            "error": {
                "message": message.to_string(),
                "statusCode": status_code.as_u16(),
                "code": 42,
            },
        })),
    )
}

fn unauthorized_error() -> (StatusCode, Json<serde_json::Value>) {
    let status_code = StatusCode::UNAUTHORIZED;
    (
        status_code,
        Json(serde_json::json!({
            "error": {
                "message": "unauthorized",
                "statusCode": status_code.as_u16(),
                "code": 42,
            },
        })),
    )
}
