use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use si_data::PgTxn;

use crate::{
    system::UNSET_SYSTEM_ID, AttributeResolver, AttributeResolverId, AttributeResolverValue,
    Component, ComponentError, ComponentId, PropId, PropKind, StandardModel, System, SystemId,
    Tenancy, Visibility,
};

use super::ComponentResult;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ComponentViewError {
    #[error("error serializing/deserializing json: {0}")]
    SerdeJson(#[from] serde_json::Error),
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct ComponentView {
    pub name: String,
    pub system: Option<System>,
    pub properties: serde_json::Value,
}

impl ComponentView {
    pub async fn for_component_and_system(
        txn: &PgTxn<'_>,
        tenancy: &Tenancy,
        visibility: &Visibility,
        component_id: ComponentId,
        system_id: SystemId,
    ) -> ComponentResult<ComponentView> {
        let component = Component::get_by_id(txn, tenancy, visibility, &component_id)
            .await?
            .ok_or(ComponentError::NotFound(component_id))?;

        // Perhaps get_by_id should just do this? -- Adam
        let system = if system_id == UNSET_SYSTEM_ID {
            None
        } else {
            System::get_by_id(txn, tenancy, visibility, &system_id).await?
        };

        let mut work_queue = AttributeResolver::list_values_for_component(
            txn,
            tenancy,
            visibility,
            component_id,
            system_id,
        )
        .await?;
        // `AttributeResolverId -> serde_json pointer` so when we have a parent_attribute_resolver_id,
        // we know _exactly_ where in the structure we need to insert, when we have a
        // parent_attribute_resolver_id.
        let mut json_pointer_for_attribute_resolver_id: HashMap<AttributeResolverId, String> =
            HashMap::new();

        // We sort the work queue according to the order of every nested IndexMap. This ensures that
        // when we reconstruct the final properties data, we don't have to worry about the order things
        // appear in - they are certain to be the right order.
        let attribute_resolver_order: Vec<AttributeResolverId> = work_queue
            .iter()
            .filter_map(|arv| arv.attribute_resolver.index_map())
            .map(|index_map| index_map.order())
            .flatten()
            .copied()
            .collect();
        work_queue.sort_by_cached_key(|arv| {
            attribute_resolver_order
                .iter()
                .position(|attribute_resolver_id| {
                    attribute_resolver_id == arv.attribute_resolver.id()
                })
                .or(Some(0))
                .unwrap()
        });

        let mut properties = serde_json::json![{}];
        let mut root_stack: Vec<(Option<PropId>, String)> = vec![(None, "".to_string())];

        while !work_queue.is_empty() {
            let mut unprocessed: Vec<AttributeResolverValue> = vec![];
            let (root_id, json_pointer) = root_stack
                .pop()
                .expect("the root prop id queue cannot be empty while work_queue is not empty");

            while let Some(AttributeResolverValue {
                prop,
                parent_prop_id,
                fbrv,
                attribute_resolver,
                parent_attribute_resolver_id,
            }) = work_queue.pop()
            {
                if let Some(value) = fbrv.value() {
                    if root_id == parent_prop_id {
                        let insertion_pointer =
                            if let Some(parent_ari) = parent_attribute_resolver_id {
                                match json_pointer_for_attribute_resolver_id.get(&parent_ari) {
                                    Some(ptr) => ptr.clone(),
                                    // A `None` here would mean that we're trying to process a child before we've handled its parent,
                                    // and that shouldn't be possible given how we're going through the work_queue.
                                    None => unreachable!(),
                                }
                            } else {
                                // After we've processed the "root" properties, we shouldn't hit this case any more.
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
                            if write_location.is_object() && attribute_resolver.key().is_some() {
                                let key = attribute_resolver.key().unwrap();
                                write_location
                                    .as_object_mut()
                                    .unwrap()
                                    .insert(key.to_string(), value.clone());
                                format!("{}/{}", insertion_pointer, key)
                            } else if write_location.is_object() {
                                write_location
                                    .as_object_mut()
                                    .unwrap()
                                    .insert(prop.name().to_string(), value.clone());
                                format!("{}/{}", insertion_pointer, prop.name())
                            } else if write_location.is_array() {
                                // This code can just push, because we ordered the work queue above.
                                // Magic!
                                let array = write_location.as_array_mut().unwrap();
                                array.push(value.clone());
                                format!("{}/{}", insertion_pointer, array.len() - 1)
                            } else {
                                // Note: this shouldn't ever actually get used.
                                insertion_pointer.to_string()
                            };
                        // Record the json pointer path to *this* specific attribute resolver's location.
                        json_pointer_for_attribute_resolver_id
                            .insert(*attribute_resolver.id(), next_json_pointer.clone());

                        match prop.kind() {
                            &PropKind::Object | &PropKind::Array | &PropKind::Map => {
                                root_stack.push((Some(*prop.id()), next_json_pointer));
                            }
                            _ => {}
                        }
                    } else {
                        unprocessed.push(AttributeResolverValue::new(
                            prop,
                            parent_prop_id,
                            fbrv,
                            attribute_resolver,
                            parent_attribute_resolver_id,
                        ));
                    }
                }
            }
            work_queue = unprocessed;
        }
        Ok(ComponentView {
            name: component.name().to_string(),
            system,
            properties,
        })
    }
}

impl From<ComponentView> for veritech::ComponentView {
    fn from(view: ComponentView) -> Self {
        Self {
            name: view.name,
            // Filters internal data out, leaving only what is useful
            system: view.system.map(|system| veritech::SystemView {
                name: system.name().to_owned(),
            }),
            properties: serde_json::to_value(view.properties)
                .expect("HashMap<String, Value> shouldn't fail to serialize to Value"),
        }
    }
}
