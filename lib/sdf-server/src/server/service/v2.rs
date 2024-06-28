use axum::{
    extract::{OriginalUri, Path},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use dal::{schema::variant, ChangeSetId, Schema, SchemaVariant, SchemaVariantId, WorkspacePk};
use si_frontend_types as frontend_types;
use thiserror::Error;

use crate::server::{
    extract::{AccessBuilder, HandlerContext, PosthogClient},
    state::AppState,
    tracking::track,
};
pub mod func;
pub mod variant;

use super::ApiError;

pub fn routes() -> Router<AppState> {
    const PREFIX: &str = "/workspaces/:workspace_id/change-sets/:change_set_id";

    Router::new()
        .nest(&format!("{PREFIX}/schema-variants"), variant::v2_routes())
        .nest(&format!("{PREFIX}/functions"), func::v2_routes())
}
