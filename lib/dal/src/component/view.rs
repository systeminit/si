use si_data::PgTxn;

use crate::{
    func::binding_return_value::FuncBindingReturnValue, system::UNSET_SYSTEM_ID, AttributeResolver,
    Component, ComponentError, ComponentId, Prop, PropId, PropKind, StandardModel, System,
    SystemId, Tenancy, Visibility,
};

use super::ComponentResult;

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
        let mut properties = serde_json::json![{}];
        let mut root_stack: Vec<(Option<PropId>, String)> = vec![(None, "".to_string())];

        while !work_queue.is_empty() {
            let mut unprocessed: Vec<(Prop, Option<PropId>, FuncBindingReturnValue)> = vec![];
            let (root_id, json_pointer) = root_stack
                .pop()
                .expect("the root prop id queue cannot be empty while work_queue is not empty");

            while let Some((prop, parent_prop_id, fbrv)) = work_queue.pop() {
                if let Some(value) = fbrv.value() {
                    if root_id == parent_prop_id {
                        let write_location = match properties.pointer_mut(&json_pointer) {
                            Some(write_location) => write_location,
                            None => {
                                return Err(ComponentError::BadJsonPointer(
                                    json_pointer.clone(),
                                    properties.to_string(),
                                ))
                            }
                        };
                        let next_json_pointer = if write_location.is_object() {
                            write_location
                                .as_object_mut()
                                .unwrap()
                                .insert(prop.name().to_string(), value.clone());
                            format!("{}/{}", json_pointer, prop.name())
                        } else if write_location.is_array() {
                            // This is wrong - we need to check the fbrv for what the
                            // actual index should be. Tomorrows problem. -- Adam
                            let array = write_location.as_array_mut().unwrap();
                            array.push(value.clone());
                            format!("{}/{}", json_pointer, array.len())
                        } else {
                            // Note: this shouldn't ever actually get used.
                            json_pointer.to_string()
                        };
                        match prop.kind() {
                            &PropKind::Object | &PropKind::Array | &PropKind::Map => {
                                root_stack.push((Some(*prop.id()), next_json_pointer));
                            }
                            _ => {}
                        }
                    } else {
                        unprocessed.push((prop, parent_prop_id, fbrv));
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
