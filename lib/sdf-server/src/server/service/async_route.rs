use dal::{DalContext, WsEvent};
use hyper::Uri;
use telemetry::prelude::*;

pub type TaskId = ulid::Ulid;

/// Handler for any fatal error condition in an "async" SDF route (one that does
/// work on a background thread and returns the result via a WsEvent)
pub async fn handle_error(
    ctx: &DalContext,
    uri: Uri,
    task_id: TaskId,
    err: impl std::error::Error,
) {
    let err_string = err.to_string();
    error!("async route '{}' error: {}", uri.to_string(), err_string);
    match WsEvent::async_error(ctx, task_id, err_string).await {
        Ok(event) => {
            if let Err(commit_err) = event.publish_immediately(ctx).await {
                error!("Unable to publish ws event for async error: {commit_err}");
            }
        }
        Err(creation_err) => {
            error!("Unable to create ws event for async error: {creation_err}")
        }
    }
}
