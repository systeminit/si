use std::task::{
    Context,
    Poll,
};

use axum::{
    RequestPartsExt as _,
    body::Body,
    http::Request,
    response::{
        IntoResponse,
        Response,
    },
};
use futures::future::BoxFuture;
use permissions::{
    Permission,
    PermissionBuilder,
};
use tower::{
    Layer,
    Service,
};

use crate::{
    AppState,
    extract::{
        unauthorized_error,
        workspace::WorkspaceAuthorization,
    },
};

#[derive(Clone)]
pub struct WorkspacePermissionLayer {
    state: AppState,
    permission: Permission,
}

impl WorkspacePermissionLayer {
    pub fn new(state: AppState, permission: Permission) -> Self {
        Self { state, permission }
    }
}

impl<S> Layer<S> for WorkspacePermissionLayer {
    type Service = WorkspacePermission<S>;

    fn layer(&self, inner: S) -> Self::Service {
        WorkspacePermission {
            inner,
            state: self.state.clone(),
            permission: self.permission,
        }
    }
}

#[derive(Clone)]
pub struct WorkspacePermission<S> {
    inner: S,
    state: AppState,
    permission: Permission,
}

impl<S> Service<Request<Body>> for WorkspacePermission<S>
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

            let (user_id, workspace_id) = match parts.extract_with_state(&me.state).await {
                Ok(WorkspaceAuthorization {
                    user, workspace_id, ..
                }) => (user.pk(), workspace_id),
                Err(err) => return Ok(err.into_response()),
            };

            if let Some(client) = me.state.spicedb_client() {
                let is_allowed = match PermissionBuilder::new()
                    .workspace_object(workspace_id)
                    .permission(me.permission)
                    .user_subject(user_id)
                    .has_permission(client)
                    .await
                {
                    Ok(is_allowed) => is_allowed,
                    Err(e) => return Ok(unauthorized_error(e).into_response()),
                };
                if !is_allowed {
                    return Ok(unauthorized_error(format!(
                        "not authorized to {} workspace",
                        me.permission
                    ))
                    .into_response());
                }
            }

            let req = Request::from_parts(parts, body);

            let response = me.inner.call(req).await?;
            Ok(response)
        })
    }
}
