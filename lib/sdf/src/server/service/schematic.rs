use axum::body::{Bytes, Full};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::Json;
use axum::Router;
use dal::{
    node::NodeId, ComponentError, NodeError, NodeKind, NodeMenuError, NodePositionError,
    ReadTenancyError, SchemaError as DalSchemaError, SchematicError as DalSchematicError,
    SchematicKind, StandardModelError, TransactionsError,
};
use std::convert::Infallible;
use thiserror::Error;

pub mod create_connection;
pub mod create_node;
pub mod get_node_add_menu;
pub mod get_node_template;
pub mod get_schematic;
pub mod set_node_position;

#[derive(Debug, Error)]
pub enum SchematicError {
    #[error(transparent)]
    Nats(#[from] si_data::NatsError),
    #[error(transparent)]
    Pg(#[from] si_data::PgError),
    #[error(transparent)]
    StandardModel(#[from] StandardModelError),
    #[error(transparent)]
    ContextTransaction(#[from] TransactionsError),
    #[error("schema error: {0}")]
    Schema(#[from] DalSchemaError),
    #[error("schema not found")]
    SchemaNotFound,
    #[error("schema variant not found")]
    SchemaVariantNotFound,
    #[error("node menu error: {0}")]
    NodeMenu(#[from] NodeMenuError),
    #[error("node error: {0}")]
    Node(#[from] NodeError),
    #[error("invalid request")]
    InvalidRequest,
    #[error("component error: {0}")]
    ComponentError(#[from] ComponentError),
    #[error("node position error: {0}")]
    NodePosition(#[from] NodePositionError),
    #[error("dal schematic error: {0}")]
    SchematicError(#[from] DalSchematicError),
    #[error("read tenancy error: {0}")]
    ReadTenancy(#[from] ReadTenancyError),
    #[error("not authorized")]
    NotAuthorized,
    #[error("invalid system")]
    InvalidSystem,
    #[error("invalid schema kind ({0}) and parent node id pair ({1:?})")]
    InvalidSchematicKindParentNodeIdPair(SchematicKind, Option<NodeId>),
    #[error("parent node not found {0}")]
    ParentNodeNotFound(NodeId),
    #[error("invalid parent node kind {0:?}")]
    InvalidParentNode(NodeKind),
}

pub type SchematicResult<T> = std::result::Result<T, SchematicError>;

impl IntoResponse for SchematicError {
    type Body = Full<Bytes>;
    type BodyError = Infallible;

    fn into_response(self) -> hyper::Response<Self::Body> {
        let (status, error_message) = match self {
            SchematicError::SchemaNotFound => (StatusCode::NOT_FOUND, self.to_string()),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };

        let body = Json(
            serde_json::json!({ "error": { "message": error_message, "code": 42, "statusCode": status.as_u16() } }),
        );

        (status, body).into_response()
    }
}

pub fn routes() -> Router {
    Router::new()
        .route("/get_schematic", get(get_schematic::get_schematic))
        .route(
            "/get_node_add_menu",
            post(get_node_add_menu::get_node_add_menu),
        )
        .route(
            "/get_node_template",
            get(get_node_template::get_node_template),
        )
        .route("/create_node", post(create_node::create_node))
        .route(
            "/set_node_position",
            post(set_node_position::set_node_position),
        )
        .route(
            "/create_connection",
            post(create_connection::create_connection),
        )
}
