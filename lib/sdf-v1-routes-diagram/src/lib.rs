use std::num::{
    ParseFloatError,
    ParseIntError,
};

use axum::{
    Router,
    http::StatusCode,
    response::{
        IntoResponse,
        Response,
    },
    routing::{
        get,
        post,
    },
};
use dal::{
    ChangeSetError,
    FuncError,
    SchemaError,
    SchemaId,
    SchemaVariantId,
    TransactionsError,
    WsEventError,
    attribute::{
        prototype::{
            AttributePrototypeError,
            argument::AttributePrototypeArgumentError,
        },
        value::AttributeValueError,
    },
    cached_module::CachedModuleError,
    component::{
        ComponentError,
        inferred_connection_graph::InferredConnectionGraphError,
    },
    pkg::PkgError,
    slow_rt::SlowRuntimeError,
    socket::{
        input::InputSocketError,
        output::OutputSocketError,
    },
    workspace_snapshot::{
        WorkspaceSnapshotError,
        dependent_value_root::DependentValueRootError,
    },
};
use sdf_core::{
    api_error::ApiError,
    app_state::AppState,
};
use telemetry::prelude::*;
use thiserror::Error;
use tokio::task::JoinError;

mod add_components_to_view;
pub mod create_component;
pub mod create_connection;
pub mod delete_component;
pub mod delete_connection;
pub mod dvu_roots;
pub mod get_all_components_and_edges;
pub mod get_diagram;
pub mod remove_delete_intent;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum DiagramError {
    #[error("attribute prototype error: {0}")]
    AttributePrototype(#[from] AttributePrototypeError),
    #[error("attribute prototype argument error: {0}")]
    AttributePrototypeArgument(#[from] AttributePrototypeArgumentError),
    #[error("attribute value error: {0}")]
    AttributeValue(#[from] AttributeValueError),
    #[error("cached module error: {0}")]
    CachedModule(#[from] CachedModuleError),
    #[error("changeset error: {0}")]
    ChangeSet(#[from] ChangeSetError),
    #[error("component error: {0}")]
    Component(#[from] ComponentError),
    #[error("component not found")]
    ComponentNotFound,
    #[error(transparent)]
    ContextTransaction(#[from] TransactionsError),
    #[error("dal diagram error: {0}")]
    DalDiagram(#[from] dal::diagram::DiagramError),
    #[error("dal frame error: {0}")]
    DalFrame(#[from] dal::component::frame::FrameError),
    #[error("dal schema variant error: {0}")]
    DalSchemaVariant(#[from] dal::schema::variant::SchemaVariantError),
    #[error("dependent value root error: {0}")]
    DependentValueRoot(#[from] DependentValueRootError),
    #[error("duplicated connection")]
    DuplicatedConnection,
    #[error("edge not found")]
    EdgeNotFound,
    #[error("frame socket not found for schema variant id: {0}")]
    FrameSocketNotFound(SchemaVariantId),
    #[error("func error: {0}")]
    Func(#[from] FuncError),
    #[error("invalid header name {0}")]
    Hyper(#[from] hyper::http::Error),
    #[error("InferredConnectionGraph error: {0}")]
    InferredConnectionGraph(#[from] InferredConnectionGraphError),
    #[error("input socket error: {0}")]
    InputSocket(#[from] InputSocketError),
    #[error("invalid request: {0}")]
    InvalidRequest(String),
    #[error("tokio join error: {0}")]
    Join(#[from] JoinError),
    #[error(transparent)]
    Nats(#[from] si_data_nats::NatsError),
    #[error("no default schema variant found for schema id {0}")]
    NoDefaultSchemaVariant(SchemaId),
    #[error("not authorized")]
    NotAuthorized,
    #[error("output socket error: {0}")]
    OutputSocket(#[from] OutputSocketError),
    #[error("parse float error: {0}")]
    ParseFloat(#[from] ParseFloatError),
    #[error("parse int error: {0}")]
    ParseInt(#[from] ParseIntError),
    #[error("paste failed")]
    Paste,
    #[error(transparent)]
    Pg(#[from] si_data_pg::PgError),
    #[error(transparent)]
    PgPool(#[from] si_data_pg::PgPoolError),
    #[error("pkg error: {0}")]
    Pkg(#[from] PkgError),
    #[error("schema error: {0}")]
    Schema(#[from] SchemaError),
    #[error("schema not found")]
    SchemaNotFound,
    #[error("serde error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("slow runtime error: {0}")]
    SlowRuntime(#[from] SlowRuntimeError),
    #[error("socket not found")]
    SocketNotFound,
    #[error("No installable module found for schema id {0}")]
    UninstalledSchemaNotFound(SchemaId),
    #[error(transparent)]
    WorkspaceSnasphot(#[from] WorkspaceSnapshotError),
    #[error("ws event error: {0}")]
    WsEvent(#[from] WsEventError),
}

pub type DiagramResult<T> = Result<T, DiagramError>;

impl IntoResponse for DiagramError {
    fn into_response(self) -> Response {
        let error_message = self.to_string();

        let status_code = match self {
            DiagramError::SchemaNotFound
            | DiagramError::ComponentNotFound
            | DiagramError::FrameSocketNotFound(_)
            | DiagramError::EdgeNotFound
            | DiagramError::SocketNotFound => StatusCode::NOT_FOUND,
            DiagramError::Component(ComponentError::ComponentAlreadyInView(_, _)) => {
                StatusCode::UNPROCESSABLE_ENTITY
            }
            DiagramError::Component(ComponentError::Diagram(e)) => match *e {
                dal::diagram::DiagramError::DeletingLastGeometryForComponent(_, _) => {
                    StatusCode::UNPROCESSABLE_ENTITY
                }
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            },

            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };

        ApiError::new(status_code, error_message).into_response()
    }
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/add_components_to_view", // USED IN OLD UI
            post(add_components_to_view::add_components_to_view),
        )
        .route(
            "/delete_connection", // USED IN OLD UI
            post(delete_connection::delete_connection),
        )
        .route(
            "/delete_components", // USED IN OLD UI
            post(delete_component::delete_components),
        )
        .route(
            "/remove_delete_intent", // USED IN OLD UI
            post(remove_delete_intent::remove_delete_intent),
        )
        .route(
            "/create_connection", // USED IN OLD UI
            post(create_connection::create_connection),
        )
        .route(
            "/create_component", // FIXME(nick): replace API tests that used this (this affects practically all of them)
            post(create_component::create_component),
        )
        .route("/get_diagram", get(get_diagram::get_diagram)) // FIXME(nick): replace API tests that used this (this affects practically all of them)
        .route(
            "/get_all_components_and_edges", // USED IN OLD UI
            get(get_all_components_and_edges::get_all_components_and_edges),
        )
        .route("/dvu_roots", get(dvu_roots::dvu_roots)) // USED IN OLD UI
}
