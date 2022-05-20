use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

use crate::{
    attribute::value::{AttributeValue, AttributeValueId, AttributeValuePayload},
    component::{ComponentKind, ComponentResult},
    AttributeReadContext, Component, ComponentError, DalContext, EncryptedSecret, PropKind,
    SecretError, SecretId, StandardModel, StandardModelError, System,
};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ComponentViewError {
    #[error("error serializing/deserializing json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("standard model error: {0}")]
    StandardModel(#[from] StandardModelError),
    #[error("secret error: {0}")]
    Secret(#[from] SecretError),
    #[error("secret not found: {0}")]
    SecretNotFound(SecretId),
    #[error("json pointer not found: {1} at {:0?}")]
    JSONPointerNotFound(serde_json::Value, String),
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ComponentView {
    pub system: Option<System>,
    pub kind: ComponentKind,
    pub properties: serde_json::Value,
}

impl Default for ComponentView {
    fn default() -> Self {
        Self {
            system: Default::default(),
            kind: Default::default(),
            properties: serde_json::json!({}),
        }
    }
}

impl ComponentView {
    pub async fn for_context(
        ctx: &DalContext<'_, '_>,
        context: AttributeReadContext,
    ) -> ComponentResult<ComponentView> {
        let component_id = match context.component_id() {
            Some(c) => c,
            None => {
                return Err(ComponentError::BadAttributeReadContext(
                    "component_id is required".to_string(),
                ))
            }
        };

        let component = Component::get_by_id(ctx, &component_id)
            .await?
            .ok_or(ComponentError::NotFound(component_id))?;

        // Perhaps get_by_id should just do this? -- Adam
        let system = match context.system_id() {
            Some(system_id) => System::get_by_id(ctx, &system_id).await?,
            None => None,
        };

        let mut initial_work = AttributeValue::list_payload_for_read_context(ctx, context).await?;

        // `AttributeValueId -> serde_json pointer` so when we have a parent_attribute_value_id,
        // we know _exactly_ where in the structure we need to insert, when we have a
        // parent_attribute_resolver_id.
        let mut json_pointer_for_attribute_value_id: HashMap<AttributeValueId, String> =
            HashMap::new();

        // We sort the work queue according to the order of every nested IndexMap. This ensures that
        // when we reconstruct the final properties data, we don't have to worry about the order things
        // appear in - they are certain to be the right order.
        let attribute_value_order: Vec<AttributeValueId> = initial_work
            .iter()
            .filter_map(|avp| avp.attribute_value.index_map())
            .flat_map(|index_map| index_map.order())
            .copied()
            .collect();
        initial_work.sort_by_cached_key(|avp| {
            attribute_value_order
                .iter()
                .position(|attribute_value_id| attribute_value_id == avp.attribute_value.id())
                .unwrap_or(0)
        });

        // We need the work_queue to be a VecDeque so we can pop elements off of the front
        // as it's supposed to be a queue, not a stack.
        let mut work_queue: VecDeque<AttributeValuePayload> = VecDeque::from(initial_work);

        let mut properties = serde_json::json![{}];
        let mut root_stack: Vec<(Option<AttributeValueId>, String)> = vec![(None, "".to_string())];

        while !work_queue.is_empty() {
            let mut unprocessed: Vec<AttributeValuePayload> = vec![];
            let (root_id, json_pointer) = root_stack
                .pop()
                .expect("the root prop id queue cannot be empty while work_queue is not empty");

            while let Some(AttributeValuePayload {
                prop,
                func_binding_return_value,
                attribute_value,
                parent_attribute_value_id,
            }) = work_queue.pop_front()
            {
                if let Some(func_binding_return_value) = func_binding_return_value {
                    if let Some(value) = func_binding_return_value.value() {
                        if root_id == parent_attribute_value_id {
                            let insertion_pointer =
                                if let Some(parent_avi) = parent_attribute_value_id {
                                    match json_pointer_for_attribute_value_id.get(&parent_avi) {
                                        Some(ptr) => ptr.clone(),
                                        // A `None` here would mean that we're trying to process a child before we've handled its parent,
                                        // and that shouldn't be possible given how we're going through the work_queue.
                                        None => unreachable!(),
                                    }
                                } else {
                                    // After we've processed the "root" property, we shouldn't hit this case any more.
                                    json_pointer.clone()
                                };
                            let write_location = match properties.pointer_mut(&insertion_pointer) {
                                Some(write_location) => write_location,
                                None => {
                                    return Err(ComponentError::BadJsonPointer(
                                        insertion_pointer.clone(),
                                        properties.to_string(),
                                    ))
                                }
                            };
                            let next_json_pointer =
                                if let Some(object) = write_location.as_object_mut() {
                                    if let Some(key) = attribute_value.key() {
                                        object.insert(key.to_string(), value.clone());
                                        format!("{}/{}", insertion_pointer, key)
                                    } else {
                                        object.insert(prop.name().to_string(), value.clone());
                                        format!("{}/{}", insertion_pointer, prop.name())
                                    }
                                } else if let Some(array) = write_location.as_array_mut() {
                                    // This code can just push, because we ordered the work queue above.
                                    // Magic!
                                    array.push(value.clone());
                                    format!("{}/{}", insertion_pointer, array.len() - 1)
                                } else {
                                    // Note: this shouldn't ever actually get used.
                                    insertion_pointer.to_string()
                                };
                            // Record the json pointer path to *this* specific attribute resolver's location.
                            json_pointer_for_attribute_value_id
                                .insert(*attribute_value.id(), next_json_pointer.clone());

                            match prop.kind() {
                                &PropKind::Object | &PropKind::Array | &PropKind::Map => {
                                    root_stack
                                        .push((Some(*attribute_value.id()), next_json_pointer));
                                }
                                _ => {}
                            }
                        } else {
                            unprocessed.push(AttributeValuePayload::new(
                                prop,
                                Some(func_binding_return_value),
                                attribute_value,
                                parent_attribute_value_id,
                            ));
                        }
                    }
                }
            }
            work_queue = VecDeque::from(unprocessed);
        }
        Ok(ComponentView {
            system,
            kind: *component.kind(),
            properties: properties["root"].clone(),
        })
    }

    pub async fn reencrypt_secrets(
        ctx: &DalContext<'_, '_>,
        component: &mut veritech::ComponentView,
    ) -> Result<(), ComponentViewError> {
        if component.kind != veritech::ComponentKind::Credential {
            return Ok(());
        }

        // If it's a credential it's already unencrypted
        if let Some(object) = component
            .properties
            .as_object_mut()
            .and_then(|obj| obj.get_mut("root").and_then(|obj| obj.as_object_mut()))
        {
            // Note: we can't know which fields are WidgetKind::SecretSelect as we lose information by being so low on the stack
            // So for now we will try to decrypt every integer root field, which kinda suck
            //
            // TODO: traverse tree and decrypt leafs
            for (_key, value) in object {
                if let Some(raw_id) = value.as_i64() {
                    let decrypted_secret = EncryptedSecret::get_by_id(ctx, &raw_id.into())
                        .await?
                        .ok_or_else(|| ComponentViewError::SecretNotFound(raw_id.into()))?
                        .decrypt(ctx)
                        .await?;
                    let encoded = ctx
                        .encryption_key()
                        .encrypt_and_encode(&serde_json::to_string(&decrypted_secret.message())?);

                    *value = serde_json::to_value(&decrypted_secret)?;
                    match value.pointer_mut("/message") {
                        Some(v) => {
                            *v = serde_json::json!({
                                "cycloneEncryptedDataMarker": true,
                                "encryptedSecret": encoded
                            })
                        }
                        None => {
                            return Err(ComponentViewError::JSONPointerNotFound(
                                value.clone(),
                                "/message".to_owned(),
                            ))
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

impl From<ComponentKind> for veritech::ComponentKind {
    fn from(view: ComponentKind) -> Self {
        match view {
            ComponentKind::Standard => Self::Standard,
            ComponentKind::Credential => Self::Credential,
        }
    }
}

impl From<ComponentView> for veritech::ComponentView {
    fn from(view: ComponentView) -> Self {
        Self {
            // Filters internal data out, leaving only what is useful
            system: view.system.map(|system| veritech::SystemView {
                name: system.name().to_owned(),
            }),
            kind: view.kind.into(),
            properties: view.properties,
        }
    }
}
