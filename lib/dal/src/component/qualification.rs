use serde::Deserialize;
use std::collections::HashMap;
use telemetry::prelude::*;

use crate::attribute::value::AttributeValue;
use crate::attribute::value::AttributeValueError;
use crate::component::ComponentResult;
use crate::func::binding_return_value::FuncBindingReturnValueId;
use crate::qualification::QualificationView;
use crate::schema::variant::leaves::LeafKind;
use crate::schema::SchemaVariant;
use crate::ws_event::WsEvent;
use crate::{AttributeReadContext, DalContext, StandardModel};
use crate::{Component, ComponentError, ComponentId};

// FIXME(nick): use the formal types from the new version of function authoring instead of this
// struct. This struct is a temporary stopgap until that's implemented.
#[derive(Deserialize, Debug)]
pub struct QualificationEntry {
    pub qualified: bool,
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

        let mut results: Vec<QualificationView> = vec![];

        // Prepare to assemble qualification views and access the "/root/qualification" prop tree.
        // We will use its implicit internal provider id and its corresponding prop id to do so.
        let qualification_map_implicit_internal_provider =
            SchemaVariant::find_leaf_implicit_internal_provider(
                ctx,
                *schema_variant.id(),
                LeafKind::Qualification,
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
        let mut entry_attribute_values: HashMap<String, FuncBindingReturnValueId> = HashMap::new();
        for entry_attribute_value in prop_qualification_map_attribute_value
            .child_attribute_values(ctx)
            .await?
        {
            let entry_attribute_value_id = *entry_attribute_value.id();
            let func_binding_return_value_id = entry_attribute_value.func_binding_return_value_id();
            let key = entry_attribute_value
                .key
                .ok_or(ComponentError::FoundMapEntryWithoutKey(
                    entry_attribute_value_id,
                ))?;
            entry_attribute_values.insert(key, func_binding_return_value_id);
        }

        // Now, check all qualifications in the tree.
        let implicit_qualification_map_attribute_read_context = AttributeReadContext {
            internal_provider_id: Some(*qualification_map_implicit_internal_provider.id()),
            component_id: Some(component_id),
            ..AttributeReadContext::default()
        };
        let implicit_qualification_map_attribute_value = AttributeValue::find_for_context(
            ctx,
            implicit_qualification_map_attribute_read_context,
        )
        .await?
        .ok_or(AttributeValueError::NotFoundForReadContext(
            implicit_qualification_map_attribute_read_context,
        ))?;
        let maybe_qualification_map_value = implicit_qualification_map_attribute_value
            .get_value(ctx)
            .await?;
        if let Some(qualification_map_value) = maybe_qualification_map_value {
            let qualification_map: HashMap<String, QualificationEntry> =
                serde_json::from_value(qualification_map_value)?;

            for (qualification_name, entry) in qualification_map {
                let found_func_binding_return_value_id =
                    entry_attribute_values.get(&qualification_name).unwrap();
                let qual_view = QualificationView::new(
                    ctx,
                    qualification_name,
                    entry,
                    *found_func_binding_return_value_id,
                )
                .await?;
                results.push(qual_view);
            }
        }

        WsEvent::checked_qualifications(ctx, component_id)
            .publish(ctx)
            .await?;

        Ok(results)
    }
}
