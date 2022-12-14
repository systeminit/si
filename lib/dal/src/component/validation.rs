use crate::{
    attribute::value::AttributeValue, attribute::value::AttributeValueError,
    component::ComponentResult, schema::variant::leaves::LeafKind, AttributeReadContext, Component,
    ComponentError, ComponentId, DalContext, Prop, PropError, PropId, SchemaVariant, StandardModel,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use telemetry::prelude::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PropValidation {
    pub prop_id: PropId,
    pub valid: bool,
    pub message: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ValidationEntry {
    pub valid: bool,
    pub message: Option<String>,
}

impl Component {
    #[instrument(skip_all)]
    pub async fn list_validations(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> ComponentResult<Vec<PropValidation>> {
        let component = Self::get_by_id(ctx, &component_id)
            .await?
            .ok_or(ComponentError::NotFound(component_id))?;
        let schema_variant = component
            .schema_variant(ctx)
            .await?
            .ok_or(ComponentError::NoSchemaVariant(component_id))?;

        let validation_map_internal_provider = SchemaVariant::find_leaf_implicit_internal_provider(
            ctx,
            *schema_variant.id(),
            LeafKind::Validation,
        )
        .await?;

        let validation_map_read_context = AttributeReadContext {
            internal_provider_id: Some(*validation_map_internal_provider.id()),
            component_id: Some(component_id),
            ..AttributeReadContext::default()
        };

        let validation_map_av = AttributeValue::find_for_context(ctx, validation_map_read_context)
            .await?
            .ok_or(AttributeValueError::NotFoundForReadContext(
                validation_map_read_context,
            ))?;

        let maybe_validation_map_value = validation_map_av.get_value(ctx).await?;

        let mut prop_validations = vec![];
        if let Some(value) = maybe_validation_map_value {
            let validations_map: HashMap<String, ValidationEntry> = serde_json::from_value(value)?;

            for (key, validation) in validations_map {
                let prop_id = match key.split_once(';') {
                    Some((prop_id, _)) => prop_id.to_string(),
                    None => Err(ComponentError::ValidationKeyMissingPropId(key))?,
                };

                let prop_id: PropId = prop_id.parse()?;
                let _ = Prop::get_by_id(ctx, &prop_id)
                    .await?
                    .ok_or_else(|| PropError::NotFound(prop_id, ctx.visibility().to_owned()))?;

                let validation = PropValidation {
                    prop_id,
                    valid: validation.valid,
                    message: validation.message,
                };

                prop_validations.push(validation);
            }
        };

        Ok(prop_validations)
    }
}
