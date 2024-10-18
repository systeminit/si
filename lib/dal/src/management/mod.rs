use std::collections::{HashMap, VecDeque};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use veritech_client::{ManagementFuncStatus, ManagementResultSuccess};

use crate::{
    action::{
        prototype::{ActionKind, ActionPrototype, ActionPrototypeError},
        Action, ActionError,
    },
    attribute::value::AttributeValueError,
    prop::{PropError, PropPath},
    AttributeValue, Component, ComponentError, ComponentId, DalContext, Func, FuncError, Prop,
    PropKind, WsEvent, WsEventError,
};

pub mod prototype;

#[derive(Debug, Error)]
pub enum ManagementError {
    #[error("action error: {0}")]
    Action(#[from] ActionError),
    #[error("action prototype error: {0}")]
    ActionPrototype(#[from] ActionPrototypeError),
    #[error("attribute value error: {0}")]
    AttributeValue(#[from] AttributeValueError),
    #[error("component error: {0}")]
    Component(#[from] ComponentError),
    #[error("cannot add an action of kind {0} because component {1} does not have an action of that kind")]
    ComponentDoesNotHaveAction(ActionKind, ComponentId),
    #[error(
        "cannot add a manual action named {0} because component {1} does not have a manual action with that name"
    )]
    ComponentDoesNotHaveManualAction(String, ComponentId),
    #[error("func error: {0}")]
    Func(#[from] FuncError),
    #[error("prop error: {0}")]
    Prop(#[from] PropError),
    #[error("ws event error: {0}")]
    WsEvent(#[from] WsEventError),
}

pub type ManagementResult<T> = Result<T, ManagementError>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagementUpdateOperation {
    properties: Option<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActionIdentifier {
    pub kind: ActionKind,
    pub manual_func_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManagementActionOperation {
    add: Option<Vec<String>>,
    remove: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagementOperations {
    update: Option<HashMap<String, ManagementUpdateOperation>>,
    actions: Option<HashMap<String, ManagementActionOperation>>,
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

    if let Some(updates) = operations.update {
        for (operation_component_id, operation) in updates {
            // We only support operations on self right now
            if operation_component_id != SELF_ID {
                continue;
            }

            let real_component_id = manager_component_id;

            if let Some(properties) = operation.properties {
                update_component(ctx, real_component_id, properties).await?;
            }
        }
    }

    if let Some(actions) = operations.actions {
        for (operation_component_id, operations) in actions {
            // We only support operations on self right now
            if operation_component_id != SELF_ID {
                continue;
            }

            let real_component_id = manager_component_id;

            operate_actions(ctx, real_component_id, operations).await?;
        }

        WsEvent::action_list_updated(ctx)
            .await?
            .publish_on_commit(ctx)
            .await?;
    }

    Ok(())
}

fn identify_action(action_string: String) -> ActionIdentifier {
    match action_string.as_str() {
        "create" => ActionIdentifier {
            kind: ActionKind::Create,
            manual_func_name: None,
        },
        "destroy" => ActionIdentifier {
            kind: ActionKind::Destroy,
            manual_func_name: None,
        },
        "refresh" => ActionIdentifier {
            kind: ActionKind::Refresh,
            manual_func_name: None,
        },
        "update" => ActionIdentifier {
            kind: ActionKind::Update,
            manual_func_name: None,
        },
        _ => ActionIdentifier {
            kind: ActionKind::Manual,
            manual_func_name: Some(action_string),
        },
    }
}

async fn operate_actions(
    ctx: &DalContext,
    component_id: ComponentId,
    operation: ManagementActionOperation,
) -> ManagementResult<()> {
    if let Some(remove_actions) = operation.remove {
        for to_remove in remove_actions.into_iter().map(identify_action) {
            remove_action(ctx, component_id, to_remove).await?;
        }
    }
    if let Some(add_actions) = operation.add {
        let sv_id = Component::schema_variant_id(ctx, component_id).await?;
        let available_actions = ActionPrototype::for_variant(ctx, sv_id).await?;
        for action in add_actions.into_iter().map(identify_action) {
            add_action(ctx, component_id, action, &available_actions).await?;
        }
    }

    Ok(())
}

async fn remove_action(
    ctx: &DalContext,
    component_id: ComponentId,
    action: ActionIdentifier,
) -> ManagementResult<()> {
    let actions = Action::find_for_kind_and_component_id(ctx, component_id, action.kind).await?;
    match action.kind {
        ActionKind::Create | ActionKind::Destroy | ActionKind::Refresh | ActionKind::Update => {
            for action_id in actions {
                Action::remove_by_id(ctx, action_id).await?;
            }
        }
        ActionKind::Manual => {
            for action_id in actions {
                let prototype_id = Action::prototype_id(ctx, action_id).await?;
                let func_id = ActionPrototype::func_id(ctx, prototype_id).await?;
                let func = Func::get_by_id_or_error(ctx, func_id).await?;
                if Some(func.name) == action.manual_func_name {
                    Action::remove_by_id(ctx, action_id).await?;
                }
            }
        }
    }

    Ok(())
}

async fn add_action(
    ctx: &DalContext,
    component_id: ComponentId,
    action: ActionIdentifier,
    available_actions: &[ActionPrototype],
) -> ManagementResult<()> {
    let prototype_id = match action.kind {
        ActionKind::Create | ActionKind::Destroy | ActionKind::Refresh | ActionKind::Update => {
            if !Action::find_for_kind_and_component_id(ctx, component_id, action.kind)
                .await?
                .is_empty()
            {
                return Ok(());
            }

            let Some(action_prototype) = available_actions
                .iter()
                .find(|proto| proto.kind == action.kind)
            else {
                return Err(ManagementError::ComponentDoesNotHaveAction(
                    action.kind,
                    component_id,
                ));
            };

            action_prototype.id()
        }
        ActionKind::Manual => {
            let Some(manual_func_name) = action.manual_func_name else {
                return Err(ManagementError::ComponentDoesNotHaveAction(
                    ActionKind::Manual,
                    component_id,
                ));
            };

            let mut proto_id = None;
            for manual_proto in available_actions
                .iter()
                .filter(|proto| proto.kind == ActionKind::Manual)
            {
                let func = Func::get_by_id_or_error(
                    ctx,
                    ActionPrototype::func_id(ctx, manual_proto.id()).await?,
                )
                .await?;
                if func.name == manual_func_name {
                    proto_id = Some(manual_proto.id());
                    break;
                }
            }

            let Some(proto_id) = proto_id else {
                return Err(ManagementError::ComponentDoesNotHaveManualAction(
                    manual_func_name,
                    component_id,
                ));
            };

            proto_id
        }
    };

    Action::new(ctx, prototype_id, Some(component_id)).await?;

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

    let component = Component::get_by_id(ctx, component_id).await?;
    WsEvent::component_updated(
        ctx,
        component
            .into_frontend_type(
                ctx,
                component.change_status(ctx).await?,
                &mut HashMap::new(),
            )
            .await?,
    )
    .await?
    .publish_on_commit(ctx)
    .await?;

    Ok(())
}
