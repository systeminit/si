use serde_json::{json, Value};
use si_crypto::{
    SensitiveStrings, VeritechDecryptionKey, VeritechDecryptionKeyError, VeritechEncryptionKey,
};
use thiserror::Error;

const MARKER_FIELD: &str = "cycloneEncryptedDataMarker";
const KEY_HASH_FIELD: &str = "keyHash";
const CRYPTED_FIELD: &str = "crypted";

#[derive(Debug, Error)]
pub enum VeritechValueEncryptError {
    #[error("invalid json pointer: {0}")]
    InvalidJSONPointer(String),
    #[error("error serializing to json: {0}")]
    Serialize(#[from] serde_json::Error),
}

#[derive(Debug, Error)]
pub enum VeritechValueDecryptError {
    #[error("object missing crypted field")]
    CryptedFieldMissing,
    #[error("object crypted field value was not a string")]
    CryptedFieldValueNotString,
    #[error("cyclone decryption error: {0}")]
    VeritechDecryption(#[from] VeritechDecryptionKeyError),
    #[error("error deserializing from json: {0}")]
    Deserialize(#[from] serde_json::Error),
    #[error("invalid json pointer: {0}")]
    InvalidJSONPointer(String),
    #[error("object missing key hash field")]
    KeyHashFieldMissing,
    #[error("object key hash field value was not a string")]
    KeyHashFieldValueNotString,
    #[error("object key hash field value does not match provided decryption key")]
    KeyHashNoMatch,
    #[error("object missing marker field")]
    MarkerFieldMissing,
    #[error("object marker field value was not a bool")]
    MarkerFieldValueNotBool,
    #[error("object marker field value was not true")]
    MarkerFieldValueNotTrue,
    #[error("json value was not an object")]
    ValueNotObject,
}

pub fn encrypt_value_tree(
    value: &mut Value,
    encryption_key: &VeritechEncryptionKey,
) -> Result<(), VeritechValueEncryptError> {
    let mut json_pointer_stack = vec!["".to_owned()];

    while let Some(pointer) = json_pointer_stack.pop() {
        match value.pointer_mut(&pointer) {
            Some(value) => match value {
                Value::String(_) => {
                    let encrypted_value = encrypt_value(value, encryption_key)?;
                    *value = encrypted_value;
                }
                Value::Array(array) => {
                    json_pointer_stack.extend(
                        array
                            .iter()
                            .enumerate()
                            .map(|(index, _element)| format!("{pointer}/{index}")),
                    );
                }
                Value::Object(object) => {
                    json_pointer_stack.extend(
                        object
                            .iter()
                            .map(|(key, _value)| format!("{pointer}/{key}")),
                    );
                }
                Value::Null | Value::Bool(_) | Value::Number(_) => {
                    // Nothing to do
                }
            },
            None => return Err(VeritechValueEncryptError::InvalidJSONPointer(pointer)),
        }
    }

    Ok(())
}

pub fn decrypt_value_tree(
    value: &mut Value,
    sensitive_strings: &mut SensitiveStrings,
    decryption_key: &VeritechDecryptionKey,
) -> Result<(), VeritechValueDecryptError> {
    let mut json_pointer_stack = vec!["".to_owned()];

    while let Some(pointer) = json_pointer_stack.pop() {
        match value.pointer_mut(&pointer) {
            Some(value) => match value {
                Value::Array(array) => {
                    json_pointer_stack.extend(
                        array
                            .iter()
                            .enumerate()
                            .map(|(index, _element)| format!("{pointer}/{index}")),
                    );
                }
                Value::Object(_) if is_value_encrypted(value) => {
                    let decrypted_value = decrypt_value(value, decryption_key)?;
                    if let Value::String(sensitive_str) = &decrypted_value {
                        sensitive_strings.insert(sensitive_str);
                    }
                    *value = decrypted_value;
                }
                Value::Object(object) => {
                    json_pointer_stack.extend(
                        object
                            .iter()
                            .map(|(key, _value)| format!("{pointer}/{key}")),
                    );
                }
                Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {
                    // Nothing to do
                }
            },
            None => return Err(VeritechValueDecryptError::InvalidJSONPointer(pointer)),
        }
    }

    Ok(())
}

fn is_value_encrypted(value: &Value) -> bool {
    match value {
        Value::Object(value) => {
            value.get(MARKER_FIELD).is_some()
                && value.get(KEY_HASH_FIELD).is_some()
                && value.get(CRYPTED_FIELD).is_some()
        }
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) | Value::Array(_) => {
            false
        }
    }
}

fn encrypt_value(
    value: &Value,
    encryption_key: &VeritechEncryptionKey,
) -> Result<Value, serde_json::Error> {
    let bytes = serde_json::to_vec(value)?;
    let crypted = encryption_key.encrypt_and_encode(bytes);

    Ok(json!({
        MARKER_FIELD: true,
        KEY_HASH_FIELD: encryption_key.key_hash().to_string(),
        CRYPTED_FIELD: crypted,
    }))
}

fn decrypt_value(
    value: &Value,
    decryption_key: &VeritechDecryptionKey,
) -> Result<Value, VeritechValueDecryptError> {
    // Confirm value is an object
    let value = value
        .as_object()
        .ok_or(VeritechValueDecryptError::ValueNotObject)?;

    // Check marker field is set and is `true`
    if !value
        .get(MARKER_FIELD)
        .ok_or(VeritechValueDecryptError::MarkerFieldMissing)?
        .as_bool()
        .ok_or(VeritechValueDecryptError::MarkerFieldValueNotBool)?
    {
        return Err(VeritechValueDecryptError::MarkerFieldValueNotTrue);
    }
    // Confirm that key hash field value matches hash for provided decryption key
    if value
        .get(KEY_HASH_FIELD)
        .ok_or(VeritechValueDecryptError::KeyHashFieldMissing)?
        .as_str()
        .ok_or(VeritechValueDecryptError::KeyHashFieldValueNotString)?
        != decryption_key.encryption_key_hash_str()
    {
        return Err(VeritechValueDecryptError::KeyHashNoMatch);
    }

    // Decrypt crypted field and deserialize decrypted contents as a JSON value
    let decrypted = {
        let crypted = value
            .get(CRYPTED_FIELD)
            .ok_or(VeritechValueDecryptError::CryptedFieldMissing)?
            .as_str()
            .ok_or(VeritechValueDecryptError::CryptedFieldValueNotString)?;
        let bytes = decryption_key.decode_and_decrypt(crypted)?;
        serde_json::from_slice::<Value>(&bytes)?
    };

    Ok(decrypted)
}

#[cfg(test)]
mod tests {
    use super::*;

    mod is_value_encrypted {
        use super::*;

        #[test]
        fn null() {
            assert!(!is_value_encrypted(&json!(null)));
        }

        #[test]
        fn bool() {
            assert!(!is_value_encrypted(&json!(true)));
            assert!(!is_value_encrypted(&json!(false)));
        }

        #[test]
        fn number() {
            assert!(!is_value_encrypted(&json!(1)));
            assert!(!is_value_encrypted(&json!(0)));
            assert!(!is_value_encrypted(&json!(-3.56)));
        }

        #[test]
        fn string() {
            assert!(!is_value_encrypted(&json!("")));
            assert!(!is_value_encrypted(&json!("false")));
            assert!(!is_value_encrypted(&json!("ponies")));
        }

        #[test]
        fn array() {
            assert!(!is_value_encrypted(&json!([])));
            assert!(!is_value_encrypted(&json!(["nope"])));
            assert!(!is_value_encrypted(&json!([{"one": "two"}])));
        }

        #[test]
        fn regular_object() {
            assert!(!is_value_encrypted(&json!({})));
            assert!(!is_value_encrypted(&json!({"one": "two"})));
        }

        #[test]
        fn encrypted_object() {
            assert!(is_value_encrypted(&json!({
                "cycloneEncryptedDataMarker": true,
                "keyHash": "abc123",
                "crypted": "sssshhh",
            })));
        }

        #[test]
        fn encrypted_object_missing_marker() {
            assert!(!is_value_encrypted(&json!({
                "keyHash": "abc123",
                "crypted": "sssshhh",
            })));
        }

        #[test]
        fn encrypted_object_missing_key_hash() {
            assert!(!is_value_encrypted(&json!({
                "cycloneEncryptedDataMarker": true,
                "crypted": "sssshhh",
            })));
        }
    }

    mod encrypt_value {
        use si_crypto::VeritechKeyPair;

        use super::*;

        #[test]
        fn string_round_trip() {
            let (encryption_key, decryption_key) = VeritechKeyPair::create();

            let message = json!("Telling the Bees");
            let encrypted_value =
                encrypt_value(&message, &encryption_key).expect("failed to encrypt value");
            let decrypted_message = decrypt_value(&encrypted_value, &decryption_key)
                .expect("failed to decrypt message");

            assert_eq!(message, decrypted_message);
        }

        #[test]
        fn obj_round_trip() {
            let (encryption_key, decryption_key) = VeritechKeyPair::create();

            let message = json!({
                "artist": "Dream Theater",
                "album": "Train of Thought",
                "songTitle": "As I Am",
                "styles": [
                    "Art Rock",
                    "Heavy Metal",
                    "Neo-Prog",
                    "Progressive Metal",
                ],
            });
            let encrypted_value =
                encrypt_value(&message, &encryption_key).expect("failed to encrypt value");
            let decrypted_message = decrypt_value(&encrypted_value, &decryption_key)
                .expect("failed to decrypt message");

            assert_eq!(message, decrypted_message);
        }
    }

    mod decrypt_value {
        use si_crypto::VeritechKeyPair;

        use super::*;

        fn encrypted(encryption_key: &VeritechEncryptionKey) -> Value {
            encrypt_value(&json!("my-secret"), encryption_key).expect("failed to encrypt value")
        }

        #[test]
        fn value_not_object() {
            let (_encryption_key, decryption_key) = VeritechKeyPair::create();

            assert!(matches!(
                decrypt_value(&json!("uh oh"), &decryption_key),
                Err(VeritechValueDecryptError::ValueNotObject),
            ));
        }

        #[test]
        fn marker_field_missing() {
            let (encryption_key, decryption_key) = VeritechKeyPair::create();

            let mut encrypted = encrypted(&encryption_key);
            encrypted
                .as_object_mut()
                .expect("value is not an object")
                .remove(MARKER_FIELD);

            assert!(matches!(
                decrypt_value(&encrypted, &decryption_key),
                Err(VeritechValueDecryptError::MarkerFieldMissing),
            ));
        }

        #[test]
        fn marker_field_value_not_bool() {
            let (encryption_key, decryption_key) = VeritechKeyPair::create();

            let mut encrypted = encrypted(&encryption_key);
            *encrypted
                .as_object_mut()
                .expect("value is not an object")
                .get_mut(MARKER_FIELD)
                .expect("missing field") = json!("not a bool");

            assert!(matches!(
                decrypt_value(&encrypted, &decryption_key),
                Err(VeritechValueDecryptError::MarkerFieldValueNotBool),
            ));
        }

        #[test]
        fn marker_field_value_not_true() {
            let (encryption_key, decryption_key) = VeritechKeyPair::create();

            let mut encrypted = encrypted(&encryption_key);
            *encrypted
                .as_object_mut()
                .expect("value is not an object")
                .get_mut(MARKER_FIELD)
                .expect("missing field") = json!(false);

            assert!(matches!(
                decrypt_value(&encrypted, &decryption_key),
                Err(VeritechValueDecryptError::MarkerFieldValueNotTrue),
            ));
        }

        #[test]
        fn key_hash_field_missing() {
            let (encryption_key, decryption_key) = VeritechKeyPair::create();

            let mut encrypted = encrypted(&encryption_key);
            encrypted
                .as_object_mut()
                .expect("value is not an object")
                .remove(KEY_HASH_FIELD);

            assert!(matches!(
                decrypt_value(&encrypted, &decryption_key),
                Err(VeritechValueDecryptError::KeyHashFieldMissing),
            ));
        }

        #[test]
        fn key_hash_field_value_not_str() {
            let (encryption_key, decryption_key) = VeritechKeyPair::create();

            let mut encrypted = encrypted(&encryption_key);
            *encrypted
                .as_object_mut()
                .expect("value is not an object")
                .get_mut(KEY_HASH_FIELD)
                .expect("missing field") = json!(true);

            assert!(matches!(
                decrypt_value(&encrypted, &decryption_key),
                Err(VeritechValueDecryptError::KeyHashFieldValueNotString),
            ));
        }

        #[test]
        fn key_hash_field_value_no_match() {
            let (encryption_key, decryption_key) = VeritechKeyPair::create();
            let (wrong_encryption_key, _) = VeritechKeyPair::create();

            let mut encrypted = encrypted(&encryption_key);
            *encrypted
                .as_object_mut()
                .expect("value is not an object")
                .get_mut(KEY_HASH_FIELD)
                .expect("missing field") = json!(wrong_encryption_key.key_hash().to_string());

            assert!(matches!(
                decrypt_value(&encrypted, &decryption_key),
                Err(VeritechValueDecryptError::KeyHashNoMatch),
            ));
        }

        #[test]
        fn crypted_field_missing() {
            let (encryption_key, decryption_key) = VeritechKeyPair::create();

            let mut encrypted = encrypted(&encryption_key);
            encrypted
                .as_object_mut()
                .expect("value is not an object")
                .remove(CRYPTED_FIELD);

            assert!(matches!(
                decrypt_value(&encrypted, &decryption_key),
                Err(VeritechValueDecryptError::CryptedFieldMissing),
            ));
        }

        #[test]
        fn crypted_field_value_not_str() {
            let (encryption_key, decryption_key) = VeritechKeyPair::create();

            let mut encrypted = encrypted(&encryption_key);
            *encrypted
                .as_object_mut()
                .expect("value is not an object")
                .get_mut(CRYPTED_FIELD)
                .expect("missing field") = json!(true);

            assert!(matches!(
                decrypt_value(&encrypted, &decryption_key),
                Err(VeritechValueDecryptError::CryptedFieldValueNotString),
            ));
        }

        #[test]
        fn crypted_field_value_not_properly_encoded() {
            let (encryption_key, decryption_key) = VeritechKeyPair::create();

            let mut encrypted = encrypted(&encryption_key);
            *encrypted
                .as_object_mut()
                .expect("value is not an object")
                .get_mut(CRYPTED_FIELD)
                .expect("missing field") = json!("uh oh");

            assert!(matches!(
                decrypt_value(&encrypted, &decryption_key),
                Err(VeritechValueDecryptError::VeritechDecryption(_)),
            ));
        }

        #[test]
        fn wrong_decryption_key() {
            let (encryption_key, _) = VeritechKeyPair::create();
            let (_, wrong_decryption_key) = VeritechKeyPair::create();

            let encrypted = encrypted(&encryption_key);

            assert!(matches!(
                dbg!(decrypt_value(&encrypted, &wrong_decryption_key)),
                Err(VeritechValueDecryptError::KeyHashNoMatch),
            ));
        }
    }

    mod encrypt_value_tree {
        use std::collections::HashSet;

        use si_crypto::VeritechKeyPair;
        use si_std::SensitiveString;

        use super::*;

        #[test]
        fn object_with_string_field_values_round_trip() {
            let (encryption_key, decryption_key) = VeritechKeyPair::create();
            let mut sensitive_strings = SensitiveStrings::default();

            let mut secret = json!({
                "username": "Mike Portnoy",
                "password": "Drummer",
            });
            let expected_secret = secret.clone();

            encrypt_value_tree(&mut secret, &encryption_key).expect("failed to encrypt tree");
            decrypt_value_tree(&mut secret, &mut sensitive_strings, &decryption_key)
                .expect("failed to decrypt tree");

            assert_eq!(expected_secret, secret);

            let strings: HashSet<SensitiveString> = sensitive_strings.into();
            assert_eq!(2, strings.len());
            assert!(strings.contains(&"Mike Portnoy".into()));
            assert!(strings.contains(&"Drummer".into()));
        }

        #[test]
        fn nested_object_round_trip() {
            let (encryption_key, decryption_key) = VeritechKeyPair::create();
            let mut sensitive_strings = SensitiveStrings::default();

            let mut secret = json!({
                "strValue": "i-am-a-string",
                "boolValue": false,
                "nestedObjValue": {
                    "again": {
                        "stuff": null,
                        "token": "token-1-2-3",
                        "groceries": ["milk", "cheese", "brains"],
                    },
                },
                "arrValue": [1, 2, 3],
                "nullValue": null,
                "numberValue": -1,
            });
            let expected_secret = secret.clone();

            encrypt_value_tree(&mut secret, &encryption_key).expect("failed to encrypt tree");

            assert!(is_value_encrypted(
                secret
                    .pointer("/strValue")
                    .expect("failed to find strValue value")
            ));
            assert!(is_value_encrypted(
                secret
                    .pointer("/nestedObjValue/again/token")
                    .expect("failed to find token value")
            ));
            assert!(is_value_encrypted(
                secret
                    .pointer("/nestedObjValue/again/groceries/2")
                    .expect("failed to find groceries/2 value")
            ));

            decrypt_value_tree(&mut secret, &mut sensitive_strings, &decryption_key)
                .expect("failed to decrypt tree");

            assert_eq!(expected_secret, secret);

            let strings: HashSet<SensitiveString> = sensitive_strings.into();
            assert_eq!(5, strings.len());
            assert!(strings.contains(&"milk".into()));
            assert!(strings.contains(&"cheese".into()));
            assert!(strings.contains(&"brains".into()));
            assert!(strings.contains(&"token-1-2-3".into()));
            assert!(strings.contains(&"i-am-a-string".into()));
        }
    }
}
