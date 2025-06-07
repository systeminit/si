use std::num::ParseIntError;

use axum::{
    Router,
    http::StatusCode,
    response::{
        IntoResponse,
        Response,
    },
    routing::{
        delete,
        get,
        post,
        put,
    },
};
use dal::{
    ChangeSetError,
    ComponentError,
    ComponentId,
    FuncError,
    SchemaError,
    SchemaVariantError,
    TransactionsError,
    WorkspaceSnapshotError,
    WsEventError,
    cached_module::CachedModuleError,
    component::{
        frame::FrameError,
        inferred_connection_graph::InferredConnectionGraphError,
    },
    pkg::PkgError,
    slow_rt::SlowRuntimeError,
    workspace_snapshot::graph::WorkspaceSnapshotGraphError,
};
use sdf_core::api_error::ApiError;
use serde::{
    Deserialize,
    Serialize,
};
use si_id::ViewId;
use thiserror::Error;
use tokio::task::JoinError;

use crate::app_state::AppState;

pub mod convert_to_view;
pub mod create_component;
pub mod create_view;
pub mod create_view_and_move;
mod create_view_object;
mod duplicate_components;
mod erase_components;
mod erase_view_object;
pub mod get_diagram;
pub mod list_views;
mod paste_component;
mod remove_view;
mod set_component_parent;
mod set_geometry;
pub mod update_view;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum ViewError {
    #[error("cached module error: {0}")]
    CachedModule(#[from] CachedModuleError),
    #[error("workspace must have at least one view")]
    CantDeleteOnlyView(),
    #[error("changeset error: {0}")]
    ChangeSet(#[from] ChangeSetError),
    #[error("component error: {0}")]
    Component(#[from] ComponentError),
    #[error("cannot convert component to a view: {0}")]
    ComponentIsNotAFrame(ComponentId),
    #[error("dal diagram error: {0}")]
    DalDiagram(#[from] dal::diagram::DiagramError),
    #[error("frame error: {0}")]
    Frame(#[from] FrameError),
    #[error("func error: {0}")]
    Func(#[from] FuncError),
    #[error("inferred connection graph error: {0}")]
    InferredConnectionGraph(#[from] InferredConnectionGraphError),
    #[error("invalid request: {0}")]
    InvalidRequest(String),
    #[error("join error: {0}")]
    Join(#[from] JoinError),
    #[error("materialized view error: {0}")]
    MaterializedView(#[from] Box<dal_materialized_views::Error>),
    #[error("there is already a view called {0}")]
    NameAlreadyInUse(String),
    #[error(transparent)]
    ParseInt(#[from] ParseIntError),
    #[error("paste error")]
    Paste,
    #[error("pkg error: {0}")]
    Pkg(#[from] PkgError),
    #[error("schema error: {0}")]
    Schema(#[from] SchemaError),
    #[error("schema variant error: {0}")]
    SchemaVariant(#[from] SchemaVariantError),
    #[error("serrde error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("slow runtime error: {0}")]
    SlowRuntime(#[from] SlowRuntimeError),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] WorkspaceSnapshotError),
    #[error("WsEvent error: {0}")]
    WsEvent(#[from] WsEventError),
}

pub type ViewResult<T> = Result<T, ViewError>;

impl IntoResponse for ViewError {
    fn into_response(self) -> Response {
        let (status_code, error_message) = match self {
            ViewError::NameAlreadyInUse(_) => (StatusCode::CONFLICT, self.to_string()),
            ViewError::CantDeleteOnlyView() => (StatusCode::PRECONDITION_FAILED, self.to_string()),
            ViewError::DalDiagram(
                dal::diagram::DiagramError::DeletingLastGeometryForComponent(_, _),
            )
            | ViewError::Component(ComponentError::ComponentAlreadyInView(_, _)) => {
                (StatusCode::FORBIDDEN, self.to_string())
            }
            ViewError::DalDiagram(dal::diagram::DiagramError::ViewNotFound(_)) => {
                (StatusCode::NOT_FOUND, self.to_string())
            }
            ViewError::DalDiagram(dal::diagram::DiagramError::WorkspaceSnapshot(
                WorkspaceSnapshotError::WorkspaceSnapshotGraph(
                    WorkspaceSnapshotGraphError::ViewRemovalWouldOrphanItems(_),
                ),
            )) => (StatusCode::CONFLICT, self.to_string()),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };

        ApiError::new(status_code, error_message).into_response()
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ViewNodeGeometry {
    pub x: String,
    pub y: String,
    pub radius: String,
}

pub fn v2_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_views::list_views))
        .route("/", post(create_view::create_view))
        .route(
            "/create_and_move",
            post(create_view_and_move::create_view_and_move),
        )
        .route("/convert_to_view", post(convert_to_view::convert_to_view))
        .route(
            "/:view_id",
            put(update_view::update_view).delete(remove_view::remove_view),
        )
        .route("/:view_id/get_diagram", get(get_diagram::get_diagram))
        .route("/:view_id/get_geometry", get(get_diagram::get_geometry))
        .route(
            "/default/get_diagram",
            get(get_diagram::get_default_diagram),
        )
        .route(
            "/:view_id/component",
            post(create_component::create_component),
        )
        .route(
            "/:view_id/paste_components",
            post(paste_component::paste_component),
        )
        .route(
            "/:view_id/duplicate_components",
            post(duplicate_components::duplicate_components),
        )
        .route(
            "/:view_id/erase_components",
            delete(erase_components::erase_components),
        )
        .route(
            "/:view_id/component/set_geometry",
            put(set_geometry::set_component_geometry),
        )
        .route(
            "/:view_id/component/set_parent",
            put(set_component_parent::set_component_parent),
        )
        .route(
            "/:view_id/view_object",
            post(create_view_object::create_view_object),
        )
        .route(
            "/:view_id/view_object",
            delete(erase_view_object::erase_view_object),
        )
        .route(
            "/:view_id/view_object/set_geometry",
            put(set_geometry::set_view_object_geometry),
        )
}

#[derive(Debug, serde::Deserialize)]
struct ViewParam {
    view_id: ViewId,
}
