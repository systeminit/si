//! This module contains the ability to convert the raw state of a
//! [`Component`](crate::Component)'s properties to friendly objects for displaying, accessing
//! and mutating said properties.

use serde::{
    Deserialize,
    Serialize,
};
use si_data_pg::PgError;
use thiserror::Error;

use crate::{
    AttributeValueId,
    ComponentError,
    PropId,
    SchemaVariantError,
    SchemaVariantId,
    SecretError,
    TransactionsError,
    attribute::{
        prototype::{
            AttributePrototypeError,
            argument::{
                AttributePrototypeArgumentError,
                value_source::ValueSourceError,
            },
        },
        value::AttributeValueError,
    },
    prop::PropError,
    validation::ValidationError,
    workspace_snapshot::{
        WorkspaceSnapshotError,
        node_weight::NodeWeightError,
    },
};

pub mod schema;
pub mod values;

#[remain::sorted]
#[derive(Error, Debug)]
pub enum PropertyEditorError {
    #[error("attribute prototype error: {0}")]
    AttributePrototype(#[from] Box<AttributePrototypeError>),
    #[error("attribute prototype argument error: {0}")]
    AttributePrototypeArgument(#[from] Box<AttributePrototypeArgumentError>),
    #[error("attribute value error: {0}")]
    AttributeValue(#[from] Box<AttributeValueError>),
    #[error("invalid AttributeReadContext: {0}")]
    BadAttributeReadContext(String),
    #[error("component error: {0}")]
    Component(#[from] Box<ComponentError>),
    #[error("component not found")]
    ComponentNotFound,
    #[error("cycle detected: {0}")]
    CycleDetected(AttributeValueId),
    #[error("node weight error: {0}")]
    NodeWeight(#[from] Box<NodeWeightError>),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("prop error: {0}")]
    Prop(#[from] Box<PropError>),
    #[error("property editor value not found by prop id: {0}")]
    PropertyEditorValueNotFoundByPropId(PropId),
    #[error("schema variant error: {0}")]
    SchemaVariant(#[from] Box<SchemaVariantError>),
    #[error("schema variant not found: {0}")]
    SchemaVariantNotFound(SchemaVariantId),
    #[error("secret error: {0}")]
    Secret(#[from] Box<SecretError>),
    #[error("secret prop for {0} leads to static value")]
    SecretPropLeadsToStaticValue(AttributeValueId, AttributeValueId),
    #[error("error serializing/deserializing json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("transactions error: {0}")]
    Transactions(#[from] Box<TransactionsError>),
    #[error("could not acquire lock: {0}")]
    TryLock(#[from] tokio::sync::TryLockError),
    #[error("validation error: {0}")]
    Validation(#[from] Box<ValidationError>),
    #[error("value source error: {0}")]
    ValueSource(#[from] Box<ValueSourceError>),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] Box<WorkspaceSnapshotError>),
}

pub type PropertyEditorResult<T> = Result<T, PropertyEditorError>;

// Property editor ids used across submodules.
pub use si_id::{
    PropertyEditorPropId,
    PropertyEditorValueId,
};

// TODO(nick): once shape is finalized and we stop serializing this within builtins, please
// convert to a more formal type.
#[derive(Deserialize, Serialize, Debug)]
pub struct SelectWidgetOption {
    pub(crate) label: String,
    pub(crate) value: String,
}

impl From<AttributePrototypeError> for PropertyEditorError {
    fn from(value: AttributePrototypeError) -> Self {
        Box::new(value).into()
    }
}

impl From<AttributePrototypeArgumentError> for PropertyEditorError {
    fn from(value: AttributePrototypeArgumentError) -> Self {
        Box::new(value).into()
    }
}

impl From<AttributeValueError> for PropertyEditorError {
    fn from(value: AttributeValueError) -> Self {
        Box::new(value).into()
    }
}

impl From<ComponentError> for PropertyEditorError {
    fn from(value: ComponentError) -> Self {
        Box::new(value).into()
    }
}

impl From<NodeWeightError> for PropertyEditorError {
    fn from(value: NodeWeightError) -> Self {
        Box::new(value).into()
    }
}

impl From<PropError> for PropertyEditorError {
    fn from(value: PropError) -> Self {
        Box::new(value).into()
    }
}

impl From<SchemaVariantError> for PropertyEditorError {
    fn from(value: SchemaVariantError) -> Self {
        Box::new(value).into()
    }
}

impl From<SecretError> for PropertyEditorError {
    fn from(value: SecretError) -> Self {
        Box::new(value).into()
    }
}

impl From<TransactionsError> for PropertyEditorError {
    fn from(value: TransactionsError) -> Self {
        Box::new(value).into()
    }
}

impl From<ValidationError> for PropertyEditorError {
    fn from(value: ValidationError) -> Self {
        Box::new(value).into()
    }
}

impl From<ValueSourceError> for PropertyEditorError {
    fn from(value: ValueSourceError) -> Self {
        Box::new(value).into()
    }
}

impl From<WorkspaceSnapshotError> for PropertyEditorError {
    fn from(value: WorkspaceSnapshotError) -> Self {
        Box::new(value).into()
    }
}
