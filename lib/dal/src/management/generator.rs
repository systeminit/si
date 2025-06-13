use std::collections::{
    HashMap,
    HashSet,
    VecDeque,
};

use si_frontend_types::RawGeometry;
use si_id::{
    ComponentId,
    ViewId,
};

use super::{
    ManagementCreateOperations,
    ManagementResult,
};
use crate::{
    AttributeValue,
    Component,
    DalContext,
    InputSocket,
    OutputSocket,
    Prop,
    PropKind,
    management::{
        IGNORE_PATHS,
        ManagementConnection,
        ManagementCreateGeometry,
        ManagementCreateOperation,
        ManagementGeometry,
        SocketRef,
    },
    prop::PropPath,
};

pub async fn generate_template(
    ctx: &DalContext,
    view_id: ViewId,
    component_ids: &[ComponentId],
) -> ManagementResult<ManagementCreateOperations> {
    #[derive(Debug, Clone)]
    struct ConnectionInfo {
        from_socket_name: String,
        to_component_id: ComponentId,
        to_socket_name: String,
    }

    #[derive(Debug, Clone)]
    struct ComponentInfo {
        component_id: ComponentId,
        component: Component,
        kind: String,
        placeholder: String,
    }

    // We have to gather up a bunch of data here and make two passes over the components
    let mut components = Vec::new();
    let mut outgoing_connections: HashMap<ComponentId, Vec<ConnectionInfo>> = HashMap::new();
    let mut geometries = HashMap::new();
    let mut placeholders_by_component_id = HashMap::new();
    let mut placeholder_set = HashSet::new();
    let mut input_socket_names = HashMap::new();
    let mut output_socket_names = HashMap::new();
    let mut schema_names = HashMap::new();

    for &component_id in component_ids {
        let component = Component::get_by_id(ctx, component_id).await?;
        let schema = Component::schema_for_component_id(ctx, component_id).await?;
        let schema_name = schema_names
            .entry(schema.id())
            .or_insert_with(|| schema.name().to_string())
            .to_owned();

        let mut geometry = component.geometry(ctx, view_id).await?.into_raw();

        // We want to be sure that frames, and only frames, always have a height/width, while
        // components never have one.
        let component_type = component.get_type(ctx).await?;
        if component_type.is_frame() && geometry.width.zip(geometry.height).is_none() {
            geometry.width = Some(500);
            geometry.height = Some(500);
        } else if !component_type.is_frame()
            && (geometry.width.is_some() || geometry.height.is_some())
        {
            geometry.width = None;
            geometry.height = None;
        }

        for incoming_connection in component.incoming_connections(ctx).await? {
            let to_socket_name = match input_socket_names
                .get(&incoming_connection.to_input_socket_id)
                .cloned()
            {
                Some(name) => name,
                None => {
                    let socket =
                        InputSocket::get_by_id(ctx, incoming_connection.to_input_socket_id).await?;
                    let name = socket.name();
                    input_socket_names
                        .insert(incoming_connection.to_input_socket_id, name.to_string());
                    name.to_string()
                }
            };

            let from_socket_name = match output_socket_names
                .get(&incoming_connection.from_output_socket_id)
                .cloned()
            {
                Some(name) => name,
                None => {
                    let socket =
                        OutputSocket::get_by_id(ctx, incoming_connection.from_output_socket_id)
                            .await?;
                    let name = socket.name();
                    output_socket_names
                        .insert(incoming_connection.from_output_socket_id, name.to_string());
                    name.to_string()
                }
            };

            let outgoing = ConnectionInfo {
                from_socket_name,
                to_socket_name,
                to_component_id: incoming_connection.to_component_id,
            };

            outgoing_connections
                .entry(incoming_connection.from_component_id)
                .and_modify(|conns| conns.push(outgoing.clone()))
                .or_insert_with(|| vec![outgoing]);
        }

        let name = component.name(ctx).await?;
        let placeholder = make_placeholder(&name, &placeholder_set).await;
        placeholder_set.insert(placeholder.clone());
        placeholders_by_component_id.insert(component_id, placeholder.clone());

        components.push(ComponentInfo {
            component_id,
            component,
            kind: schema_name,
            placeholder,
        });
        geometries.insert(component_id, geometry);
    }

    let (origin_x, origin_y) = calculate_top_and_center(&geometries).await;

    let mut creates: HashMap<String, ManagementCreateOperation> = HashMap::new();

    for ComponentInfo {
        component_id,
        component,
        kind,
        placeholder,
    } in components
    {
        // get parentage
        let parent = component
            .parent(ctx)
            .await?
            .and_then(|parent_id| placeholders_by_component_id.get(&parent_id).cloned());

        let mut connections = vec![];
        if let Some(conns) = outgoing_connections.remove(&component_id) {
            for conn in conns {
                if let Some(to_placeholder) =
                    placeholders_by_component_id.get(&conn.to_component_id)
                {
                    connections.push(ManagementConnection::Output {
                        from: conn.from_socket_name,
                        to: SocketRef {
                            component: to_placeholder.to_owned(),
                            socket: conn.to_socket_name,
                        },
                    })
                }
            }
        }

        let geometry: Option<ManagementGeometry> =
            geometries.get(&component_id).cloned().map(|mut geo| {
                geo.x -= origin_x;
                geo.y -= origin_y;
                geo.into()
            });

        let connect = if connections.is_empty() {
            None
        } else {
            Some(connections)
        };

        let properties = component.view(ctx).await?;
        let properties = if let Some(mut properties) = properties {
            let remove_paths = calculate_paths_to_remove(ctx, component.id(), &properties).await?;
            for path in remove_paths {
                let path_as_refs: Vec<_> = path.iter().skip(1).map(|s| s.as_str()).collect();
                remove_value_at_path(&mut properties, &path_as_refs);
            }

            for remove_path in IGNORE_PATHS {
                let path_as_refs: Vec<_> = remove_path.iter().skip(1).copied().collect();
                remove_value_at_path(&mut properties, &path_as_refs);
            }

            Some(properties)
        } else {
            None
        };

        let create = ManagementCreateOperation {
            kind: Some(kind),
            properties,
            attributes: None,
            geometry: geometry.map(ManagementCreateGeometry::CurrentView),
            connect,
            parent,
        };

        creates.insert(placeholder, create);
    }

    Ok(creates)
}

async fn make_placeholder(name: &str, placeholders: &HashSet<String>) -> String {
    let mut cursor = name.to_string();
    loop {
        if !placeholders.contains(&cursor) {
            return cursor;
        }

        let mut whitespace_split = cursor.rsplitn(2, ' ');
        cursor = match whitespace_split
            .next()
            .and_then(|last_split| last_split.parse::<i32>().ok())
            .zip(whitespace_split.next())
        {
            Some((number, before_split)) => {
                if number > 0 {
                    format!("{before_split} {}", number.wrapping_add(1))
                } else {
                    format!("{before_split} {number} 2")
                }
            }
            None => format!("{cursor} 2"),
        };

        tokio::task::yield_now().await;
    }
}

const RESOURCE_ID: &[&str] = &["root", "si", "resourceId"];

async fn calculate_paths_to_remove(
    ctx: &DalContext,
    component_id: ComponentId,
    properties: &serde_json::Value,
) -> ManagementResult<Vec<Vec<String>>> {
    let variant_id = Component::schema_variant_id(ctx, component_id).await?;

    // walk the properties serde_json::Value object without recursion
    let mut work_queue = VecDeque::new();
    work_queue.push_back((vec!["root".to_string()], properties));

    // Never copy resource id into the template
    let mut result = vec![RESOURCE_ID.iter().map(ToString::to_string).collect()];

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
            result.push(path_as_refs.iter().map(|&s| s.to_string()).collect());
            continue;
        }

        // remove the value if it matches the default. This ensures default value changes from prop
        // updates are propagated
        if Prop::default_value(ctx, prop_id).await?.as_ref() == Some(current_val) {
            result.push(path_as_refs.iter().map(|&s| s.to_string()).collect());
            continue;
        }

        let prop = Prop::get_by_id(ctx, prop_id).await?;

        match prop.kind {
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
                let map_children =
                    AttributeValue::map_children(ctx, path_attribute_value_id).await?;

                for (key, child_id) in map_children {
                    if AttributeValue::is_set_by_dependent_function(ctx, child_id).await? {
                        let mut path: Vec<String> =
                            path_as_refs.iter().map(|&s| s.to_string()).collect();
                        path.push(key);
                        result.push(path);
                    }
                }
            }
            _ => {}
        }
    }

    Ok(result)
}

pub async fn calculate_top_and_center(
    geometries: &HashMap<ComponentId, RawGeometry>,
) -> (isize, isize) {
    let mut topmost: Option<isize> = None;
    let mut leftmost: Option<isize> = None;
    let mut rightmost: Option<isize> = None;

    for geometry in geometries.values() {
        let current_topmost = topmost.unwrap_or(geometry.y);
        if geometry.y <= current_topmost {
            topmost.replace(geometry.y);
        }

        let x = geometry.x - (geometry.width.unwrap_or(0) / 2);

        let current_leftmost = leftmost.unwrap_or(x);
        if x <= current_leftmost {
            leftmost.replace(x);
        }

        let component_right_edge = geometry.x
            + (geometry
                .width
                .map(|width| if width > 200 { width - 200 } else { 0 })
                .unwrap_or(0)
                / 2);

        let current_rightmost = rightmost.unwrap_or(component_right_edge);
        if component_right_edge >= current_rightmost {
            rightmost.replace(component_right_edge);
        }

        tokio::task::yield_now().await;
    }

    let Some((topmost, (leftmost, rightmost))) = topmost.zip(leftmost.zip(rightmost)) else {
        return (0, 0);
    };

    (leftmost + ((rightmost - leftmost).abs() / 2), topmost - 500)
}

fn remove_value_at_path(from: &mut serde_json::Value, remove_path: &[&str]) {
    if let Some(serde_json::Value::Object(obj)) = remove_path
        .iter()
        .take(remove_path.len() - 1)
        .try_fold(from, |val, path_part| match *val {
            serde_json::Value::Object(ref mut obj) => obj.get_mut(*path_part),
            _ => None,
        })
    {
        if let Some(&key) = remove_path.iter().last() {
            obj.remove_entry(key);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn removes_value_at_path() {
        let mut value = serde_json::json!({
            "a": { "b": { "c": "d", "e": { "f": "g"}}}
        });

        let mut value_clone = value.clone();

        remove_value_at_path(&mut value_clone, &["a", "b", "e", "f"]);

        assert_eq!(
            serde_json::json!({
                "a": { "b": { "c": "d", "e": {}}}
            }),
            value_clone
        );

        remove_value_at_path(&mut value, &["a", "b", "c"]);
        assert_eq!(
            serde_json::json!({
                "a": { "b": { "e": { "f": "g"}}}
            }),
            value
        );
    }

    #[tokio::test]
    async fn makes_placeholders() {
        let mut placeholder_set = HashSet::new();
        let base_name = "a bcd ef g -10";
        for i in 1..100 {
            let placeholder = make_placeholder(base_name, &placeholder_set).await;
            assert!(!placeholder_set.contains(&placeholder));
            if i == 1 {
                assert_eq!(base_name, placeholder.as_str());
            } else {
                assert_eq!(format!("{base_name} {i}"), placeholder);
            }
            placeholder_set.insert(placeholder);
        }
    }
}
