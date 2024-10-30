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
    change_status::ChangeStatus::Added,
    diagram::geometry::RawGeometry,
    prop::{PropError, PropPath},
    AttributeValue, Component, ComponentError, ComponentId, DalContext, Func, FuncError, Prop,
    PropKind, Schema, SchemaError, SchemaId, WsEvent, WsEventError,
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
    #[error("cannot create component with 'self' as a placeholder")]
    CannotCreateComponentWithSelfPlaceholder,
    #[error("component error: {0}")]
    Component(#[from] ComponentError),
    #[error("cannot add an action of kind {0} because component {1} does not have an action of that kind")]
    ComponentDoesNotHaveAction(ActionKind, ComponentId),
    #[error(
        "cannot add a manual action named {0} because component {1} does not have a manual action with that name"
    )]
    ComponentDoesNotHaveManualAction(String, ComponentId),
    #[error("Component with management placeholder {0} could not be found")]
    ComponentWithPlaceholderNotFound(String),
    #[error("Duplicate component placeholder {0}")]
    DuplicateComponentPlaceholder(String),
    #[error("func error: {0}")]
    Func(#[from] FuncError),
    #[error("prop error: {0}")]
    Prop(#[from] PropError),
    #[error("schema error: {0}")]
    Schema(#[from] SchemaError),
    #[error("Cannot create component for Schema {0}, this schema does not exist or is not managed by this component")]
    SchemaDoesNotExist(String),
    #[error("ws event error: {0}")]
    WsEvent(#[from] WsEventError),
}

pub type ManagementResult<T> = Result<T, ManagementError>;

#[derive(Clone, Debug, Copy, Serialize, Deserialize, PartialEq)]
pub struct NumericGeometry {
    pub x: f64,
    pub y: f64,
    pub width: Option<f64>,
    pub height: Option<f64>,
}

impl NumericGeometry {
    pub fn offset_by(&self, mut x_off: f64, mut y_off: f64) -> Self {
        if !x_off.is_normal() {
            x_off = 0.0;
        }
        if !y_off.is_normal() {
            y_off = 0.0;
        }

        let x = if self.x.is_normal() {
            self.x + x_off
        } else {
            x_off
        };

        let y = if self.y.is_normal() {
            self.y + y_off
        } else {
            y_off
        };

        Self {
            x,
            y,
            width: self.width,
            height: self.height,
        }
    }
}

#[inline(always)]
fn avoid_nan_string(n: f64, fallback: f64) -> String {
    if n.is_normal() { n.round() } else { fallback }.to_string()
}

impl From<NumericGeometry> for RawGeometry {
    fn from(value: NumericGeometry) -> Self {
        Self {
            x: avoid_nan_string(value.x, 0.0),
            y: avoid_nan_string(value.y, 0.0),
            width: value.width.map(|w| avoid_nan_string(w, 500.0)),
            height: value.height.map(|h| avoid_nan_string(h, 500.0)),
        }
    }
}

impl From<RawGeometry> for NumericGeometry {
    fn from(value: RawGeometry) -> Self {
        Self {
            x: value.x.parse().ok().unwrap_or(0.0),
            y: value.y.parse().ok().unwrap_or(0.0),
            width: value.width.and_then(|w| w.parse().ok()),
            height: value.height.and_then(|h| h.parse().ok()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagementUpdateOperation {
    properties: Option<serde_json::Value>,
    geometry: Option<RawGeometry>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagementCreateOperation {
    kind: Option<String>,
    properties: Option<serde_json::Value>,
    geometry: Option<NumericGeometry>,
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagementOperations {
    create: Option<HashMap<String, ManagementCreateOperation>>,
    update: Option<HashMap<String, ManagementUpdateOperation>>,
    actions: Option<HashMap<String, ManagementActionOperation>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

pub struct ManagementOperator<'a> {
    ctx: &'a DalContext,
    manager_component_id: ComponentId,
    last_component_geometry: NumericGeometry,
    operations: ManagementOperations,
    manager_schema_id: SchemaId,
    schema_map: HashMap<String, SchemaId>,
    component_id_placeholders: HashMap<String, ComponentId>,
}

impl<'a> ManagementOperator<'a> {
    pub async fn new(
        ctx: &'a DalContext,
        manager_component_id: ComponentId,
        manager_component_geometry: RawGeometry,
        operations: ManagementOperations,
        schema_map: HashMap<String, SchemaId>,
        mut component_id_placeholders: HashMap<String, ComponentId>,
    ) -> ManagementResult<Self> {
        component_id_placeholders.insert(SELF_ID.to_string(), manager_component_id);

        let manager_schema_id = Component::schema_for_component_id(ctx, manager_component_id)
            .await?
            .id();

        Ok(Self {
            ctx,
            manager_component_id,
            last_component_geometry: manager_component_geometry.into(),
            operations,
            manager_schema_id,
            schema_map,
            component_id_placeholders,
        })
    }

    async fn create_component(
        &self,
        placeholder: &str,
        operation: &ManagementCreateOperation,
    ) -> ManagementResult<(ComponentId, NumericGeometry)> {
        let schema_id = match &operation.kind {
            Some(kind) => self
                .schema_map
                .get(kind)
                .copied()
                .ok_or(ManagementError::SchemaDoesNotExist(kind.clone()))?,
            None => self.manager_schema_id,
        };

        let variant_id = Schema::get_or_install_default_variant(self.ctx, schema_id).await?;

        let mut component = Component::new(self.ctx, placeholder, variant_id).await?;
        let geometry = if let Some(numeric_geometry) = &operation.geometry {
            component
                .set_raw_geometry(self.ctx, (*numeric_geometry).into())
                .await?;

            *numeric_geometry
        } else {
            // We don't want to just stack components on top of each other if no
            // geometry is provided, so we're gonna do a bit of you-just-won
            // solitaire staggering
            let auto_geometry = self.last_component_geometry.offset_by(50.0, 50.0);
            component
                .set_raw_geometry(self.ctx, auto_geometry.into())
                .await?;

            auto_geometry
        };

        WsEvent::component_created(
            self.ctx,
            component
                .into_frontend_type(self.ctx, Added, &mut HashMap::new())
                .await?,
        )
        .await?
        .publish_on_commit(self.ctx)
        .await?;

        Ok((component.id(), geometry))
    }

    async fn creates(&mut self) -> ManagementResult<()> {
        if let Some(creates) = &self.operations.create {
            for (placeholder, operation) in creates {
                if placeholder == SELF_ID {
                    return Err(ManagementError::CannotCreateComponentWithSelfPlaceholder);
                }

                if self.component_id_placeholders.contains_key(placeholder) {
                    return Err(ManagementError::DuplicateComponentPlaceholder(
                        placeholder.to_owned(),
                    ));
                }

                let (component_id, geometry) =
                    self.create_component(placeholder, operation).await?;

                self.last_component_geometry = geometry;

                self.component_id_placeholders
                    .insert(placeholder.to_owned(), component_id);

                if let Some(properties) = &operation.properties {
                    update_component(
                        self.ctx,
                        component_id,
                        properties,
                        &[&["root", "si", "name"]],
                    )
                    .await?;
                }

                Component::add_manages_edge_to_component(
                    self.ctx,
                    self.manager_component_id,
                    component_id,
                    crate::EdgeWeightKind::Manages,
                )
                .await?;
            }
        }

        Ok(())
    }

    async fn get_real_component_id(&self, placeholder: &String) -> ManagementResult<ComponentId> {
        self.component_id_placeholders
            .get(placeholder)
            .copied()
            .ok_or(ManagementError::ComponentWithPlaceholderNotFound(
                placeholder.to_owned(),
            ))
    }

    async fn updates(&mut self) -> ManagementResult<()> {
        if let Some(updates) = &self.operations.update {
            for (placeholder, operation) in updates {
                let component_id = self.get_real_component_id(placeholder).await?;

                if let Some(properties) = &operation.properties {
                    update_component(self.ctx, component_id, properties, &[]).await?;
                }
                if let Some(raw_geometry) = &operation.geometry {
                    let mut component = Component::get_by_id(self.ctx, component_id).await?;
                    component
                        .set_raw_geometry(self.ctx, raw_geometry.to_owned())
                        .await?;
                }
            }
        }

        Ok(())
    }

    async fn actions(&self) -> ManagementResult<()> {
        if let Some(actions) = &self.operations.actions {
            for (placeholder, operations) in actions {
                let component_id = self.get_real_component_id(placeholder).await?;

                operate_actions(self.ctx, component_id, operations).await?;
            }

            WsEvent::action_list_updated(self.ctx)
                .await?
                .publish_on_commit(self.ctx)
                .await?;
        }

        Ok(())
    }

    pub async fn operate(&mut self) -> ManagementResult<()> {
        self.creates().await?;
        self.updates().await?;
        self.actions().await?;

        Ok(())
    }
}

fn identify_action(action_str: &str) -> ActionIdentifier {
    match action_str {
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
            manual_func_name: Some(action_str.to_string()),
        },
    }
}

async fn operate_actions(
    ctx: &DalContext,
    component_id: ComponentId,
    operation: &ManagementActionOperation,
) -> ManagementResult<()> {
    if let Some(remove_actions) = &operation.remove {
        for to_remove in remove_actions
            .iter()
            .map(|action| identify_action(action.as_str()))
        {
            remove_action(ctx, component_id, to_remove).await?;
        }
    }
    if let Some(add_actions) = &operation.add {
        let sv_id = Component::schema_variant_id(ctx, component_id).await?;
        let available_actions = ActionPrototype::for_variant(ctx, sv_id).await?;
        for action in add_actions
            .iter()
            .map(|action| identify_action(action.as_str()))
        {
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
    properties: &serde_json::Value,
    extra_ignore_paths: &[&[&str]],
) -> ManagementResult<()> {
    let variant_id = Component::schema_variant_id(ctx, component_id).await?;

    // walk the properties serde_json::Value object without recursion
    let mut work_queue = VecDeque::new();
    work_queue.push_back((vec!["root".to_string()], properties));

    while let Some((path, current_val)) = work_queue.pop_front() {
        let path_as_refs: Vec<_> = path.iter().map(|part| part.as_str()).collect();
        if IGNORE_PATHS.contains(&path_as_refs.as_slice())
            || extra_ignore_paths.contains(&path_as_refs.as_slice())
        {
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
            AttributeValue::update(ctx, path_attribute_value_id, Some(current_val.to_owned()))
                .await?;
            continue;
        }

        let prop = Prop::get_by_id(ctx, prop_id).await?;

        match prop.kind {
            PropKind::String | PropKind::Boolean | PropKind::Integer | PropKind::Json => {
                // todo: type check!
                let view = AttributeValue::get_by_id(ctx, path_attribute_value_id)
                    .await?
                    .view(ctx)
                    .await?;
                if Some(current_val) != view.as_ref() {
                    AttributeValue::update(
                        ctx,
                        path_attribute_value_id,
                        Some(current_val.to_owned()),
                    )
                    .await?;
                }
            }
            PropKind::Object => {
                let serde_json::Value::Object(obj) = current_val else {
                    continue;
                };

                for (key, value) in obj {
                    let mut new_path = path.clone();
                    new_path.push(key.to_owned());
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
                    match map_children.get(key) {
                        Some(child_id) => {
                            if AttributeValue::is_set_by_dependent_function(ctx, *child_id).await? {
                                continue;
                            }
                            let view = AttributeValue::get_by_id(ctx, *child_id)
                                .await?
                                .view(ctx)
                                .await?;
                            if Some(value) != view.as_ref() {
                                AttributeValue::update(ctx, *child_id, Some(value.to_owned()))
                                    .await?;
                            }
                        }
                        None => {
                            AttributeValue::insert(
                                ctx,
                                path_attribute_value_id,
                                Some(value.to_owned()),
                                Some(key.to_owned()),
                            )
                            .await?;
                        }
                    }
                }
            }
            PropKind::Array => {
                if matches!(current_val, serde_json::Value::Array(_)) {
                    let view = AttributeValue::get_by_id(ctx, path_attribute_value_id)
                        .await?
                        .view(ctx)
                        .await?;

                    if Some(current_val) != view.as_ref() {
                        // Just update the entire array whole cloth
                        AttributeValue::update(
                            ctx,
                            path_attribute_value_id,
                            Some(current_val.to_owned()),
                        )
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
