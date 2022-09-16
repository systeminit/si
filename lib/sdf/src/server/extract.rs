use std::{collections::HashMap, fmt, sync::Arc};

use axum::{
    async_trait,
    extract::{Extension, FromRequest, Query, RequestParts},
    Json,
};
use dal::{
    context::{self, DalContextBuilder, ServicesContext},
    ReadTenancy, RequestContext, User, UserClaim, WorkspaceId, WriteTenancy,
};
use hyper::StatusCode;
use si_data::{nats, pg};

pub struct AccessBuilder(pub context::AccessBuilder);

#[async_trait]
impl<P> FromRequest<P> for AccessBuilder
where
    P: Send,
{
    type Rejection = (StatusCode, Json<serde_json::Value>);

    async fn from_request(req: &mut RequestParts<P>) -> Result<Self, Self::Rejection> {
        let Authorization(claim) = Authorization::from_request(req).await?;
        let mut pg_ro_txn = PgRoTxn::from_request(req).await?;
        let _pg_txn = pg_ro_txn.start().await.map_err(internal_error)?;

        let history_actor = dal::HistoryActor::from(claim.user_id);
        let Tenancy(write_tenancy, read_tenancy) = tenancy_from_request(req, &claim).await?;

        Ok(Self(context::AccessBuilder::new(
            read_tenancy,
            write_tenancy,
            history_actor,
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

pub struct PgPool(pub pg::PgPool);

#[async_trait]
impl<P> FromRequest<P> for PgPool
where
    P: Send,
{
    type Rejection = (StatusCode, Json<serde_json::Value>);

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
    type Rejection = (StatusCode, Json<serde_json::Value>);

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
    type Rejection = (StatusCode, Json<serde_json::Value>);

    async fn from_request(req: &mut RequestParts<P>) -> Result<Self, Self::Rejection> {
        let PgConn(pg_conn) = PgConn::from_request(req).await?;
        Ok(Self(pg_conn))
    }
}

impl PgRwTxn {
    pub async fn start(&mut self) -> Result<pg::InstrumentedTransaction<'_>, pg::PgError> {
        self.0.transaction().await
    }
}

pub struct PgRoTxn(pg::InstrumentedClient);

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
    pub async fn start(&mut self) -> Result<pg::InstrumentedTransaction<'_>, pg::PgError> {
        self.0.build_transaction().read_only(true).start().await
    }
}

pub struct Nats(pub nats::Client);

#[async_trait]
impl<P> FromRequest<P> for Nats
where
    P: Send,
{
    type Rejection = (StatusCode, Json<serde_json::Value>);

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
    type Rejection = (StatusCode, Json<serde_json::Value>);

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

pub struct Veritech(pub veritech::Client);

#[async_trait]
impl<P> FromRequest<P> for Veritech
where
    P: Send,
{
    type Rejection = (StatusCode, Json<serde_json::Value>);

    async fn from_request(req: &mut RequestParts<P>) -> Result<Self, Self::Rejection> {
        let Extension(veritech) = Extension::<veritech::Client>::from_request(req)
            .await
            .map_err(internal_error)?;
        Ok(Self(veritech))
    }
}

pub struct EncryptionKey(pub Arc<veritech::EncryptionKey>);

#[async_trait]
impl<P> FromRequest<P> for EncryptionKey
where
    P: Send,
{
    type Rejection = (StatusCode, Json<serde_json::Value>);

    async fn from_request(req: &mut RequestParts<P>) -> Result<Self, Self::Rejection> {
        let Extension(encryption_key) =
            Extension::<Arc<veritech::EncryptionKey>>::from_request(req)
                .await
                .map_err(internal_error)?;
        Ok(Self(encryption_key))
    }
}

pub struct JwtSecretKey(pub crate::JwtSecretKey);

#[async_trait]
impl<P> FromRequest<P> for JwtSecretKey
where
    P: Send,
{
    type Rejection = (StatusCode, Json<serde_json::Value>);

    async fn from_request(req: &mut RequestParts<P>) -> Result<Self, Self::Rejection> {
        let Extension(key) = Extension::<crate::server::config::JwtSecretKey>::from_request(req)
            .await
            .map_err(internal_error)?;
        Ok(Self(key))
    }
}

impl JwtSecretKey {
    pub fn key(&self) -> &sodiumoxide::crypto::secretbox::Key {
        &self.0.key
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
            .build(RequestContext::new_universal_head(
                dal::HistoryActor::SystemInit,
            ))
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
        let read_tenancy = ReadTenancy::new_billing_account(vec![claim.billing_account_id]);
        ctx.update_read_tenancy(read_tenancy);

        User::authorize(&ctx, &claim.user_id)
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
            .build(RequestContext::new_universal_head(
                dal::HistoryActor::SystemInit,
            ))
            .await
            .map_err(internal_error)?;

        let query: Query<HashMap<String, String>> = Query::from_request(req)
            .await
            .map_err(|_| unauthorized_error())?;
        let authorization = query.get("token").ok_or_else(unauthorized_error)?;

        let claim = UserClaim::from_bearer_token(&ctx, authorization)
            .await
            .map_err(|_| unauthorized_error())?;
        let read_tenancy = ReadTenancy::new_billing_account(vec![claim.billing_account_id]);
        ctx.update_read_tenancy(read_tenancy);

        User::authorize(&ctx, &claim.user_id)
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
        Ok(Self(dal::HistoryActor::from(claim.user_id)))
    }
}

pub struct Tenancy(pub WriteTenancy, pub ReadTenancy);

#[async_trait]
impl<P> FromRequest<P> for Tenancy
where
    P: Send,
{
    type Rejection = (StatusCode, Json<serde_json::Value>);

    async fn from_request(req: &mut RequestParts<P>) -> Result<Self, Self::Rejection> {
        let Authorization(claim) = Authorization::from_request(req).await?;
        tenancy_from_request(req, &claim).await
    }
}

async fn tenancy_from_request<P: Send>(
    req: &mut RequestParts<P>,
    claim: &UserClaim,
) -> Result<Tenancy, (StatusCode, Json<serde_json::Value>)> {
    let HandlerContext(builder) = HandlerContext::from_request(req).await?;
    let mut ctx = builder
        .build(RequestContext::new_universal_head(
            dal::HistoryActor::SystemInit,
        ))
        .await
        .map_err(internal_error)?;

    let headers = req.headers();
    let write_tenancy = if let Some(workspace_header_value) = headers.get("WorkspaceId") {
        let workspace_id = workspace_header_value.to_str().map_err(internal_error)?;
        let workspace_id = workspace_id
            .parse::<WorkspaceId>()
            .map_err(not_acceptable_error)?;
        WriteTenancy::new_workspace(workspace_id)
    } else if headers.get("Authorization").is_some() {
        WriteTenancy::new_billing_account(claim.billing_account_id)
    } else {
        // Should only happen at signup where the billing account creation should set the
        // tenancy manually to universal as we don't write to universal implicitly
        //
        // Empty tenancy means things can be written, but won't ever be read
        WriteTenancy::new_empty()
    };
    ctx.update_write_tenancy(write_tenancy.clone());

    let read_tenancy = write_tenancy
        .clone_into_read_tenancy(&ctx)
        .await
        .map_err(internal_error)?;

    // Ensures user can access workspace specified by id
    if !read_tenancy.billing_accounts().is_empty()
        && !read_tenancy
            .billing_accounts()
            .contains(&claim.billing_account_id)
    {
        return Err(not_acceptable_error("failed to determine valid tenacy"));
    }

    Ok(Tenancy(write_tenancy, read_tenancy))
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

fn not_acceptable_error(message: impl fmt::Display) -> (StatusCode, Json<serde_json::Value>) {
    let status_code = StatusCode::NOT_ACCEPTABLE;
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
