use crate::{ValidateResponse, Validator};
pub(crate) use color_eyre::{Result, eyre::eyre};
pub(crate) use serde_json::json;

pub(crate) fn valid_opt(json: &str, value: Option<serde_json::Value>) -> Result<()> {
    let validator: Validator = serde_json::from_str(json)?;
    let response = validator.validate(&value);
    if let ValidateResponse {
        error: Some(error), ..
    } = response
    {
        return Err(eyre!("validation error: {:?}", error));
    }
    Ok(())
}

pub(crate) fn valid(json: &str, value: impl Into<serde_json::Value>) -> Result<()> {
    valid_opt(json, Some(value.into()))
}

pub(crate) fn invalid_opt(json: &str, value: Option<serde_json::Value>) -> Result<()> {
    match valid_opt(json, value.clone()) {
        Ok(_) => Err(eyre!("expected validation error on {:?}", value)),
        Err(_) => Ok(()),
    }
}

pub(crate) fn invalid(json: &str, value: impl Into<serde_json::Value>) -> Result<()> {
    invalid_opt(json, Some(value.into()))
}

pub(crate) fn unsupported(json: &str) -> Result<()> {
    match serde_json::from_str::<Validator>(json) {
        Ok(_) => Err(eyre!("expected schema to be unsupported: {:?}", json)),
        Err(_) => Ok(()),
    }
}

mod any {
    use super::*;

    #[test]
    fn unsupported_basics() -> Result<()> {
        unsupported(r#"{ }"#)?;
        unsupported(r#""string"#)?;
        unsupported(r#"10""#)?;
        unsupported(r#"null"#)?;
        unsupported(r#"[]"#)?;
        Ok(())
    }

    #[test]
    fn unsupported_types() -> Result<()> {
        unsupported(r#"{ "type": "any" }"#)?;
        unsupported(r#"{ "type": "alternatives" }"#)?;
        unsupported(r#"{ "type": "date" }"#)?;
        Ok(())
    }
}

mod boolean {
    use super::*;

    #[test]
    fn type_boolean() -> Result<()> {
        let joi = r#"{ "type": "boolean" }"#;
        valid_opt(joi, None)?;
        valid(joi, true)?;
        valid(joi, "true")?;
        valid(joi, false)?;
        valid(joi, "false")?;
        Ok(())
    }

    #[test]
    fn allow_values() -> Result<()> {
        let joi = r#"{ "type": "boolean" }"#;
        valid(joi, true)?;
        valid(joi, "true")?;
        valid(joi, false)?;
        valid(joi, "false")?;

        let joi = r#"{ "type": "boolean", "allow": [false] }"#;
        valid(joi, true)?;
        valid(joi, "true")?;
        valid(joi, false)?;
        valid(joi, "false")?;

        Ok(())
    }

    #[test]
    fn valid_values() -> Result<()> {
        let joi = r#"{ "type": "boolean", "flags": { "only": true } }"#;
        valid(joi, true)?;
        valid(joi, "true")?;
        valid(joi, false)?;
        valid(joi, "false")?;

        let joi = r#"{ "type": "boolean", "flags": { "only": true }, "allow": [] }"#;
        invalid(joi, true)?;
        invalid(joi, "true")?;
        invalid(joi, false)?;
        invalid(joi, "false")?;

        let joi = r#"{ "type": "boolean", "flags": { "only": true }, "allow": [true] }"#;
        valid(joi, true)?;
        valid(joi, "true")?;
        invalid(joi, false)?;
        invalid(joi, "false")?;

        Ok(())
    }

    #[test]
    fn invalid_values() -> Result<()> {
        let joi = r#"{ "type": "boolean", "invalid": [] }"#;
        valid(joi, true)?;
        valid(joi, "true")?;
        valid(joi, false)?;
        valid(joi, "false")?;

        let joi = r#"{ "type": "boolean", "invalid": [false] }"#;
        valid(joi, true)?;
        valid(joi, "true")?;
        invalid(joi, false)?;
        invalid(joi, "false")?;

        let joi = r#"{ "type": "boolean", "allow": [true], "invalid": [false] }"#;
        valid(joi, true)?;
        valid(joi, "true")?;
        invalid(joi, false)?;
        invalid(joi, "false")?;

        let joi = r#"{ "type": "boolean", "flags": { "only": true }, "invalid": [false] }"#;
        valid(joi, true)?;
        valid(joi, "true")?;
        invalid(joi, false)?;
        invalid(joi, "false")?;

        let joi = r#"{ "type": "boolean", "flags": { "only": true }, "allow": [true], "invalid": [false] }"#;
        valid(joi, true)?;
        valid(joi, "true")?;
        invalid(joi, false)?;
        invalid(joi, "false")?;

        Ok(())
    }

    #[test]
    fn required() -> Result<()> {
        let joi = r#"{ "type": "boolean", "flags": { "presence": "required" } }"#;
        valid(joi, true)?;
        valid(joi, "true")?;
        valid(joi, false)?;
        valid(joi, "false")?;
        invalid(joi, json!(null))?;
        invalid_opt(joi, None)?;
        Ok(())
    }

    #[test]
    fn forbidden() -> Result<()> {
        let joi = r#"{ "type": "boolean", "flags": { "presence": "forbidden" } }"#;
        invalid(joi, true)?;
        invalid(joi, "true")?;
        invalid(joi, false)?;
        invalid(joi, "false")?;
        invalid(joi, json!(null))?;
        valid_opt(joi, None)?;
        Ok(())
    }

    #[test]
    fn optional() -> Result<()> {
        let joi = r#"{ "type": "boolean", "flags": { "presence": "optional" } }"#;
        valid(joi, true)?;
        valid(joi, "true")?;
        valid(joi, false)?;
        valid(joi, "false")?;
        invalid(joi, json!(null))?;
        valid_opt(joi, None)?;
        Ok(())
    }

    #[test]
    fn passthrough() -> Result<()> {
        valid(
            r#"{ "type": "boolean", "flags": { "description": "a" } }"#,
            true,
        )?;
        valid(r#"{ "type": "boolean", "examples": [ true ] }"#, true)?;
        valid(
            r#"{ "type": "boolean", "metas": [ { "foo": "bar" } ] }"#,
            true,
        )?;
        valid(r#"{ "type": "boolean", "notes": [ "a", "b" ] }"#, true)?;
        valid(r#"{ "type": "boolean", "tags": [ "a", "b" ] }"#, true)?;
        Ok(())
    }

    #[test]
    fn unsupported_rules() -> Result<()> {
        unsupported(r#"{ "type": "boolean", "rules": [ { "name": "extra" } ] }"#)?;
        Ok(())
    }

    #[test]
    fn unsupported_flags() -> Result<()> {
        unsupported(r#"{ "type": "boolean", "flags": { "sensitive": false } }"#)?;
        unsupported(r#"{ "type": "boolean", "flags": { "sensitive": true } }"#)?;
        unsupported(r#"{ "type": "boolean", "flags": { "extra": true } }"#)?;
        Ok(())
    }

    #[test]
    fn unsupported_basics() -> Result<()> {
        unsupported(r#"{ "type": "boolean", "truthy": ["Y"] }"#)?;
        unsupported(r#"{ "type": "boolean", "falsy": ["Y"] }"#)?;
        unsupported(r#"{ "type": "boolean", "extra": true }"#)?;
        unsupported(r#"{ "type": "boolean", "preferences": {} }"#)?;
        Ok(())
    }
}

mod number {
    use super::{Result, invalid_opt, json, unsupported, valid_opt};
    use std::f64::consts::PI;

    // tests both the float and string version of the value
    fn valid(joi: &str, value: impl Into<f64>) -> Result<()> {
        let value = value.into();
        super::valid(joi, value)?;
        super::valid(joi, value.to_string())?;
        Ok(())
    }

    // tests both the float and string version of the value
    fn invalid(joi: &str, value: impl Into<f64>) -> Result<()> {
        let value = value.into();
        super::invalid(joi, value)?;
        super::invalid(joi, value.to_string())?;
        Ok(())
    }

    #[test]
    fn type_number() -> Result<()> {
        let joi = r#"{ "type": "number" }"#;
        valid_opt(joi, None)?;
        valid(joi, 0)?;
        valid(joi, 10)?;
        valid(joi, PI)?;
        valid(joi, -PI)?;
        super::invalid(joi, "")?;
        super::invalid(joi, "a")?;
        super::invalid(joi, json!([]))?;
        super::invalid(joi, json!(["a"]))?;
        super::invalid(joi, json!({}))?;
        super::invalid(joi, json!({ "a": "b" }))?;
        super::invalid(joi, json!(null))?;
        Ok(())
    }

    #[test]
    fn unsafe_numbers() -> Result<()> {
        // Safe range
        let joi = r#"{ "type": "number" }"#;
        super::valid(joi, 9007199254740991i64)?;
        super::valid(joi, -9007199254740991i64)?;
        valid(joi, 9007199254740991.0f64)?;
        valid(joi, -9007199254740991.0f64)?;
        super::invalid(joi, 9007199254740992i64)?;
        super::invalid(joi, -9007199254740992i64)?;
        invalid(joi, 9007199254740992.0f64)?;
        invalid(joi, -9007199254740992.0f64)?;

        let joi = r#"{ "type": "number", "flags": { "unsafe": false } }"#;
        super::valid(joi, 9007199254740991i64)?;
        super::valid(joi, -9007199254740991i64)?;
        valid(joi, 9007199254740991.0f64)?;
        valid(joi, -9007199254740991.0f64)?;
        super::invalid(joi, 9007199254740992i64)?;
        super::invalid(joi, -9007199254740992i64)?;
        invalid(joi, 9007199254740992.0f64)?;
        invalid(joi, -9007199254740992.0f64)?;

        let joi = r#"{ "type": "number", "flags": { "unsafe": true } }"#;
        super::valid(joi, 9007199254740991i64)?;
        super::valid(joi, -9007199254740991i64)?;
        valid(joi, 9007199254740991.0f64)?;
        valid(joi, -9007199254740991.0f64)?;
        super::valid(joi, 9007199254740992i64)?;
        super::valid(joi, -9007199254740992i64)?;
        valid(joi, 9007199254740992.0f64)?;
        valid(joi, -9007199254740992.0f64)?;

        Ok(())
    }

    #[test]
    fn integer() -> Result<()> {
        let joi = r#"{ "type": "number", "rules": [
            { "name": "integer" }
        ] }"#;
        valid(joi, -100)?;
        valid(joi, 0)?;
        valid(joi, 100)?;
        valid(joi, 0.0f64)?;
        invalid(joi, -0.1)?;
        invalid(joi, -100.5)?;
        invalid(joi, 0.1)?;
        invalid(joi, 100.5)?;
        unsupported(r#"{ "type": "number", "rules": [ { "name": "min", "args": { } } ] }"#)?;
        Ok(())
    }

    #[test]
    fn min() -> Result<()> {
        let joi = r#"{ "type": "number", "rules": [
            { "name": "min", "args": { "limit": 2 } }
        ] }"#;
        invalid(joi, -5)?;
        invalid(joi, 0)?;
        invalid(joi, 1)?;
        valid(joi, 2)?;
        valid(joi, 2.5)?;
        valid(joi, 3)?;
        valid(joi, 4)?;
        valid(joi, 5)?;
        unsupported(r#"{ "type": "number", "rules": [ { "name": "min" } ] }"#)?;
        unsupported(r#"{ "type": "number", "rules": [ { "name": "min", "args": { } } ] }"#)?;
        unsupported(
            r#"{ "type": "number", "rules": [ { "name": "min", "args": { "limit": 2, "extra": true } } ] }"#,
        )?;
        unsupported(
            r#"{ "type": "number", "rules": [ { "name": "min", "args": { "limit": "2" } } ] }"#,
        )?;

        Ok(())
    }

    #[test]
    fn greater() -> Result<()> {
        let joi = r#"{ "type": "number", "rules": [
            { "name": "greater", "args": { "limit": 2 } }
        ] }"#;
        invalid(joi, -5)?;
        invalid(joi, 0)?;
        invalid(joi, 1)?;
        invalid(joi, 2)?;
        valid(joi, 2.5)?;
        valid(joi, 3)?;
        valid(joi, 4)?;
        valid(joi, 5)?;
        unsupported(r#"{ "type": "number", "rules": [ { "name": "greater" } ] }"#)?;
        unsupported(r#"{ "type": "number", "rules": [ { "name": "greater", "args": { } } ] }"#)?;
        unsupported(
            r#"{ "type": "number", "rules": [ { "name": "greater", "args": { "limit": 2, "extra": true } } ] }"#,
        )?;
        unsupported(
            r#"{ "type": "number", "rules": [ { "name": "greater", "args": { "limit": "2" } } ] }"#,
        )?;

        Ok(())
    }

    #[test]
    fn max() -> Result<()> {
        let joi = r#"{ "type": "number", "rules": [
            { "name": "max", "args": { "limit": 3 } }
        ] }"#;
        valid(joi, -5)?;
        valid(joi, 0)?;
        valid(joi, 1)?;
        valid(joi, 2)?;
        valid(joi, 2.5)?;
        valid(joi, 3)?;
        invalid(joi, 4)?;
        invalid(joi, 5)?;
        unsupported(r#"{ "type": "number", "rules": [ { "name": "max" } ] }"#)?;
        unsupported(r#"{ "type": "number", "rules": [ { "name": "max", "args": { } } ] }"#)?;
        unsupported(
            r#"{ "type": "number", "rules": [ { "name": "max", "args": { "limit": 2, "extra": true } } ] }"#,
        )?;
        unsupported(
            r#"{ "type": "number", "rules": [ { "name": "max", "args": { "limit": "2" } } ] }"#,
        )?;

        Ok(())
    }

    #[test]
    fn less() -> Result<()> {
        let joi = r#"{ "type": "number", "rules": [
            { "name": "less", "args": { "limit": 3 } }
        ] }"#;
        valid(joi, -5)?;
        valid(joi, 0)?;
        valid(joi, 1)?;
        valid(joi, 2)?;
        valid(joi, 2.5)?;
        invalid(joi, 3)?;
        invalid(joi, 4)?;
        invalid(joi, 5)?;
        unsupported(r#"{ "type": "number", "rules": [ { "name": "less" } ] }"#)?;
        unsupported(
            r#"{ "type": "number", "rules": [ { "name": "less", "args": { "limit": 2 }, "extra": true } ] }"#,
        )?;
        unsupported(r#"{ "type": "number", "rules": [ { "name": "less", "args": { } } ] }"#)?;
        unsupported(
            r#"{ "type": "number", "rules": [ { "name": "less", "args": { "limit": 2, "extra": true } } ] }"#,
        )?;

        Ok(())
    }

    #[test]
    fn allow_values() -> Result<()> {
        let joi = r#"{ "type": "number" }"#;
        valid(joi, 1)?;
        valid(joi, 2)?;
        valid(joi, 3)?;
        valid(joi, 4)?;
        valid(joi, 5)?;
        valid(joi, 6)?;

        let joi = r#"{ "type": "number", "allow": [1, 2] }"#;
        valid(joi, 1)?;
        valid(joi, 2)?;
        valid(joi, 3)?;
        valid(joi, 4)?;
        valid(joi, 5)?;
        valid(joi, 6)?;

        Ok(())
    }

    #[test]
    fn valid_values() -> Result<()> {
        let joi = r#"{ "type": "number", "flags": { "only": true } }"#;
        valid(joi, 1)?;
        valid(joi, 2)?;
        valid(joi, 3)?;
        valid(joi, 4)?;
        valid(joi, 5)?;
        valid(joi, 6)?;

        let joi = r#"{ "type": "number", "flags": { "only": true }, "allow": [] }"#;
        invalid(joi, 1)?;
        invalid(joi, 2)?;
        invalid(joi, 3)?;
        invalid(joi, 4)?;
        invalid(joi, 5)?;
        invalid(joi, 6)?;

        let joi = r#"{ "type": "number", "flags": { "only": true }, "allow": [1, 2] }"#;
        valid(joi, 1)?;
        valid(joi, 2)?;
        invalid(joi, 3)?;
        invalid(joi, 4)?;
        invalid(joi, 5)?;
        invalid(joi, 6)?;

        Ok(())
    }

    #[test]
    fn invalid_values() -> Result<()> {
        let joi = r#"{ "type": "number", "invalid": [] }"#;
        valid(joi, 1)?;
        valid(joi, 2)?;
        valid(joi, 3)?;
        valid(joi, 4)?;
        valid(joi, 5)?;
        valid(joi, 6)?;

        let joi = r#"{ "type": "number", "invalid": [3, 4] }"#;
        valid(joi, 1)?;
        valid(joi, 2)?;
        invalid(joi, 3)?;
        invalid(joi, 4)?;
        valid(joi, 5)?;
        valid(joi, 6)?;

        let joi = r#"{ "type": "number", "allow": [1, 2], "invalid": [3, 4] }"#;
        valid(joi, 1)?;
        valid(joi, 2)?;
        invalid(joi, 3)?;
        invalid(joi, 4)?;
        valid(joi, 5)?;
        valid(joi, 6)?;

        let joi = r#"{ "type": "number", "flags": { "only": true }, "invalid": [3, 4] }"#;
        valid(joi, 1)?;
        valid(joi, 2)?;
        invalid(joi, 3)?;
        invalid(joi, 4)?;
        valid(joi, 5)?;
        valid(joi, 6)?;

        let joi = r#"{ "type": "number", "flags": { "only": true }, "allow": [1, 2], "invalid": [3, 4] }"#;
        valid(joi, 1)?;
        valid(joi, 2)?;
        invalid(joi, 3)?;
        invalid(joi, 4)?;
        invalid(joi, 5)?;
        invalid(joi, 6)?;

        Ok(())
    }

    #[test]
    fn required() -> Result<()> {
        let joi = r#"{ "type": "number", "flags": { "presence": "required" } }"#;
        valid(joi, 0)?;
        valid(joi, 1)?;
        super::invalid(joi, json!(null))?;
        invalid_opt(joi, None)?;
        Ok(())
    }

    #[test]
    fn forbidden() -> Result<()> {
        let joi = r#"{ "type": "number", "flags": { "presence": "forbidden" } }"#;
        invalid(joi, 0)?;
        invalid(joi, 1)?;
        super::invalid(joi, json!(null))?;
        valid_opt(joi, None)?;
        Ok(())
    }

    #[test]
    fn optional() -> Result<()> {
        let joi = r#"{ "type": "number", "flags": { "presence": "optional" } }"#;
        valid(joi, 0)?;
        valid(joi, 1)?;
        super::invalid(joi, json!(null))?;
        valid_opt(joi, None)?;
        Ok(())
    }

    #[test]
    fn passthrough() -> Result<()> {
        valid(
            r#"{ "type": "number", "flags": { "description": "a" } }"#,
            1,
        )?;
        valid(r#"{ "type": "number", "examples": [ 1, 2 ] }"#, 1)?;
        valid(r#"{ "type": "number", "metas": [ { "foo": "bar" } ] }"#, 1)?;
        valid(r#"{ "type": "number", "notes": [ "a", "b" ] }"#, 1)?;
        valid(r#"{ "type": "number", "tags": [ "a", "b" ] }"#, 1)?;
        Ok(())
    }

    #[test]
    fn unsupported_rules() -> Result<()> {
        unsupported(r#"{ "type": "number", "rules": [ { "name": "multiple" } ] }"#)?;
        unsupported(r#"{ "type": "number", "rules": [ { "name": "port" } ] }"#)?;
        unsupported(r#"{ "type": "number", "rules": [ { "name": "precision" } ] }"#)?;
        unsupported(r#"{ "type": "number", "rules": [ { "name": "sign" } ] }"#)?;
        unsupported(r#"{ "type": "number", "rules": [ { "name": "extra" } ] }"#)?;
        Ok(())
    }

    #[test]
    fn unsupported_flags() -> Result<()> {
        unsupported(r#"{ "type": "number", "flags": { "extra": true } }"#)?;
        Ok(())
    }

    #[test]
    fn unsupported_basics() -> Result<()> {
        unsupported(r#"{ "type": "number", "extra": true }"#)?;
        unsupported(r#"{ "type": "number", "preferences": {} }"#)?;
        Ok(())
    }
}

mod string {
    use super::*;
    use std::f64::consts::PI;

    #[test]
    fn type_string() -> Result<()> {
        let joi = r#"{ "type": "string" }"#;
        valid_opt(joi, None)?;
        invalid(joi, 0)?;
        invalid(joi, 10)?;
        invalid(joi, PI)?;
        invalid(joi, -PI)?;
        valid(joi, "")?;
        valid(joi, "a")?;
        invalid(joi, json!([]))?;
        invalid(joi, json!(["a"]))?;
        invalid(joi, json!({}))?;
        invalid(joi, json!({ "a": "b" }))?;
        invalid(joi, json!(null))?;
        Ok(())
    }

    #[test]
    fn min() -> Result<()> {
        let joi = r#"{ "type": "string", "rules": [
            { "name": "min", "args": { "limit": 2 } }
        ] }"#;
        invalid(joi, "")?;
        invalid(joi, "a")?;
        valid(joi, "ab")?;
        valid(joi, "abc")?;
        valid(joi, "abcd")?;
        valid(joi, "abcde")?;
        unsupported(r#"{ "type": "string", "rules": [ { "name": "min" } ] }"#)?;
        unsupported(r#"{ "type": "string", "rules": [ { "name": "min", "args": { } } ] }"#)?;
        unsupported(
            r#"{ "type": "string", "rules": [ { "name": "min", "args": { "limit": 2, "extra": true } } ] }"#,
        )?;
        unsupported(
            r#"{ "type": "string", "rules": [ { "name": "min", "args": { "limit": "2" } } ] }"#,
        )?;
        Ok(())
    }

    #[test]
    fn max() -> Result<()> {
        let joi = r#"{ "type": "string", "rules": [
            { "name": "max", "args": { "limit": 3 } }
        ] }"#;
        valid(joi, "")?;
        valid(joi, "a")?;
        valid(joi, "ab")?;
        valid(joi, "abc")?;
        invalid(joi, "abcd")?;
        invalid(joi, "abcde")?;
        unsupported(r#"{ "type": "string", "rules": [ { "name": "max" } ] }"#)?;
        unsupported(r#"{ "type": "string", "rules": [ { "name": "max", "args": { } } ] }"#)?;
        unsupported(
            r#"{ "type": "string", "rules": [ { "name": "max", "args": { "limit": 2, "extra": true } } ] }"#,
        )?;
        unsupported(
            r#"{ "type": "string", "rules": [ { "name": "max", "args": { "limit": "2" } } ] }"#,
        )?;
        Ok(())
    }

    #[test]
    fn length() -> Result<()> {
        let joi = r#"{ "type": "string", "rules": [
            { "name": "length", "args": { "limit": 2 } }
        ] }"#;
        invalid(joi, "")?;
        invalid(joi, "a")?;
        valid(joi, "ab")?;
        invalid(joi, "abc")?;
        invalid(joi, "abcd")?;
        invalid(joi, "abcde")?;
        unsupported(r#"{ "type": "string", "rules": [ { "name": "length" } ] }"#)?;
        unsupported(r#"{ "type": "string", "rules": [ { "name": "length", "args": { } } ] }"#)?;
        unsupported(
            r#"{ "type": "string", "rules": [ { "name": "length", "args": { "limit": 2, "extra": true } } ] }"#,
        )?;
        unsupported(
            r#"{ "type": "string", "rules": [ { "name": "length", "args": { "limit": "2" } } ] }"#,
        )?;
        Ok(())
    }

    #[test]
    fn multi() -> Result<()> {
        let joi = r#"{ "type": "string", "flags": { "presence": "required", "only": true }, "rules": [
            { "name": "min", "args": { "limit": 2 } },
            { "name": "max", "args": { "limit": 3 } }
        ] }"#;
        invalid(joi, "")?;
        invalid(joi, "a")?;
        valid(joi, "ab")?;
        valid(joi, "abc")?;
        invalid(joi, "abcd")?;
        invalid(joi, "abcde")?;
        invalid(joi, 10)?;
        invalid(joi, json!(null))?;
        invalid_opt(joi, None)?;
        Ok(())
    }

    #[test]
    fn allow_values() -> Result<()> {
        let joi = r#"{ "type": "string" }"#;
        valid(joi, "a")?;
        valid(joi, "b")?;
        valid(joi, "c")?;
        valid(joi, "d")?;
        valid(joi, "e")?;
        valid(joi, "f")?;

        let joi = r#"{ "type": "string", "allow": ["a", "b"] }"#;
        valid(joi, "a")?;
        valid(joi, "b")?;
        valid(joi, "c")?;
        valid(joi, "d")?;
        valid(joi, "e")?;
        valid(joi, "f")?;

        Ok(())
    }

    #[test]
    fn valid_values() -> Result<()> {
        let joi = r#"{ "type": "string", "flags": { "only": true } }"#;
        valid(joi, "a")?;
        valid(joi, "b")?;
        valid(joi, "c")?;
        valid(joi, "d")?;
        valid(joi, "e")?;
        valid(joi, "f")?;

        let joi = r#"{ "type": "string", "flags": { "only": true }, "allow": [] }"#;
        invalid(joi, "a")?;
        invalid(joi, "b")?;
        invalid(joi, "c")?;
        invalid(joi, "d")?;
        invalid(joi, "e")?;
        invalid(joi, "f")?;

        let joi = r#"{ "type": "string", "flags": { "only": true }, "allow": ["a", "b"] }"#;
        valid(joi, "a")?;
        valid(joi, "b")?;
        invalid(joi, "c")?;
        invalid(joi, "d")?;
        invalid(joi, "e")?;
        invalid(joi, "f")?;

        Ok(())
    }

    #[test]
    fn invalid_values() -> Result<()> {
        let joi = r#"{ "type": "string", "invalid": [] }"#;
        valid(joi, "a")?;
        valid(joi, "b")?;
        valid(joi, "c")?;
        valid(joi, "d")?;
        valid(joi, "e")?;
        valid(joi, "f")?;

        let joi = r#"{ "type": "string", "invalid": ["c", "d"] }"#;
        valid(joi, "a")?;
        valid(joi, "b")?;
        invalid(joi, "c")?;
        invalid(joi, "d")?;
        valid(joi, "e")?;
        valid(joi, "f")?;

        let joi = r#"{ "type": "string", "allow": ["a", "b"], "invalid": ["c", "d"] }"#;
        valid(joi, "a")?;
        valid(joi, "b")?;
        invalid(joi, "c")?;
        invalid(joi, "d")?;
        valid(joi, "e")?;
        valid(joi, "f")?;

        let joi = r#"{ "type": "string", "flags": { "only": true }, "invalid": ["c", "d"] }"#;
        valid(joi, "a")?;
        valid(joi, "b")?;
        invalid(joi, "c")?;
        invalid(joi, "d")?;
        valid(joi, "e")?;
        valid(joi, "f")?;

        let joi = r#"{ "type": "string", "flags": { "only": true }, "allow": ["a", "b"], "invalid": ["c", "d"] }"#;
        valid(joi, "a")?;
        valid(joi, "b")?;
        invalid(joi, "c")?;
        invalid(joi, "d")?;
        invalid(joi, "e")?;
        invalid(joi, "f")?;

        Ok(())
    }

    #[test]
    fn required() -> Result<()> {
        let joi = r#"{ "type": "string", "flags": { "presence": "required" } }"#;
        valid(joi, "")?;
        valid(joi, "a")?;
        invalid(joi, json!(null))?;
        invalid_opt(joi, None)?;
        Ok(())
    }

    #[test]
    fn forbidden() -> Result<()> {
        let joi = r#"{ "type": "string", "flags": { "presence": "forbidden" } }"#;
        invalid(joi, "")?;
        invalid(joi, "a")?;
        invalid(joi, json!(null))?;
        valid_opt(joi, None)?;
        Ok(())
    }

    #[test]
    fn optional() -> Result<()> {
        let joi = r#"{ "type": "string", "flags": { "presence": "optional" } }"#;
        valid(joi, "")?;
        valid(joi, "a")?;
        invalid(joi, json!(null))?;
        valid_opt(joi, None)?;
        Ok(())
    }

    #[test]
    fn passthrough() -> Result<()> {
        valid(
            r#"{ "type": "string", "flags": { "description": "a" } }"#,
            "",
        )?;
        valid(r#"{ "type": "string", "examples": [ "a", "b" ] }"#, "")?;
        valid(r#"{ "type": "string", "metas": [ { "foo": "bar" } ] }"#, "")?;
        valid(r#"{ "type": "string", "notes": [ "a", "b" ] }"#, "")?;
        valid(r#"{ "type": "string", "tags": [ "a", "b" ] }"#, "")?;
        Ok(())
    }

    #[test]
    fn unsupported_rules() -> Result<()> {
        unsupported(r#"{ "type": "string", "rules": [ { "name": "alphanum" } ] }"#)?;
        unsupported(r#"{ "type": "string", "rules": [ { "name": "base64" } ] }"#)?;
        unsupported(r#"{ "type": "string", "rules": [ { "name": "case" } ] }"#)?;
        unsupported(r#"{ "type": "string", "rules": [ { "name": "creditCard" } ] }"#)?;
        unsupported(r#"{ "type": "string", "rules": [ { "name": "dataUri" } ] }"#)?;
        unsupported(r#"{ "type": "string", "rules": [ { "name": "domain" } ] }"#)?;
        unsupported(r#"{ "type": "string", "rules": [ { "name": "email" } ] }"#)?;
        unsupported(r#"{ "type": "string", "rules": [ { "name": "guid" } ] }"#)?;
        unsupported(r#"{ "type": "string", "rules": [ { "name": "hex" } ] }"#)?;
        unsupported(r#"{ "type": "string", "rules": [ { "name": "hostname" } ] }"#)?;
        unsupported(r#"{ "type": "string", "rules": [ { "name": "ip" } ] }"#)?;
        unsupported(r#"{ "type": "string", "rules": [ { "name": "isoDate" } ] }"#)?;
        unsupported(r#"{ "type": "string", "rules": [ { "name": "isoDuration" } ] }"#)?;
        unsupported(r#"{ "type": "string", "rules": [ { "name": "normalize" } ] }"#)?;
        unsupported(r#"{ "type": "string", "rules": [ { "name": "pattern" } ] }"#)?;
        unsupported(r#"{ "type": "string", "rules": [ { "name": "replace" } ] }"#)?;
        unsupported(r#"{ "type": "string", "rules": [ { "name": "token" } ] }"#)?;
        unsupported(r#"{ "type": "string", "rules": [ { "name": "trim" } ] }"#)?;
        unsupported(r#"{ "type": "string", "rules": [ { "name": "truncate" } ] }"#)?;
        unsupported(r#"{ "type": "string", "rules": [ { "name": "uri" } ] }"#)?;
        unsupported(r#"{ "type": "string", "rules": [ { "name": "extra" } ] }"#)?;
        Ok(())
    }

    #[test]
    fn unsupported_flags() -> Result<()> {
        unsupported(r#"{ "type": "string", "flags": { "extra": true } }"#)?;
        unsupported(r#"{ "type": "string", "flags": { "insensitive": true } }"#)?;
        unsupported(r#"{ "type": "string", "flags": { "insensitive": false } }"#)?;
        Ok(())
    }

    #[test]
    fn unsupported_basics() -> Result<()> {
        unsupported(r#"{ "type": "string", "extra": true }"#)?;
        unsupported(r#"{ "type": "string", "preferences": {} }"#)?;
        Ok(())
    }
}
