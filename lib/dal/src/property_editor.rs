//! This module contains the ability to convert the raw state of a
//! [`Component`](crate::Component)'s properties to friendly objects for displaying, accessing
//! and mutating said properties.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use si_data_pg::PgError;
use thiserror::Error;

use crate::{
    attribute::{
        prototype::{
            argument::{value_source::ValueSourceError, AttributePrototypeArgumentError},
            AttributePrototypeError,
        },
        value::AttributeValueError,
    },
    prop::PropError,
    validation::ValidationError,
    workspace_snapshot::{node_weight::NodeWeightError, WorkspaceSnapshotError},
    AttributeValueId, ComponentError, PropId, SchemaVariantError, SchemaVariantId, SecretError,
    StandardModelError, TransactionsError,
};

pub mod schema;
pub mod values;

#[remain::sorted]
#[derive(Error, Debug)]
pub enum PropertyEditorError {
    #[error("attribute prototype error: {0}")]
    AttributePrototype(#[from] AttributePrototypeError),
    #[error("attribute prototype argument error: {0}")]
    AttributePrototypeArgument(#[from] AttributePrototypeArgumentError),
    #[error("attribute value error: {0}")]
    AttributeValue(#[from] AttributeValueError),
    #[error("invalid AttributeReadContext: {0}")]
    BadAttributeReadContext(String),
    #[error("component error: {0}")]
    Component(#[from] ComponentError),
    #[error("component not found")]
    ComponentNotFound,
    #[error("cycle detected: {0}")]
    CycleDetected(AttributeValueId),
    #[error("node weight error: {0}")]
    NodeWeight(#[from] NodeWeightError),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("prop error: {0}")]
    Prop(#[from] PropError),
    #[error("property editor value not found by prop id: {0}")]
    PropertyEditorValueNotFoundByPropId(PropId),
    #[error("schema variant error: {0}")]
    SchemaVariant(#[from] SchemaVariantError),
    #[error("schema variant not found: {0}")]
    SchemaVariantNotFound(SchemaVariantId),
    #[error("secret error: {0}")]
    Secret(#[from] SecretError),
    #[error("secret prop for {0} leads to static value")]
    SecretPropLeadsToStaticValue(AttributeValueId, AttributeValueId),
    #[error("error serializing/deserializing json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("standard model error: {0}")]
    StandardModel(#[from] StandardModelError),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error("could not acquire lock: {0}")]
    TryLock(#[from] tokio::sync::TryLockError),
    #[error("validation error: {0}")]
    Validation(#[from] ValidationError),
    #[error("value source error: {0}")]
    ValueSource(#[from] ValueSourceError),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] WorkspaceSnapshotError),
}

pub type PropertyEditorResult<T> = Result<T>;

// Property editor ids used across submodules.
pub use si_id::PropertyEditorPropId;
pub use si_id::PropertyEditorValueId;

// TODO(nick): once shape is finalized and we stop serializing this within builtins, please
// convert to a more formal type.
#[derive(Deserialize, Serialize, Debug)]
pub struct SelectWidgetOption {
    pub(crate) label: String,
    pub(crate) value: String,
}
