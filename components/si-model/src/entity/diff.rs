use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::Entity;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum DiffEntry {
    Edit(Edit),
    Add(Add),
    Remove(Remove),
    RepeatedSize(RepeatedSize),
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Edit {
    pub path: Vec<String>,
    pub before: serde_json::Value,
    pub after: serde_json::Value,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Add {
    pub path: Vec<String>,
    pub after: serde_json::Value,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Remove {
    pub path: Vec<String>,
    pub before: serde_json::Value,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RepeatedSize {
    pub path: Vec<String>,
    pub size: usize,
}

#[derive(Error, Debug)]
pub enum DiffError {
    #[error("serde json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("expected a JSON object, but did not find one; bug!")]
    ObjectError,
}

pub type DiffResult<T> = Result<T, DiffError>;

pub type Diffs = Vec<DiffEntry>;

// TODO: We need to turn the registry data into pure data, and we need to have it
// available to SDF. The core of the application is written in rust now, not in
// typescript. It's a bit of a trip. For now, we hard code our only code property.
pub fn diff_for_props(lhs: &Entity, rhs: &Entity) -> DiffResult<Diffs> {
    let diffs = diff(lhs, rhs)?;
    let result = diffs
        .into_iter()
        .filter(|de| match de {
            DiffEntry::Add(d) => {
                (d.path.starts_with(&["name".to_string()])
                    || d.path.starts_with(&["properties".to_string()]))
                    && !d.path.contains(&"kubernetesObjectYaml".to_string())
            }
            DiffEntry::Remove(d) => {
                (d.path.starts_with(&["name".to_string()])
                    || d.path.starts_with(&["properties".to_string()]))
                    && !d.path.contains(&"kubernetesObjectYaml".to_string())
            }
            DiffEntry::Edit(d) => {
                (d.path.starts_with(&["name".to_string()])
                    || d.path.starts_with(&["properties".to_string()]))
                    && !d.path.contains(&"kubernetesObjectYaml".to_string())
            }
            DiffEntry::RepeatedSize(d) => d.path.starts_with(&["properties".to_string()]),
        })
        .collect();
    Ok(result)
}

pub fn diff(lhs: &Entity, rhs: &Entity) -> DiffResult<Diffs> {
    let mut diffs = Vec::new();

    let lhs_json = serde_json::to_value(lhs)?;
    let rhs_json = serde_json::to_value(rhs)?;

    if lhs_json == rhs_json {
        return Ok(diffs);
    }

    let lhs_json_object = lhs_json.as_object().ok_or(DiffError::ObjectError)?;
    let rhs_json_object = rhs_json.as_object().ok_or(DiffError::ObjectError)?;

    // Walk the right hand side - each change is a diff.
    diff_object(&mut diffs, vec![], Some(&lhs_json_object), &rhs_json_object);
    // Walk the left hand side - any missing properties on the right are removals.
    removal_diff_object(&mut diffs, vec![], &lhs_json_object, Some(&rhs_json_object));

    Ok(diffs)
}

pub fn diff_object(
    diffs: &mut Diffs,
    initial_path: Vec<String>,
    lhs_object_option: Option<&serde_json::map::Map<String, serde_json::Value>>,
    rhs_object: &serde_json::map::Map<String, serde_json::Value>,
) {
    for (rhs_key, rhs_value) in rhs_object.iter() {
        let lhs_value_option = lhs_object_option.and_then(|lhs_object| lhs_object.get(rhs_key));
        let mut path = initial_path.clone();
        path.push(rhs_key.clone());
        if rhs_value.is_i64()
            || rhs_value.is_u64()
            || rhs_value.is_f64()
            || rhs_value.is_null()
            || rhs_value.is_string()
            || rhs_value.is_boolean()
            || rhs_value.is_number()
        {
            diff_scalar(diffs, path, lhs_value_option, rhs_value);
        } else if rhs_value.is_object() {
            let rhs_value_object = rhs_value.as_object().unwrap();
            let lhs_value_object = lhs_value_option.and_then(|lhs_object| lhs_object.as_object());
            diff_object(diffs, path, lhs_value_object, rhs_value_object);
        } else if rhs_value.is_array() {
            let rhs_value_array = rhs_value.as_array().unwrap();
            let lhs_value_array = lhs_value_option.and_then(|lhs_object| lhs_object.as_array());
            diff_array(diffs, path, lhs_value_array, rhs_value_array);
        }
    }
}

pub fn diff_array(
    diffs: &mut Diffs,
    initial_path: Vec<String>,
    lhs_array_option: Option<&Vec<serde_json::Value>>,
    rhs_array: &Vec<serde_json::Value>,
) {
    if lhs_array_option.is_none() {
        diffs.push(DiffEntry::RepeatedSize(RepeatedSize {
            path: initial_path.clone(),
            size: rhs_array.len(),
        }));
    } else {
        let lhs_array =
            lhs_array_option.expect("lhs_array_option is none but cannot unwrap - bug!");
        if lhs_array.len() != rhs_array.len() {
            diffs.push(DiffEntry::RepeatedSize(RepeatedSize {
                path: initial_path.clone(),
                size: rhs_array.len(),
            }));
        }
    }
    for (rhs_index, rhs_value) in rhs_array.iter().enumerate() {
        let lhs_value_option = lhs_array_option.and_then(|lhs_array| lhs_array.get(rhs_index));
        let mut path = initial_path.clone();
        path.push(format!("{}", rhs_index));
        if rhs_value.is_i64()
            || rhs_value.is_u64()
            || rhs_value.is_f64()
            || rhs_value.is_null()
            || rhs_value.is_string()
            || rhs_value.is_boolean()
            || rhs_value.is_number()
        {
            diff_scalar(diffs, path, lhs_value_option, rhs_value);
        } else if rhs_value.is_object() {
            let rhs_value_object = rhs_value.as_object().unwrap();
            let lhs_value_object = lhs_value_option.and_then(|lhs_object| lhs_object.as_object());
            diff_object(diffs, path, lhs_value_object, rhs_value_object);
        } else if rhs_value.is_array() {
            let rhs_value_array = rhs_value.as_array().unwrap();
            let lhs_value_array = lhs_value_option.and_then(|lhs_object| lhs_object.as_array());
            diff_array(diffs, path, lhs_value_array, rhs_value_array);
        }
    }
}

pub fn diff_scalar(
    diffs: &mut Diffs,
    path: Vec<String>,
    lhs_value_option: Option<&serde_json::Value>,
    rhs_value: &serde_json::Value,
) {
    match lhs_value_option {
        Some(lhs_value) => {
            if lhs_value != rhs_value {
                if rhs_value.is_null() {
                    // This is a removal event, and we don't allow
                    // null values for fields that have one? Lets see
                    // how that plays out in practice.
                    return;
                }
                diffs.push(DiffEntry::Edit(Edit {
                    path: path.clone(),
                    before: lhs_value.clone(),
                    after: rhs_value.clone(),
                }));
            }
        }
        None => {
            diffs.push(DiffEntry::Add(Add {
                path: path.clone(),
                after: rhs_value.clone(),
            }));
        }
    }
}

pub fn removal_diff_object(
    diffs: &mut Diffs,
    initial_path: Vec<String>,
    lhs_object: &serde_json::map::Map<String, serde_json::Value>,
    rhs_object_option: Option<&serde_json::map::Map<String, serde_json::Value>>,
) {
    if rhs_object_option.is_none() {
        diffs.push(DiffEntry::Remove(Remove {
            path: initial_path.clone(),
            before: serde_json::Value::Object(lhs_object.clone()),
        }));
        return;
    }
    for (lhs_key, lhs_value) in lhs_object.iter() {
        let rhs_value_option = rhs_object_option.and_then(|rhs_object| rhs_object.get(lhs_key));
        let mut path = initial_path.clone();
        path.push(lhs_key.clone());
        if lhs_value.is_i64()
            || lhs_value.is_u64()
            || lhs_value.is_f64()
            || lhs_value.is_null()
            || lhs_value.is_string()
            || lhs_value.is_boolean()
            || lhs_value.is_number()
        {
            if rhs_value_option.is_none() {
                diffs.push(DiffEntry::Remove(Remove {
                    path: initial_path.clone(),
                    before: lhs_value.clone(),
                }));
            }
        } else if lhs_value.is_object() {
            let lhs_value_object = lhs_value.as_object().unwrap();
            let rhs_value_object = rhs_value_option.and_then(|rhs_object| rhs_object.as_object());
            removal_diff_object(diffs, path, lhs_value_object, rhs_value_object);
        } else if lhs_value.is_array() {
            let lhs_value_array = lhs_value.as_array().unwrap();
            let rhs_value_array = rhs_value_option.and_then(|rhs_object| rhs_object.as_array());
            removal_diff_array(diffs, path, lhs_value_array, rhs_value_array);
        }
    }
}

pub fn removal_diff_array(
    diffs: &mut Diffs,
    initial_path: Vec<String>,
    lhs_array: &Vec<serde_json::Value>,
    rhs_array_option: Option<&Vec<serde_json::Value>>,
) {
    if rhs_array_option.is_none() {
        diffs.push(DiffEntry::Remove(Remove {
            path: initial_path.clone(),
            before: serde_json::Value::Array(lhs_array.clone()),
        }));
        return;
    }
    for (lhs_index, lhs_value) in lhs_array.iter().enumerate() {
        let rhs_value_option = rhs_array_option.and_then(|rhs_array| rhs_array.get(lhs_index));
        let mut path = initial_path.clone();
        path.push(format!("{}", lhs_index));
        if lhs_value.is_i64()
            || lhs_value.is_u64()
            || lhs_value.is_f64()
            || lhs_value.is_null()
            || lhs_value.is_string()
            || lhs_value.is_boolean()
            || lhs_value.is_number()
        {
            if rhs_value_option.is_none() {
                diffs.push(DiffEntry::Remove(Remove {
                    path: initial_path.clone(),
                    before: lhs_value.clone(),
                }));
            }
        } else if lhs_value.is_object() {
            let lhs_value_object = lhs_value.as_object().unwrap();
            let rhs_value_object = rhs_value_option.and_then(|rhs_object| rhs_object.as_object());
            removal_diff_object(diffs, path, lhs_value_object, rhs_value_object);
        } else if lhs_value.is_array() {
            let lhs_value_array = lhs_value.as_array().unwrap();
            let rhs_value_array = rhs_value_option.and_then(|rhs_object| rhs_object.as_array());
            removal_diff_array(diffs, path, lhs_value_array, rhs_value_array);
        }
    }
}
