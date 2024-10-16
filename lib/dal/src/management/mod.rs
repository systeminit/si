use std::collections::{HashMap, VecDeque};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use veritech_client::{ManagementFuncStatus, ManagementResultSuccess};

use crate::{
    attribute::value::AttributeValueError,
    prop::{PropError, PropPath},
    AttributeValue, Component, ComponentError, ComponentId, DalContext, Prop, PropKind,
};

pub mod prototype;

#[derive(Debug, Error)]
pub enum ManagementError {
    #[error("attribute value error: {0}")]
    AttributeValue(#[from] AttributeValueError),
    #[error("component error: {0}")]
    Component(#[from] ComponentError),
    #[error("prop error: {0}")]
    Prop(#[from] PropError),
}

pub type ManagementResult<T> = Result<T, ManagementError>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagementUpdateOperation {
    properties: Option<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagementOperations {
    update: HashMap<String, ManagementUpdateOperation>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagementFuncReturn {
    pub status: ManagementFuncStatus,
    pub operations: Option<ManagementOperations>,
    pub message: Option<String>,
    pub error: Option<String>,
}

impl TryFrom<ManagementResultSuccess> for ManagementFuncReturn {
    type Error = serde_json::Error;

    fn try_from(value: ManagementResultSuccess) -> Result<Self, Self::Error> {
        Ok(ManagementFuncReturn {
            status: value.health,
            operations: match value.operations {
                Some(ops) => serde_json::from_value(ops)?,
                None => None,
            },
            message: value.message,
            error: value.error,
        })
    }
}

const SELF_ID: &str = "self";

pub async fn operate(
    ctx: &DalContext,
    manager_component_id: ComponentId,
    operations: ManagementOperations,
) -> ManagementResult<()> {
    // creation should be first

    for (operation_component_id, operation) in operations.update {
        // We only support operations on self right now
        if operation_component_id != SELF_ID {
            continue;
        }

        if let Some(properties) = operation.properties {
            update_component(ctx, manager_component_id, properties).await?;
        }
    }

    Ok(())
}

// Update operations should not be able to set these props or their children
const IGNORE_PATHS: [&[&str]; 6] = [
    &["root", "code"],
    &["root", "deleted_at"],
    &["root", "qualification"],
    &["root", "resource"],
    &["root", "resource_value"],
    &["root", "secrets"],
];

async fn update_component(
    ctx: &DalContext,
    component_id: ComponentId,
    properties: serde_json::Value,
) -> ManagementResult<()> {
    let variant_id = Component::schema_variant_id(ctx, component_id).await?;

    // walk the properties serde_json::Value object without recursion
    let mut work_queue = VecDeque::new();
    work_queue.push_back((vec!["root".to_string()], properties));

    while let Some((path, current_val)) = work_queue.pop_front() {
        let path_as_refs: Vec<_> = path.iter().map(|part| part.as_str()).collect();
        if IGNORE_PATHS.contains(&path_as_refs.as_slice()) {
            continue;
        }

        let Some(prop_id) =
            Prop::find_prop_id_by_path_opt(ctx, variant_id, &PropPath::new(path.as_slice()))
                .await?
        else {
            continue;
        };

        let path_attribute_value_id =
            Component::attribute_value_for_prop_id(ctx, component_id, prop_id).await?;

        if AttributeValue::is_set_by_dependent_function(ctx, path_attribute_value_id).await? {
            continue;
        }
        if let serde_json::Value::Null = current_val {
            AttributeValue::update(ctx, path_attribute_value_id, Some(current_val)).await?;
            continue;
        }

        let prop = Prop::get_by_id_or_error(ctx, prop_id).await?;

        match prop.kind {
            PropKind::String | PropKind::Boolean | PropKind::Integer | PropKind::Json => {
                // todo: type check!
                let view = AttributeValue::get_by_id_or_error(ctx, path_attribute_value_id)
                    .await?
                    .view(ctx)
                    .await?;
                if Some(&current_val) != view.as_ref() {
                    AttributeValue::update(ctx, path_attribute_value_id, Some(current_val)).await?;
                }
            }
            PropKind::Object => {
                let serde_json::Value::Object(obj) = current_val else {
                    continue;
                };

                for (key, value) in obj {
                    let mut new_path = path.clone();
                    new_path.push(key);
                    work_queue.push_back((new_path, value));
                }
            }
            PropKind::Map => {
                let serde_json::Value::Object(map) = current_val else {
                    continue;
                };

                let map_children =
                    AttributeValue::map_children(ctx, path_attribute_value_id).await?;

                // Remove any children that are not in the new map
                for (key, child_id) in &map_children {
                    if !map.contains_key(key) {
                        if AttributeValue::is_set_by_dependent_function(ctx, *child_id).await? {
                            continue;
                        }

                        AttributeValue::remove_by_id(ctx, *child_id).await?;
                    }
                }

                // We do not descend below a map. Instead we update the *entire*
                // child tree of each map key
                for (key, value) in map {
                    match map_children.get(&key) {
                        Some(child_id) => {
                            if AttributeValue::is_set_by_dependent_function(ctx, *child_id).await? {
                                continue;
                            }
                            let view = AttributeValue::get_by_id_or_error(ctx, *child_id)
                                .await?
                                .view(ctx)
                                .await?;
                            if Some(&value) != view.as_ref() {
                                AttributeValue::update(ctx, *child_id, Some(value)).await?;
                            }
                        }
                        None => {
                            AttributeValue::insert(
                                ctx,
                                path_attribute_value_id,
                                Some(value),
                                Some(key),
                            )
                            .await?;
                        }
                    }
                }
            }
            PropKind::Array => {
                if matches!(current_val, serde_json::Value::Array(_)) {
                    let view = AttributeValue::get_by_id_or_error(ctx, path_attribute_value_id)
                        .await?
                        .view(ctx)
                        .await?;

                    if Some(&current_val) != view.as_ref() {
                        // Just update the entire array whole cloth
                        AttributeValue::update(ctx, path_attribute_value_id, Some(current_val))
                            .await?;
                    }
                }
            }
        }
    }

    Ok(())
}
