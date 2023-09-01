use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use dal::{
    change_status::ChangeStatusError, ActionError, ActionId, ChangeSetError as DalChangeSetError,
    ComponentError as DalComponentError, FixError, StandardModelError, TransactionsError,
    UserError, UserPk, WsEventError,
};
use module_index_client::IndexClientError;
use telemetry::prelude::*;
use thiserror::Error;

use crate::{server::state::AppState, service::pkg::PkgError};

pub mod add_action;
pub mod apply_change_set;
pub mod create_change_set;
pub mod get_change_set;
pub mod get_stats;
pub mod list_open_change_sets;
pub mod remove_action;
pub mod update_selected_change_set;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum ChangeSetError {
    #[error(transparent)]
    Action(#[from] ActionError),
    #[error("action {0} not found")]
    ActionNotFound(ActionId),
    #[error(transparent)]
    ChangeSet(#[from] DalChangeSetError),
    #[error("change set not found")]
    ChangeSetNotFound,
    #[error(transparent)]
    ChangeStatusError(#[from] ChangeStatusError),
    #[error(transparent)]
    Component(#[from] DalComponentError),
    #[error(transparent)]
    ContextError(#[from] TransactionsError),
    #[error(transparent)]
    DalPkg(#[from] dal::pkg::PkgError),
    #[error(transparent)]
    Fix(#[from] FixError),
    #[error(transparent)]
    IndexClient(#[from] IndexClientError),
    #[error("invalid user {0}")]
    InvalidUser(UserPk),
    #[error("invalid user system init")]
    InvalidUserSystemInit,
    #[error(transparent)]
    Nats(#[from] si_data_nats::NatsError),
    #[error(transparent)]
    Pg(#[from] si_data_pg::PgError),
    #[error(transparent)]
    PkgService(#[from] PkgError),
    #[error(transparent)]
    StandardModel(#[from] StandardModelError),
    #[error(transparent)]
    UrlParse(#[from] url::ParseError),
    #[error(transparent)]
    User(#[from] UserError),
    #[error(transparent)]
    WsEvent(#[from] WsEventError),
}

pub type ChangeSetResult<T> = std::result::Result<T, ChangeSetError>;

impl IntoResponse for ChangeSetError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            ChangeSetError::ChangeSetNotFound => (StatusCode::NOT_FOUND, self.to_string()),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };

        let body = Json(
            serde_json::json!({ "error": { "message": error_message, "code": 42, "statusCode": status.as_u16() } }),
        );

        (status, body).into_response()
    }
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/list_open_change_sets",
            get(list_open_change_sets::list_open_change_sets),
        )
        .route("/remove_action", post(remove_action::remove_action))
        .route("/add_action", post(add_action::add_action))
        .route(
            "/create_change_set",
            post(create_change_set::create_change_set),
        )
        .route("/get_change_set", get(get_change_set::get_change_set))
        .route("/get_stats", get(get_stats::get_stats))
        .route(
            "/apply_change_set",
            post(apply_change_set::apply_change_set),
        )
        .route(
            "/update_selected_change_set",
            post(update_selected_change_set::update_selected_change_set),
        )
}

// Ideally, this would be in a background job (and triggered directly by ChangeSet::apply_raw),
// but we'll need to nail down exactly how the job will auth to the module-index API first.
// Passing the user's access token into the background job processing system is kind of a
// non-starter.
/*
async fn upload_workspace_backup_module(
    ctx: DalContext,
    access_token: String,
) -> ChangeSetResult<()> {
    let ctx = &ctx;

    let schema_variant_ids = SchemaVariant::list(ctx)
        .await?
        .iter()
        .map(|sv| *sv.id())
        .collect();
    let module_name = "Workspace Backup";
    let module_version = Ulid::new().to_string();
    let module_bytes = dal::pkg::export_pkg_as_bytes(
        ctx,
        module_name,
        &module_version,
        Some("Backup of all schema variants on HEAD."),
        "Sally Signup",
        schema_variant_ids,
    )
    .instrument(debug_span!("Generating workspace backup module"))
    .await?;
    let Some(module_index_url) = ctx.module_index_url() else {
        return Err(PkgError::ModuleIndexNotConfigured.into());
    };
    let index_client =
        module_index_client::IndexClient::new(module_index_url.try_into()?, &access_token);
    let _upload_response = index_client
        .upload_module(module_name, &module_version, module_bytes)
        .instrument(debug_span!("Uploading module"))
        .await?;

    info!("Success");
    Ok(())
}
*/
