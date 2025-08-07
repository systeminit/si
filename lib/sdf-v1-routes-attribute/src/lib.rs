use axum::{
    Router,
    routing::get,
};
use dal::{
    AttributeValueId,
    OutputSocketId,
    PropId,
    TransactionsError,
    attribute::value::AttributeValueError,
    prop::PropError,
    socket::output::OutputSocketError,
};
use sdf_core::{
    app_state::AppState,
    impl_default_error_into_response,
};
use thiserror::Error;

pub mod get_prototype_arguments;

#[remain::sorted]
#[derive(Error, Debug)]
pub enum AttributeError {
    #[error("attribute value error: {0}")]
    AttributeValue(#[from] AttributeValueError),
    #[error("multiple attribute values ({0:?}) found for output socket ({1})")]
    MultipleAttributeValuesForOutputSocket(Vec<AttributeValueId>, OutputSocketId),
    #[error("multiple attribute values ({0:?}) found for prop ({1})")]
    MultipleAttributeValuesForProp(Vec<AttributeValueId>, PropId),
    #[error("multiple output locations provided: prop id ({0}) and output socket id ({1})")]
    MultipleOutputLocationsProvided(PropId, OutputSocketId),
    #[error("no attribute values found for output socket: {0}")]
    NoAttributeValuesFoundForOutputSocket(OutputSocketId),
    #[error("no attribute values found for ou: {0}")]
    NoAttributeValuesFoundForProp(PropId),
    #[error("no output locations provided")]
    NoOutputLocationsProvided,
    #[error("output socket error: {0}")]
    OutputSocket(#[from] OutputSocketError),
    #[error("prop error: {0}")]
    Prop(#[from] PropError),
    #[error("transaction error: {0}")]
    Transactions(#[from] TransactionsError),
}

pub type AttributeResult<T> = Result<T, AttributeError>;

impl_default_error_into_response!(AttributeError);

pub fn routes() -> Router<AppState> {
    Router::new().route(
        "/get_prototype_arguments", // USED IN FUNC EDITOR
        get(get_prototype_arguments::get_prototype_arguments),
    )
}
