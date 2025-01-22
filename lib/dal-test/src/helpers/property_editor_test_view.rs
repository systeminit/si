use async_recursion::async_recursion;
use color_eyre::eyre::eyre;
use color_eyre::Result;
use dal::property_editor::schema::{
    PropertyEditorProp, PropertyEditorPropKind, PropertyEditorSchema,
};
use dal::property_editor::values::{PropertyEditorValue, PropertyEditorValues};
use dal::property_editor::{PropertyEditorPropId, PropertyEditorValueId};
use dal::{Component, ComponentId, DalContext, Prop};
use itertools::enumerate;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[allow(missing_docs)]
#[derive(Serialize, Deserialize, Debug)]
pub struct PropEditorTestView {
    pub prop: PropertyEditorProp,
    pub value: PropertyEditorValue,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<HashMap<String, PropEditorTestView>>,
}

impl PropEditorTestView {
    fn get_view(&self, prop_path: &[&str]) -> crate::Result<Value> {
        let mut value = serde_json::to_value(self)?;

        // "root" is necessary for compatibility with other prop apis, but we skip it here
        for &prop_name in prop_path.iter().skip(1) {
            value = value
                .get("children")
                .ok_or(eyre!("nothing found in children entry for view"))?
                .get(prop_name)
                .ok_or(eyre!("specific child entry not found for view"))?
                .clone();
        }

        Ok(value)
    }

    /// Gets the "value" for a given [`Prop`](dal::Prop) path.
    pub fn get_value(&self, prop_path: &[&str]) -> crate::Result<Value> {
        let view = self.get_view(prop_path)?;
        Ok(view.get("value").ok_or(eyre!("value not found"))?.clone())
    }

    /// Generates a [`PropEditorTestView`] for a given [`ComponentId`](Component).
    pub async fn for_component_id(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> crate::Result<Self> {
        let sv_id = Component::schema_variant_id(ctx, component_id).await?;

        let PropertyEditorValues {
            root_value_id,
            values,
            child_values,
        } = PropertyEditorValues::assemble(ctx, component_id).await?;

        let PropertyEditorSchema { props, .. } =
            PropertyEditorSchema::assemble(ctx, sv_id, false).await?;

        let root_view = {
            let value = values
                .get(&root_value_id)
                .ok_or(eyre!("no value for root value"))?
                .clone();

            let prop = props
                .get(&value.prop_id)
                .ok_or(eyre!("property editor prop not found"))?;

            Self {
                prop: prop.clone(),
                value,
                children: Self::property_editor_compile_children(
                    ctx,
                    root_value_id,
                    &prop.kind,
                    &values,
                    &child_values,
                    &props,
                )
                .await?,
            }
        };

        Ok(root_view)
    }

    #[async_recursion]
    async fn property_editor_compile_children(
        ctx: &DalContext,
        parent_value_id: PropertyEditorValueId,
        parent_prop_kind: &PropertyEditorPropKind,
        values: &HashMap<PropertyEditorValueId, PropertyEditorValue>,
        child_values: &HashMap<PropertyEditorValueId, Vec<PropertyEditorValueId>>,
        props: &HashMap<PropertyEditorPropId, PropertyEditorProp>,
    ) -> Result<Option<HashMap<String, PropEditorTestView>>> {
        let mut children = HashMap::new();

        for (index, child_id) in enumerate(
            child_values
                .get(&parent_value_id)
                .ok_or(eyre!("could not get children for parent"))?,
        ) {
            let value = values
                .get(child_id)
                .ok_or(eyre!("could not get value for child"))?
                .clone();
            let real_prop = Prop::get_by_id(ctx, value.prop_id.into_inner().into()).await?;
            if real_prop.hidden {
                continue;
            }

            let prop = props
                .get(&value.prop_id)
                .ok_or(eyre!("could not get property editor prop"))?;

            let key = match parent_prop_kind {
                PropertyEditorPropKind::Array => index.to_string(),
                PropertyEditorPropKind::Map => value.key.clone().unwrap_or("ERROR".to_string()),
                _ => prop.name.clone(),
            };

            let child = PropEditorTestView {
                prop: prop.clone(),
                value,
                children: Self::property_editor_compile_children(
                    ctx,
                    *child_id,
                    &prop.kind,
                    values,
                    child_values,
                    props,
                )
                .await?,
            };

            children.insert(key, child);
        }

        Ok(if children.is_empty() {
            None
        } else {
            Some(children)
        })
    }
}
