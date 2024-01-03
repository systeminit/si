use serde::Deserialize;
use std::collections::HashMap;
use telemetry::prelude::*;

use crate::attribute::value::AttributeValue;
use crate::attribute::value::AttributeValueError;
use crate::component::ComponentResult;
use crate::qualification::{
    QualificationResult, QualificationSubCheck, QualificationSubCheckStatus, QualificationView,
};
use crate::schema::SchemaVariant;
use crate::validation::ValidationError;
use crate::ws_event::WsEvent;
use crate::{AttributeReadContext, DalContext, RootPropChild, StandardModel, ValidationResolver};
use crate::{Component, ComponentError, ComponentId, Prop};

// FIXME(nick): use the formal types from the new version of function authoring instead of this
// struct. This struct is a temporary stopgap until that's implemented.
#[derive(Deserialize, Debug)]
pub struct QualificationEntry {
    pub result: Option<QualificationSubCheckStatus>,
    pub message: Option<String>,
}

impl Component {
    // TODO(nick): big query potential here.
    #[instrument(skip_all)]
    pub async fn list_qualifications(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> ComponentResult<Vec<QualificationView>> {
        let component = Self::get_by_id(ctx, &component_id)
            .await?
            .ok_or(ComponentError::NotFound(component_id))?;
        let schema_variant = component
            .schema_variant(ctx)
            .await?
            .ok_or(ComponentError::NoSchemaVariant(component_id))?;

        // First, check the ephemeral and universal "All Fields Valid" qualification.
        let all_fields_valid_qualification_view =
            Self::all_fields_valid_qualification(ctx, component_id).await?;
        let mut results: Vec<QualificationView> = vec![all_fields_valid_qualification_view];
        let mut qualification_views = vec![];

        // Prepare to assemble qualification views and access the "/root/qualification" prop tree.
        // We will use its implicit internal provider id and its corresponding prop id to do so.
        let qualification_map_implicit_internal_provider =
            SchemaVariant::find_root_child_implicit_internal_provider(
                ctx,
                *schema_variant.id(),
                RootPropChild::Qualification,
            )
            .await?;

        // Collect all the func binding return value ids for the child attribute values
        // (map entries) for reference later.
        let prop_qualification_map_attribute_read_context = AttributeReadContext {
            prop_id: Some(*qualification_map_implicit_internal_provider.prop_id()),
            component_id: Some(component_id),
            ..AttributeReadContext::default()
        };
        let prop_qualification_map_attribute_value =
            AttributeValue::find_for_context(ctx, prop_qualification_map_attribute_read_context)
                .await?
                .ok_or(AttributeValueError::NotFoundForReadContext(
                    prop_qualification_map_attribute_read_context,
                ))?;

        let mut entries = HashMap::new();
        for entry_attribute_value in prop_qualification_map_attribute_value
            .child_attribute_values(ctx)
            .await?
        {
            let entry_attribute_value_id = *entry_attribute_value.id();
            let func_binding_return_value_id = entry_attribute_value.func_binding_return_value_id();
            let entry_prototype_func_id = entry_attribute_value
                .attribute_prototype(ctx)
                .await?
                .ok_or(ComponentError::MissingAttributePrototype(
                    entry_attribute_value_id,
                ))?
                .func_id();

            let entry: QualificationEntry = serde_json::from_value(
                entry_attribute_value
                    .get_unprocessed_value(ctx)
                    .await?
                    .ok_or(ComponentError::QualificationResultEmpty(
                        entry_attribute_value
                            .key
                            .clone()
                            .unwrap_or("unknown".to_string()),
                        *component.id(),
                    ))?,
            )?;

            let key =
                entry_attribute_value
                    .key()
                    .ok_or(ComponentError::FoundMapEntryWithoutKey(
                        entry_attribute_value_id,
                    ))?;

            // We're going to get values at both contexts (component and schema variant), but we
            // should prefer component level ones
            if entries.contains_key(key) && entry_attribute_value.context.is_component_unset() {
                continue;
            }

            entries.insert(
                key.to_string(),
                (entry, entry_prototype_func_id, func_binding_return_value_id),
            );
        }

        for (key, (entry, entry_prototype_func_id, func_binding_return_value_id)) in entries.drain()
        {
            if let Some(qual_view) = QualificationView::new(
                ctx,
                &key,
                entry,
                entry_prototype_func_id,
                func_binding_return_value_id,
            )
            .await?
            {
                qualification_views.push(qual_view);
            }
        }

        qualification_views.sort();
        // We want the "all fields valid" to always be first
        results.extend(qualification_views);

        WsEvent::checked_qualifications(ctx, component_id)
            .await?
            .publish_on_commit(ctx)
            .await?;

        Ok(results)
    }

    /// An ephemeral qualification (not present in the
    /// [`prop tree`](crate::schema::variant::leaves)) that qualifies if all validations passed.
    #[instrument(skip_all)]
    pub async fn all_fields_valid_qualification(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> ComponentResult<QualificationView> {
        // TODO(nick): this function is a partial port of the original "all fields valid" logic.
        // Use the validation prop tree once available.
        let statuses = ValidationResolver::find_status(ctx, component_id).await?;

        let component = Component::get_by_id(ctx, &component_id)
            .await?
            .ok_or(ComponentError::NotFound(component_id))?;
        let schema_variant = component
            .schema_variant(ctx)
            .await?
            .ok_or(ComponentError::NoSchemaVariant(component_id))?;

        let props = Prop::find_by_attr(ctx, "schema_variant_id", schema_variant.id()).await?;

        let mut validation_errors = Vec::<(String, ValidationError)>::new();
        for status in statuses {
            // FIXME(nick): there's a race condition or bug where the attribute value where
            // "get_by_id" will fail with a given attribute value id from the the "FIND_STATUS"
            // query. This seems to only happen when the "get_summary" route is called from "sdf",
            // but it could happen more frequently. The good news is that this is intermittent
            // and the route is always called again and returns 200 OK. For now, let's log
            // where and when this happens.
            let av = AttributeValue::get_by_id(ctx, &status.attribute_value_id).await?;
            match av {
                Some(av) => {
                    let prop = if let Some(prop) =
                        props.iter().find(|p| *p.id() == av.context.prop_id())
                    {
                        prop
                    } else {
                        continue;
                    };

                    for error in status.errors {
                        validation_errors.push((prop.json_pointer(), error));
                    }
                }
                None => warn!(
                    "skipping, could not find attribute value for status: {:?}",
                    status
                ),
            }
        }

        let sub_checks: Vec<QualificationSubCheck> = validation_errors
            .iter()
            .map(|(prop_name, error)| QualificationSubCheck {
                description: format!("validation failed for \"{}\": {}", prop_name, error.message),
                status: QualificationSubCheckStatus::Failure,
            })
            .collect();

        let name = "All fields are valid";
        Ok(QualificationView {
            title: name.to_string(),
            output: vec![],
            description: None,
            link: None,
            result: Some(QualificationResult {
                status: if sub_checks.is_empty() {
                    QualificationSubCheckStatus::Success
                } else {
                    QualificationSubCheckStatus::Failure
                },
                title: None,
                link: None,
                sub_checks,
            }),
            qualification_name: name.to_string(),
        })
    }
}
