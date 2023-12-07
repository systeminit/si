use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::Json;
use axum::Router;
use dal::workspace_snapshot::WorkspaceSnapshotError;
use dal::WsEventError;
use dal::{
    node::NodeId, ChangeSetError, NodeKind, SchemaVariantId, StandardModelError, TransactionsError,
};
use thiserror::Error;

use crate::server::state::AppState;

pub mod get_diagram;
pub mod list_schema_variants;

// mod connect_component_to_frame;
// pub mod create_connection;
// pub mod create_node;
// pub mod delete_component;
// pub mod delete_connection;
// mod restore_component;
// pub mod restore_connection;
// pub mod set_node_position;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum DiagramError {
    #[error("changeset error: {0}")]
    ChangeSet(#[from] ChangeSetError),
    #[error("change set not found")]
    ChangeSetNotFound,
    #[error("component not found")]
    ComponentNotFound,
    #[error(transparent)]
    ContextTransaction(#[from] TransactionsError),
    #[error("dal schema error: {0}")]
    DalSchema(#[from] dal::SchemaError),
    #[error("dal schema variant error: {0}")]
    DalSchemaVariant(#[from] dal::schema::variant::SchemaVariantError),
    #[error("edge not found")]
    EdgeNotFound,
    #[error("frame internal provider not found for schema variant id: {0}")]
    FrameInternalProviderNotFoundForSchemaVariant(SchemaVariantId),
    #[error("frame socket not found for schema variant id: {0}")]
    FrameSocketNotFound(SchemaVariantId),
    #[error("invalid header name {0}")]
    Hyper(#[from] hyper::http::Error),
    #[error("invalid parent node kind {0:?}")]
    InvalidParentNode(NodeKind),
    #[error("invalid request")]
    InvalidRequest,
    #[error("invalid system")]
    InvalidSystem,
    #[error(transparent)]
    Nats(#[from] si_data_nats::NatsError),
    #[error("node not found: {0}")]
    NodeNotFound(NodeId),
    #[error("not authorized")]
    NotAuthorized,
    #[error("parent node not found {0}")]
    ParentNodeNotFound(NodeId),
    #[error(transparent)]
    Pg(#[from] si_data_pg::PgError),
    #[error(transparent)]
    PgPool(#[from] si_data_pg::PgPoolError),
    #[error("schema not found")]
    SchemaNotFound,
    #[error("schema variant not found")]
    SchemaVariantNotFound,
    #[error("serde error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("socket not found")]
    SocketNotFound,
    #[error(transparent)]
    StandardModel(#[from] StandardModelError),
    #[error(transparent)]
    WorkspaceSnaphot(#[from] WorkspaceSnapshotError),
    #[error("ws event error: {0}")]
    WsEvent(#[from] WsEventError),
}

pub type DiagramResult<T> = Result<T, DiagramError>;

impl IntoResponse for DiagramError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            DiagramError::SchemaNotFound => (StatusCode::NOT_FOUND, self.to_string()),
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
        // .route("/create_node", post(create_node::create_node))
        // .route(
        //     "/set_node_position",
        //     post(set_node_position::set_node_position),
        // )
        // .route(
        //     "/create_connection",
        //     post(create_connection::create_connection),
        // )
        // .route(
        //     "/delete_connection",
        //     post(delete_connection::delete_connection),
        // )
        // .route(
        //     "/restore_connection",
        //     post(restore_connection::restore_connection),
        // )
        // .route(
        //     "/delete_component",
        //     post(delete_component::delete_component),
        // )
        // .route(
        //     "/delete_components",
        //     post(delete_component::delete_components),
        // )
        // .route(
        //     "/restore_component",
        //     post(restore_component::restore_component),
        // )
        // .route(
        //     "/restore_components",
        //     post(restore_component::restore_components),
        // )
        // .route(
        //     "/connect_component_to_frame",
        //     post(connect_component_to_frame::connect_component_to_frame),
        // )
        .route("/get_diagram", get(get_diagram::get_diagram))
        .route(
            "/list_schema_variants",
            get(list_schema_variants::list_schema_variants),
        )
}
