//! This module contains the ability to construct the latest state of validations for a
//! [`Component`](crate::Component)'s properties.

use serde::{Deserialize, Serialize};

use crate::property_editor::{PropertyEditorResult, PropertyEditorValueId};
use crate::{ComponentId, DalContext, ValidationResolver};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PropertyEditorValidationError {
    message: String,
    level: Option<String>,
    kind: Option<String>,
    link: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PropertyEditorValidation {
    value_id: PropertyEditorValueId,
    valid: bool,
    errors: Vec<PropertyEditorValidationError>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PropertyEditorValidations {
    validations: Vec<PropertyEditorValidation>,
}

impl PropertyEditorValidations {
    pub async fn for_component(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> PropertyEditorResult<Self> {
        let status = ValidationResolver::find_status(ctx, component_id).await?;

        let mut validations = Vec::new();
        for stat in status {
            validations.push(PropertyEditorValidation {
                value_id: stat.attribute_value_id.into(),
                valid: stat.errors.is_empty(),
                errors: stat
                    .errors
                    .into_iter()
                    .map(|err| PropertyEditorValidationError {
                        message: err.message,
                        level: err.level,
                        kind: Some(err.kind.as_str().to_string()),
                        link: err.link,
                    })
                    .collect(),
            });
        }
        Ok(Self { validations })
    }
}
