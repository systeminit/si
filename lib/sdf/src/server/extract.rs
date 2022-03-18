use std::{collections::HashMap, fmt, sync::Arc};

use axum::{
    async_trait,
    extract::{Extension, FromRequest, Query, RequestParts},
    Json,
};
use dal::{
    context::{self, DalContextBuilder, ServicesContext, Transactions, TransactionsError},
    ChangeSetPk, EditSessionPk, ReadTenancy, StandardModel, User, UserClaim, Visibility, Workspace,
    WorkspaceId, WriteTenancy,
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
        let pg_txn = pg_ro_txn.start().await.map_err(internal_error)?;

        let history_actor = dal::HistoryActor::from(claim.user_id);
        let Tenancy(write_tenancy, read_tenancy) =
            tenancy_from_request(req, &claim, &pg_txn).await?;

        Ok(Self(context::AccessBuilder::new(
            read_tenancy,
            write_tenancy,
            history_actor,
        )))
    }
}

pub struct TransactionsStarter {
    pg_conn: pg::InstrumentedClient,
    nats_conn: nats::Client,
}

impl TransactionsStarter {
    pub async fn start(&mut self) -> Result<Transactions<'_>, TransactionsError> {
        let pg_txn = self.pg_conn.transaction().await?;
        let nats_txn = self.nats_conn.transaction();
        Ok(Transactions::new(pg_txn, nats_txn))
    }
}

pub struct HandlerContext(pub DalContextBuilder, pub TransactionsStarter);

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
        let (dal_ctx_builder, pg_conn) = services_context
            .into_builder_and_pg_conn()
            .await
            .map_err(internal_error)?;
        let nats_conn = dal_ctx_builder.nats_conn().clone();
        Ok(Self(
            dal_ctx_builder,
            TransactionsStarter { pg_conn, nats_conn },
        ))
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
    type Rejection = (StatusCode, Json<serde_json::Value>);

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

pub struct Authorization(pub UserClaim);

#[async_trait]
impl<P> FromRequest<P> for Authorization
where
    P: Send,
{
    type Rejection = (StatusCode, Json<serde_json::Value>);

    async fn from_request(req: &mut RequestParts<P>) -> Result<Self, Self::Rejection> {
        let mut ro_txn = PgRoTxn::from_request(req).await?;
        let txn = ro_txn.start().await.map_err(internal_error)?;

        let headers = req.headers().ok_or_else(unauthorized_error)?;
        let authorization_header_value = headers
            .get("Authorization")
            .ok_or_else(unauthorized_error)?;
        let authorization = authorization_header_value
            .to_str()
            .map_err(internal_error)?;
        let claim = UserClaim::from_bearer_token(&txn, authorization)
            .await
            .map_err(|_| unauthorized_error())?;
        let read_tenancy = ReadTenancy::new_billing_account(vec![claim.billing_account_id]);
        let visibility = Visibility::new_head(false);
        User::authorize(&txn, &read_tenancy, &visibility, &claim.user_id)
            .await
            .map_err(|_| unauthorized_error())?;
        txn.commit().await.map_err(internal_error)?;

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
        let mut ro_txn = PgRoTxn::from_request(req).await?;
        let txn = ro_txn.start().await.map_err(internal_error)?;

        let query: Query<HashMap<String, String>> = Query::from_request(req)
            .await
            .map_err(|_| unauthorized_error())?;
        let authorization = query.get("token").ok_or_else(unauthorized_error)?;

        let claim = UserClaim::from_bearer_token(&txn, authorization)
            .await
            .map_err(|_| unauthorized_error())?;
        let read_tenancy = ReadTenancy::new_billing_account(vec![claim.billing_account_id]);
        let visibility = Visibility::new_head(false);
        User::authorize(&txn, &read_tenancy, &visibility, &claim.user_id)
            .await
            .map_err(|_| unauthorized_error())?;
        txn.commit().await.map_err(internal_error)?;

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
        let err_msg = "bad or missing visibility";

        let query: Query<HashMap<String, String>> = Query::from_request(req)
            .await
            .map_err(|_| not_acceptable_error(err_msg))?;
        let change_set_pk_string = query
            .get("visibility_change_set_pk")
            .ok_or_else(|| not_acceptable_error(err_msg))?;
        let change_set_pk: ChangeSetPk = change_set_pk_string
            .parse()
            .map_err(|_| not_acceptable_error(err_msg))?;
        let edit_session_pk_string = query
            .get("visibility_edit_session_pk")
            .ok_or_else(|| not_acceptable_error(err_msg))?;
        let edit_session_pk: EditSessionPk = edit_session_pk_string
            .parse()
            .map_err(|_| not_acceptable_error(err_msg))?;
        let deleted_string = query
            .get("visibility_deleted")
            .ok_or_else(|| not_acceptable_error(err_msg))?;
        let deleted = match deleted_string.as_ref() {
            "0" => true,
            "1" => false,
            _ => return Err(not_acceptable_error(err_msg)),
        };
        let visibility = Visibility::new(change_set_pk, edit_session_pk, deleted);
        Ok(Self(visibility))
    }
}

pub struct QueryWorkspaceTenancy(pub dal::Tenancy);

#[async_trait]
impl<P> FromRequest<P> for QueryWorkspaceTenancy
where
    P: Send,
{
    type Rejection = (StatusCode, Json<serde_json::Value>);

    async fn from_request(req: &mut RequestParts<P>) -> Result<Self, Self::Rejection> {
        let err_msg = "bad or missing workspace id";

        let query: Query<HashMap<String, String>> = Query::from_request(req)
            .await
            .map_err(|_| not_acceptable_error(err_msg))?;
        let workspace_id_string = query
            .get("workspaceId")
            .ok_or_else(|| not_acceptable_error(err_msg))?;

        let QueryVisibility(visibility) = QueryVisibility::from_request(req).await?;
        let mut ro_txn = PgRoTxn::from_request(req).await?;
        let txn = ro_txn.start().await.map_err(internal_error)?;

        let Authorization(claim) = Authorization::from_request(req).await?;
        let billing_account_tenancy =
            dal::Tenancy::new_billing_account(vec![claim.billing_account_id]);

        let workspace_id: WorkspaceId = workspace_id_string
            .parse()
            .map_err(|_| not_acceptable_error(err_msg))?;

        let _workspace =
            Workspace::get_by_id(&txn, &billing_account_tenancy, &visibility, &workspace_id)
                .await
                .map_err(|_| not_acceptable_error(err_msg))?;

        let tenancy = dal::Tenancy::new_workspace(vec![workspace_id]);

        Ok(Self(tenancy))
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
        let mut pg_ro_txn = PgRoTxn::from_request(req).await?;
        let pg_txn = pg_ro_txn.start().await.map_err(internal_error)?;

        tenancy_from_request(req, &claim, &pg_txn).await
    }
}

async fn tenancy_from_request<P: Send>(
    req: &mut RequestParts<P>,
    claim: &UserClaim,
    pg_txn: &pg::PgTxn<'_>,
) -> Result<Tenancy, (StatusCode, Json<serde_json::Value>)> {
    let headers = req
        .headers()
        .ok_or_else(|| not_acceptable_error("headers not found for request"))?;
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
        // Empty tenancy means things can be written, but they will never match any read
        // tenancy, so they won't ever be read
        WriteTenancy::from(&dal::Tenancy::new_empty())
    };

    let read_tenancy = write_tenancy
        .clone_into_read_tenancy(pg_txn)
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
