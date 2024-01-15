use std::sync::{
    atomic::{AtomicU32, Ordering},
    Arc,
};

use async_trait::async_trait;
use axum::{
    extract::{Extension, FromRequestParts},
    http::request::Parts,
    Json,
};
use hyper::StatusCode;
use telemetry::prelude::*;
use tokio::sync::mpsc;

use super::server::ShutdownSource;

#[derive(Clone, Debug)]
pub struct RequestLimiter {
    remaining: Arc<Option<AtomicU32>>,
    execution_timeout: Arc<Option<AtomicU32>>,
    shutdown_tx: mpsc::Sender<ShutdownSource>,
}

impl RequestLimiter {
    pub fn new(
        remaining: Arc<Option<AtomicU32>>,
        execution_timeout: Arc<Option<AtomicU32>>,
        shutdown_tx: mpsc::Sender<ShutdownSource>,
    ) -> Self {
        Self {
            remaining,
            execution_timeout,
            shutdown_tx,
        }
    }
}

pub struct LimitRequestGuard(Option<mpsc::Sender<ShutdownSource>>);

impl Drop for LimitRequestGuard {
    fn drop(&mut self) {
        trace!("dropping LimitRequest guard");
        if let Some(tx) = &mut self.0 {
            let tx = tx.clone();
            tokio::spawn(async move {
                trace!("sending shutdown to limit request shutdown receiver");
                if tx.send(ShutdownSource::LimitRequest).await.is_err() {
                    warn!("the limit request shutdown receiver has already been dropped");
                }
            });
        }
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for LimitRequestGuard
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, Json<serde_json::Value>);

    async fn from_request_parts(req: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Extension(limiter) = Extension::<RequestLimiter>::from_request_parts(req, state)
            .await
            .map_err(internal_error)?;

        let shutdown_tx = match (*limiter.remaining).as_ref() {
            Some(remaining) => {
                let mut updated = remaining.load(Ordering::Relaxed);
                updated = updated.saturating_sub(1);
                remaining.store(updated, Ordering::Relaxed);
                debug!("requests remaining: {}", updated);

                if updated > 0 {
                    None
                } else {
                    Some(limiter.shutdown_tx.clone())
                }
            }
            None => None,
        };

        Ok(Self(shutdown_tx))
    }
}

fn internal_error(err: impl std::error::Error) -> (StatusCode, Json<serde_json::Value>) {
    let status_code = StatusCode::INTERNAL_SERVER_ERROR;
    (
        status_code,
        Json(serde_json::json!({
            "error": {
                "message": err.to_string(),
                "statusCode": status_code.as_u16(),
            },
        })),
    )
}
