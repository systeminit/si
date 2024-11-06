use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post, put},
    Router,
};
use axum_util::{app_state::AppState, service::ApiError};
use dal::{
    diagram::view::{View, ViewId},
    ChangeSetError, DalContext, Timestamp, TransactionsError,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub mod create_view;
pub mod list_views;
pub mod update_view;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum ViewError {
    #[error("changeset error: {0}")]
    ChangeSet(#[from] ChangeSetError),
    #[error("dal diagram error: {0}")]
    DalDiagram(#[from] dal::diagram::DiagramError),
    #[error("there is already a view called {0}")]
    NameAlreadyInUse(String),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
}

pub type ViewResult<T> = Result<T, ViewError>;

impl IntoResponse for ViewError {
    fn into_response(self) -> Response {
        let (status_code, error_message) = match self {
            ViewError::NameAlreadyInUse(_) => (StatusCode::UNPROCESSABLE_ENTITY, self.to_string()),

            _ => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };

        ApiError::new(status_code, error_message).into_response()
    }
}

/// Frontend representation for a [View](View).
/// Yeah, it's a silly name, but all the other frontend representation structs are *View,
/// so we either keep it or change everything.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct ViewView {
    id: ViewId,
    name: String,
    is_default: bool,
    #[serde(flatten)]
    timestamp: Timestamp,
}

impl ViewView {
    pub async fn from_view(ctx: &DalContext, view: View) -> ViewResult<Self> {
        Ok(ViewView {
            id: view.id(),
            name: view.name().to_owned(),
            is_default: view.is_default(ctx).await?,
            timestamp: view.timestamp().to_owned(),
        })
    }
}

pub fn v2_routes() -> Router<AppState> {
    Router::new()
        // Func Stuff
        .route("/", get(list_views::list_views))
        .route("/", post(create_view::create_view))
        .route("/:view_id", put(update_view::update_view))
}
