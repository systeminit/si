use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::Json;
use axum::Router;
use dal::provider::external::ExternalProviderError as DalExternalProviderError;
use dal::socket::{SocketError, SocketId};
use dal::{
    node::NodeId, schema::variant::SchemaVariantError, ActionError, ActionPrototypeError,
    AttributeValueError, ChangeSetError, ComponentError, ComponentType,
    DiagramError as DalDiagramError, EdgeError, InternalProviderError, NodeError, NodeKind,
    NodeMenuError, SchemaError as DalSchemaError, SchemaVariantId, StandardModelError,
    TransactionsError,
};
use dal::{AttributeReadContext, WsEventError};
use thiserror::Error;

use crate::server::state::AppState;
use crate::service::schema::SchemaError;

mod connect_component_to_frame;
pub mod create_connection;
pub mod create_node;
pub mod delete_component;
pub mod delete_connection;
pub mod get_diagram;
pub mod get_node_add_menu;
pub mod list_schema_variants;
mod restore_component;
pub mod restore_connection;
pub mod set_node_position;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum DiagramError {
    #[error("action error: {0}")]
    ActionError(#[from] ActionError),
    #[error("action prototype error: {0}")]
    ActionPrototype(#[from] ActionPrototypeError),
    #[error("attribute value error: {0}")]
    AttributeValue(#[from] AttributeValueError),
    #[error("attribute value not found for context: {0:?}")]
    AttributeValueNotFoundForContext(AttributeReadContext),
    #[error("changeset error: {0}")]
    ChangeSet(#[from] ChangeSetError),
    #[error("change set not found")]
    ChangeSetNotFound,
    #[error("component error: {0}")]
    Component(#[from] ComponentError),
    #[error("component not found")]
    ComponentNotFound,
    #[error(transparent)]
    ContextTransaction(#[from] TransactionsError),
    #[error("dal schema error: {0}")]
    DalSchema(#[from] DalSchemaError),
    #[error("dal diagram error: {0}")]
    DiagramError(#[from] DalDiagramError),
    #[error(transparent)]
    Edge(#[from] EdgeError),
    #[error("edge not found")]
    EdgeNotFound,
    #[error("external provider error: {0}")]
    ExternalProvider(#[from] DalExternalProviderError),
    #[error("external provider not found for socket id: {0}")]
    ExternalProviderNotFoundForSocket(SocketId),
    #[error("frame internal provider not found for schema variant id: {0}")]
    FrameInternalProviderNotFoundForSchemaVariant(SchemaVariantId),
    #[error("frame socket not found for schema variant id: {0}")]
    FrameSocketNotFound(SchemaVariantId),
    #[error("invalid header name {0}")]
    Hyper(#[from] hyper::http::Error),
    #[error(transparent)]
    InternalProvider(#[from] InternalProviderError),
    #[error("internal provider not found for socket id: {0}")]
    InternalProviderNotFoundForSocket(SocketId),
    #[error("invalid component type ({0:?}) for frame")]
    InvalidComponentTypeForFrame(ComponentType),
    #[error("invalid parent node kind {0:?}")]
    InvalidParentNode(NodeKind),
    #[error("invalid request")]
    InvalidRequest,
    #[error("invalid system")]
    InvalidSystem,
    #[error(transparent)]
    Nats(#[from] si_data_nats::NatsError),
    #[error("node error: {0}")]
    Node(#[from] NodeError),
    #[error("node menu error: {0}")]
    NodeMenu(#[from] NodeMenuError),
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
    #[error("schema error: {0}")]
    Schema(#[from] SchemaError),
    #[error("schema not found")]
    SchemaNotFound,
    #[error("schema variant error: {0}")]
    SchemaVariant(#[from] SchemaVariantError),
    #[error("schema variant not found")]
    SchemaVariantNotFound,
    #[error("serde error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("socket error: {0}")]
    Socket(#[from] SocketError),
    #[error("socket not found")]
    SocketNotFound,
    #[error(transparent)]
    StandardModel(#[from] StandardModelError),
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
        .route("/get_diagram", get(get_diagram::get_diagram))
        .route(
            "/get_node_add_menu",
            post(get_node_add_menu::get_node_add_menu),
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
        .route(
            "/delete_connection",
            post(delete_connection::delete_connection),
        )
        .route(
            "/restore_connection",
            post(restore_connection::restore_connection),
        )
        .route(
            "/delete_component",
            post(delete_component::delete_component),
        )
        .route(
            "/delete_components",
            post(delete_component::delete_components),
        )
        .route(
            "/restore_component",
            post(restore_component::restore_component),
        )
        .route(
            "/restore_components",
            post(restore_component::restore_components),
        )
        .route(
            "/connect_component_to_frame",
            post(connect_component_to_frame::connect_component_to_frame),
        )
        .route(
            "/list_schema_variants",
            get(list_schema_variants::list_schema_variants),
        )
}
