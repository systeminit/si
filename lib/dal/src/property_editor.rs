//! This module contains the ability to convert the raw state of a
//! [`Component`](crate::Component)'s properties to friendly objects for displaying, accessing
//! and mutating said properties.

use serde::{Deserialize, Serialize};
use si_data_pg::PgError;
use thiserror::Error;

use crate::{
    pk, schema::variant::SchemaVariantError, AttributeValueError, AttributeValueId, ComponentError,
    PropId, SchemaVariantId, StandardModelError, ValidationResolverError,
};

pub mod schema;
pub mod validations;
pub mod values;

#[derive(Error, Debug)]
pub enum PropertyEditorError {
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("standard model error: {0}")]
    StandardModel(#[from] StandardModelError),
    #[error("schema variant not found: {0}")]
    SchemaVariantNotFound(SchemaVariantId),
    #[error("root prop not found for schema variant")]
    RootPropNotFound,
    #[error("schema variant: {0}")]
    SchemaVariant(#[from] SchemaVariantError),
    #[error("error serializing/deserializing json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("attribute value error: {0}")]
    AttributeValue(#[from] AttributeValueError),
    #[error("invalid AttributeReadContext: {0}")]
    BadAttributeReadContext(String),
    #[error("component not found")]
    ComponentNotFound,
    #[error("component error: {0}")]
    Component(#[from] ComponentError),
    #[error("validation resolver error: {0}")]
    ValidationResolver(#[from] ValidationResolverError),
    #[error("prop not found for id: {0}")]
    PropNotFound(PropId),
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
