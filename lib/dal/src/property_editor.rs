//! This module contains the ability to convert the raw state of a
//! [`Component`](crate::Component)'s properties to friendly objects for displaying, accessing
//! and mutating said properties.

use serde::{Deserialize, Serialize};
use si_data_pg::PgError;
use thiserror::Error;

use crate::attribute::value::AttributeValueError;
use crate::prop::PropError;
use crate::validation::ValidationError;
use crate::workspace_snapshot::node_weight::NodeWeightError;
use crate::workspace_snapshot::WorkspaceSnapshotError;
use crate::{
    pk, AttributeValueId, ComponentError, PropId, SchemaVariantId, StandardModelError,
    TransactionsError,
};

pub mod schema;
pub mod values;

#[remain::sorted]
#[derive(Error, Debug)]
pub enum PropertyEditorError {
    #[error("attribute value error: {0}")]
    AttributeValue(#[from] AttributeValueError),
    #[error("invalid AttributeReadContext: {0}")]
    BadAttributeReadContext(String),
    #[error("component error: {0}")]
    Component(#[from] ComponentError),
    #[error("component not found")]
    ComponentNotFound,
    #[error("node weight error: {0}")]
    NodeWeight(#[from] NodeWeightError),
    #[error("no value(s) found for property editor prop id: {0}")]
    NoValuesFoundForPropertyEditorProp(PropertyEditorPropId),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("prop error: {0}")]
    Prop(#[from] PropError),
    #[error("schema variant not found: {0}")]
    SchemaVariantNotFound(SchemaVariantId),
    #[error("error serializing/deserializing json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("standard model error: {0}")]
    StandardModel(#[from] StandardModelError),
    #[error("too many values found (likely not the prop for an element of a map or an array) for property editor prop id: {0}")]
    TooManyValuesFoundForPropertyEditorProp(PropertyEditorPropId),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error("could not acquire lock: {0}")]
    TryLock(#[from] tokio::sync::TryLockError),
    #[error("validation error: {0}")]
    Validation(#[from] ValidationError),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] WorkspaceSnapshotError),
}

pub type PropertyEditorResult<T> = Result<T, PropertyEditorError>;

// Property editor ids used across submodules.
pk!(PropertyEditorValueId);
pk!(PropertyEditorPropId);

impl From<AttributeValueId> for PropertyEditorValueId {
    fn from(id: AttributeValueId) -> Self {
        Self::from(ulid::Ulid::from(id))
    }
}

impl From<PropertyEditorValueId> for AttributeValueId {
    fn from(id: PropertyEditorValueId) -> Self {
        Self::from(ulid::Ulid::from(id))
    }
}

impl From<PropId> for PropertyEditorPropId {
    fn from(prop_id: PropId) -> Self {
        Self::from(ulid::Ulid::from(prop_id))
    }
}

impl From<PropertyEditorPropId> for PropId {
    fn from(property_editor_prop_id: PropertyEditorPropId) -> Self {
        Self::from(ulid::Ulid::from(property_editor_prop_id))
    }
}

// TODO(nick): once shape is finalized and we stop serializing this within builtins, please
// convert to a more formal type.
#[derive(Deserialize, Serialize, Debug)]
pub struct SelectWidgetOption {
    pub(crate) label: String,
    pub(crate) value: String,
}
