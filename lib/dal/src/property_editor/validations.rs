//! This module contains the ability to construct the latest state of validations for a
//! [`Component`](crate::Component)'s properties.

use serde::{Deserialize, Serialize};

use crate::{property_editor::PropertyEditorResult, Component, ComponentId, DalContext};

use super::PropertyEditorPropId;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PropertyEditorValidation {
    prop_id: PropertyEditorPropId,
    valid: bool,
    message: Option<String>,
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
        let validations = Component::list_validations(ctx, component_id)
            .await?
            .iter()
            .map(|validation| PropertyEditorValidation {
                prop_id: validation.prop_id.into(),
                valid: validation.valid,
                message: validation.message.clone(),
            })
            .collect();

        Ok(Self { validations })
    }
}
