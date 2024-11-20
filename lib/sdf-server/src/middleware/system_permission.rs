use std::task::{Context, Poll};

use axum::{
    body::Body,
    extract::FromRequestParts,
    http::Request,
    response::{IntoResponse, Response},
};
use futures::future::BoxFuture;
use permissions::{Permission, PermissionBuilder};
use tower::{Layer, Service};

use crate::{
    extract::{self, Authorization},
    AppState,
};

#[derive(Clone)]
pub struct SystemPermissionLayer {
    state: AppState,
    permission: Permission,
}

impl SystemPermissionLayer {
    pub fn new(state: AppState, permission: Permission) -> Self {
        Self { state, permission }
    }
}

impl<S> Layer<S> for SystemPermissionLayer {
    type Service = SystemPermission<S>;

    fn layer(&self, inner: S) -> Self::Service {
        SystemPermission {
            inner,
            state: self.state.clone(),
            permission: self.permission,
        }
    }
}

#[derive(Clone)]
pub struct SystemPermission<S> {
    inner: S,
    state: AppState,
    permission: Permission,
}

impl<S> Service<Request<Body>> for SystemPermission<S>
where
    S: Service<Request<Body>, Response = Response> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let mut me = self.clone();

        Box::pin(async move {
            let (mut parts, body) = req.into_parts();

            let Authorization(claim) =
                match Authorization::from_request_parts(&mut parts, &me.state).await {
                    Ok(claim) => claim,
                    Err(err) => return Ok(err.into_response()),
                };

            if let Some(client) = me.state.spicedb_client() {
                let is_allowed = match PermissionBuilder::new()
                    .system_object()
                    .permission(me.permission)
                    .user_subject(claim.user_pk.into())
                    .has_permission(client)
                    .await
                {
                    Ok(is_allowed) => is_allowed,
                    Err(_) => return Ok(extract::unauthorized_error().into_response()),
                };
                if !is_allowed {
                    return Ok(extract::unauthorized_error().into_response());
                }
            }

            let req = Request::from_parts(parts, body);

            let response = me.inner.call(req).await?;
            Ok(response)
        })
    }
}
