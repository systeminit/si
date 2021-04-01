// This is an implementation of the logic from lodash's 'get' method.
//
// It allows a path separated list of items to retrieve from a serde_json::Value.
//
// It does not support defaults - yet!

use thiserror::Error;

#[derive(Error, Debug)]
pub enum LodashError {
    #[error("initial get must be on a serde_json::Value::Object")]
    InvalidFirstObject,
    #[error("array value requires numeric index")]
    NumericIndex(#[from] std::num::ParseIntError),
}

pub type LodashResult<T> = Result<T, LodashError>;

pub fn get(
    json: &serde_json::Value,
    path: &Vec<impl AsRef<str>>,
) -> LodashResult<Option<serde_json::Value>> {
    if !json.is_object() {
        return Err(LodashError::InvalidFirstObject);
    }
    if path.is_empty() {
        return Ok(Some(json.clone()));
    }
    let mut current_value = json;
    let list_entry_index = path.len() - 1;
    for (i, entry) in path.into_iter().enumerate() {
        let entry = entry.as_ref();
        let is_last_entry = i == list_entry_index;
        if current_value.is_array() {
            let index: usize = entry.parse::<usize>()?;
            let value = current_value.as_array().unwrap().get(index);
            if is_last_entry {
                return Ok(value.map(|v| v.clone()));
            } else {
                match value {
                    Some(v) => current_value = v,
                    None => return Ok(None),
                }
            }
        } else if current_value.is_object() {
            let object = current_value.as_object().unwrap();
            let value = object.get(entry);
            if is_last_entry {
                return Ok(value.map(|v| v.clone()));
            } else {
                match value {
                    Some(v) => current_value = v,
                    None => return Ok(None),
                }
            }
        } else {
            // If it isn't an array or an object - then what was asked for
            // does not exist.
            return Ok(None);
        }
    }
    Ok(None)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn no_object_errors() {
        let json = serde_json::json!["pinky"];
        let result = get(&json, &vec!["making us", "hostile"]);
        assert_eq!(result.is_err(), true, "should error");
    }

    #[test]
    fn top_level_value() {
        let json = serde_json::json![{"pinky":"the brain" }];
        let result = get(&json, &vec!["pinky"])
            .expect("lookup should succeed")
            .expect("lookup should have a value");
        assert_eq!(result, serde_json::json!["the brain"]);
    }

    #[test]
    fn nested_value() {
        let json = serde_json::json![{"pinky":{"what will we do tonight": "same thing we do every night" }}];
        let result = get(&json, &vec!["pinky", "what will we do tonight"])
            .expect("lookup should succeed")
            .expect("lookup should have a value");
        assert_eq!(result, serde_json::json!["same thing we do every night"]);
    }

    #[test]
    fn nested_value_array() {
        let json = serde_json::json![{"pinky":{"what will we do tonight": ["nothing", "same thing we do every night"] }}];
        let result = get(&json, &vec!["pinky", "what will we do tonight", "1"])
            .expect("lookup should succeed")
            .expect("lookup should have a value");
        assert_eq!(result, serde_json::json!["same thing we do every night"]);
    }

    #[test]
    fn nested_value_array_bad_index() {
        let json = serde_json::json![{"pinky":{"what will we do tonight": ["nothing", "same thing we do every night"] }}];
        let result = get(&json, &vec!["pinky", "what will we do tonight", "55"])
            .expect("lookup should succeed");
        assert_eq!(result, None);
    }

    #[test]
    fn nested_value_array_bad_index_not_a_number() {
        let json = serde_json::json![{"pinky":{"what will we do tonight": ["nothing", "same thing we do every night"] }}];
        let result = get(
            &json,
            &vec!["pinky", "what will we do tonight", "fiftyfive"],
        );
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn path_does_not_exist() {
        let json = serde_json::json![{"pinky":{"what will we do tonight": ["nothing", "same thing we do every night"] }}];
        let result = get(&json, &vec!["what", "is", "my", "pinky"]).expect("lookup should succeed");
        assert_eq!(result, None);
    }
}
