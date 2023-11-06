use std::collections::HashSet;

use serde_json::{json, Value};
use si_crypto::{CycloneDecryptionKey, CycloneDecryptionKeyError, CycloneEncryptionKey};
use si_std::SensitiveString;
use thiserror::Error;

const MARKER_FIELD: &str = "cycloneEncryptedDataMarker";
const KEY_HASH_FIELD: &str = "keyHash";
const CRYPTED_FIELD: &str = "crypted";
const REDACTED_TXT: &str = "[redacted]";

#[derive(Debug, Error)]
pub enum CycloneValueEncryptError {
    #[error("invalid json pointer: {0}")]
    InvalidJSONPointer(String),
    #[error("error serializing to json: {0}")]
    Serialize(#[from] serde_json::Error),
}

#[derive(Debug, Error)]
pub enum CycloneValueDecryptError {
    #[error("object missing crypted field")]
    CryptedFieldMissing,
    #[error("object crypted field value was not a string")]
    CryptedFieldValueNotString,
    #[error("cyclone decryption error: {0}")]
    CycloneDecryption(#[from] CycloneDecryptionKeyError),
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

#[derive(Clone, Debug, Default)]
pub struct CycloneSensitiveStrings(HashSet<SensitiveString>);

impl CycloneSensitiveStrings {
    pub fn insert(&mut self, value: impl Into<SensitiveString>) {
        self.0.insert(value.into());
    }

    pub fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = SensitiveString>,
    {
        self.0.extend(iter)
    }

    pub fn has_sensitive(&self, s: &str) -> bool {
        self.0
            .iter()
            .any(|sensitive_s| s.contains(sensitive_s.as_str()))
    }

    pub fn redact(&self, s: &str) -> String {
        let mut redacted = s.to_string();

        for redacted_str in self.0.iter() {
            // Note: This brings a possibility of random substrings being matched out of context,
            // exposing that we have a secret by censoring it But trying to infer word boundary
            // might leak the plaintext credential which is arguably worse
            if s.contains(redacted_str.as_str()) {
                redacted = redacted.replace(redacted_str.as_str(), REDACTED_TXT);
            }
        }

        redacted
    }
}

pub fn encrypt_value_tree(
    value: &mut Value,
    encryption_key: &CycloneEncryptionKey,
) -> Result<(), CycloneValueEncryptError> {
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
            None => return Err(CycloneValueEncryptError::InvalidJSONPointer(pointer)),
        }
    }

    Ok(())
}

pub fn decrypt_value_tree(
    value: &mut Value,
    sensitive_strings: &mut CycloneSensitiveStrings,
    decryption_key: &CycloneDecryptionKey,
) -> Result<(), CycloneValueDecryptError> {
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
            None => return Err(CycloneValueDecryptError::InvalidJSONPointer(pointer)),
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
    encryption_key: &CycloneEncryptionKey,
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
    decryption_key: &CycloneDecryptionKey,
) -> Result<Value, CycloneValueDecryptError> {
    // Confirm value is an object
    let value = value
        .as_object()
        .ok_or(CycloneValueDecryptError::ValueNotObject)?;

    // Check marker field is set and is `true`
    if !value
        .get(MARKER_FIELD)
        .ok_or(CycloneValueDecryptError::MarkerFieldMissing)?
        .as_bool()
        .ok_or(CycloneValueDecryptError::MarkerFieldValueNotBool)?
    {
        return Err(CycloneValueDecryptError::MarkerFieldValueNotTrue);
    }
    // Confirm that key hash field value matches hash for provided decryption key
    if value
        .get(KEY_HASH_FIELD)
        .ok_or(CycloneValueDecryptError::KeyHashFieldMissing)?
        .as_str()
        .ok_or(CycloneValueDecryptError::KeyHashFieldValueNotString)?
        != decryption_key.encryption_key_hash_str()
    {
        return Err(CycloneValueDecryptError::KeyHashNoMatch);
    }

    // Decrypt crypted field and deserialize decrypted contents as a JSON value
    let decrypted = {
        let crypted = value
            .get(CRYPTED_FIELD)
            .ok_or(CycloneValueDecryptError::CryptedFieldMissing)?
            .as_str()
            .ok_or(CycloneValueDecryptError::CryptedFieldValueNotString)?;
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
        use si_crypto::CycloneKeyPair;

        use super::*;

        #[test]
        fn string_round_trip() {
            let (encryption_key, decryption_key) = CycloneKeyPair::create();

            let message = json!("Telling the Bees");
            let encrypted_value =
                encrypt_value(&message, &encryption_key).expect("failed to encrypt value");
            let decrypted_message = decrypt_value(&encrypted_value, &decryption_key)
                .expect("failed to decrypt message");

            assert_eq!(message, decrypted_message);
        }

        #[test]
        fn obj_round_trip() {
            let (encryption_key, decryption_key) = CycloneKeyPair::create();

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
        use si_crypto::CycloneKeyPair;

        use super::*;

        fn encrypted(encryption_key: &CycloneEncryptionKey) -> Value {
            encrypt_value(&json!("my-secret"), encryption_key).expect("failed to encrypt value")
        }

        #[test]
        fn value_not_object() {
            let (_encryption_key, decryption_key) = CycloneKeyPair::create();

            assert!(matches!(
                decrypt_value(&json!("uh oh"), &decryption_key),
                Err(CycloneValueDecryptError::ValueNotObject),
            ));
        }

        #[test]
        fn marker_field_missing() {
            let (encryption_key, decryption_key) = CycloneKeyPair::create();

            let mut encrypted = encrypted(&encryption_key);
            encrypted
                .as_object_mut()
                .expect("value is not an object")
                .remove(MARKER_FIELD);

            assert!(matches!(
                decrypt_value(&encrypted, &decryption_key),
                Err(CycloneValueDecryptError::MarkerFieldMissing),
            ));
        }

        #[test]
        fn marker_field_value_not_bool() {
            let (encryption_key, decryption_key) = CycloneKeyPair::create();

            let mut encrypted = encrypted(&encryption_key);
            *encrypted
                .as_object_mut()
                .expect("value is not an object")
                .get_mut(MARKER_FIELD)
                .expect("missing field") = json!("not a bool");

            assert!(matches!(
                decrypt_value(&encrypted, &decryption_key),
                Err(CycloneValueDecryptError::MarkerFieldValueNotBool),
            ));
        }

        #[test]
        fn marker_field_value_not_true() {
            let (encryption_key, decryption_key) = CycloneKeyPair::create();

            let mut encrypted = encrypted(&encryption_key);
            *encrypted
                .as_object_mut()
                .expect("value is not an object")
                .get_mut(MARKER_FIELD)
                .expect("missing field") = json!(false);

            assert!(matches!(
                decrypt_value(&encrypted, &decryption_key),
                Err(CycloneValueDecryptError::MarkerFieldValueNotTrue),
            ));
        }

        #[test]
        fn key_hash_field_missing() {
            let (encryption_key, decryption_key) = CycloneKeyPair::create();

            let mut encrypted = encrypted(&encryption_key);
            encrypted
                .as_object_mut()
                .expect("value is not an object")
                .remove(KEY_HASH_FIELD);

            assert!(matches!(
                decrypt_value(&encrypted, &decryption_key),
                Err(CycloneValueDecryptError::KeyHashFieldMissing),
            ));
        }

        #[test]
        fn key_hash_field_value_not_str() {
            let (encryption_key, decryption_key) = CycloneKeyPair::create();

            let mut encrypted = encrypted(&encryption_key);
            *encrypted
                .as_object_mut()
                .expect("value is not an object")
                .get_mut(KEY_HASH_FIELD)
                .expect("missing field") = json!(true);

            assert!(matches!(
                decrypt_value(&encrypted, &decryption_key),
                Err(CycloneValueDecryptError::KeyHashFieldValueNotString),
            ));
        }

        #[test]
        fn key_hash_field_value_no_match() {
            let (encryption_key, decryption_key) = CycloneKeyPair::create();
            let (wrong_encryption_key, _) = CycloneKeyPair::create();

            let mut encrypted = encrypted(&encryption_key);
            *encrypted
                .as_object_mut()
                .expect("value is not an object")
                .get_mut(KEY_HASH_FIELD)
                .expect("missing field") = json!(wrong_encryption_key.key_hash().to_string());

            assert!(matches!(
                decrypt_value(&encrypted, &decryption_key),
                Err(CycloneValueDecryptError::KeyHashNoMatch),
            ));
        }

        #[test]
        fn crypted_field_missing() {
            let (encryption_key, decryption_key) = CycloneKeyPair::create();

            let mut encrypted = encrypted(&encryption_key);
            encrypted
                .as_object_mut()
                .expect("value is not an object")
                .remove(CRYPTED_FIELD);

            assert!(matches!(
                decrypt_value(&encrypted, &decryption_key),
                Err(CycloneValueDecryptError::CryptedFieldMissing),
            ));
        }

        #[test]
        fn crypted_field_value_not_str() {
            let (encryption_key, decryption_key) = CycloneKeyPair::create();

            let mut encrypted = encrypted(&encryption_key);
            *encrypted
                .as_object_mut()
                .expect("value is not an object")
                .get_mut(CRYPTED_FIELD)
                .expect("missing field") = json!(true);

            assert!(matches!(
                decrypt_value(&encrypted, &decryption_key),
                Err(CycloneValueDecryptError::CryptedFieldValueNotString),
            ));
        }

        #[test]
        fn crypted_field_value_not_properly_encoded() {
            let (encryption_key, decryption_key) = CycloneKeyPair::create();

            let mut encrypted = encrypted(&encryption_key);
            *encrypted
                .as_object_mut()
                .expect("value is not an object")
                .get_mut(CRYPTED_FIELD)
                .expect("missing field") = json!("uh oh");

            assert!(matches!(
                decrypt_value(&encrypted, &decryption_key),
                Err(CycloneValueDecryptError::CycloneDecryption(_)),
            ));
        }

        #[test]
        fn wrong_decryption_key() {
            let (encryption_key, _) = CycloneKeyPair::create();
            let (_, wrong_decryption_key) = CycloneKeyPair::create();

            let encrypted = encrypted(&encryption_key);

            assert!(matches!(
                dbg!(decrypt_value(&encrypted, &wrong_decryption_key)),
                Err(CycloneValueDecryptError::KeyHashNoMatch),
            ));
        }
    }

    mod encrypt_value_tree {
        use si_crypto::CycloneKeyPair;

        use super::*;

        #[test]
        fn object_with_string_field_values_round_trip() {
            let (encryption_key, decryption_key) = CycloneKeyPair::create();
            let mut sensitive_strings = CycloneSensitiveStrings::default();

            let mut secret = json!({
                "username": "Mike Portnoy",
                "password": "Drummer",
            });
            let expected_secret = secret.clone();

            encrypt_value_tree(&mut secret, &encryption_key).expect("failed to encrypt tree");
            decrypt_value_tree(&mut secret, &mut sensitive_strings, &decryption_key)
                .expect("failed to decrypt tree");

            assert_eq!(expected_secret, secret);
            assert_eq!(2, sensitive_strings.0.len());
            assert!(sensitive_strings.0.contains(&"Mike Portnoy".into()));
            assert!(sensitive_strings.0.contains(&"Drummer".into()));
        }

        #[test]
        fn nested_object_round_trip() {
            let (encryption_key, decryption_key) = CycloneKeyPair::create();
            let mut sensitive_strings = CycloneSensitiveStrings::default();

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
            assert_eq!(5, sensitive_strings.0.len());
            assert!(sensitive_strings.0.contains(&"milk".into()));
            assert!(sensitive_strings.0.contains(&"cheese".into()));
            assert!(sensitive_strings.0.contains(&"brains".into()));
            assert!(sensitive_strings.0.contains(&"token-1-2-3".into()));
            assert!(sensitive_strings.0.contains(&"i-am-a-string".into()));
        }
    }

    mod sensitive_strings {
        use super::*;

        #[test]
        fn has_sensitive_with_empty() {
            let sensitive_strings = CycloneSensitiveStrings::default();

            assert!(!sensitive_strings.has_sensitive("nope"));
        }

        #[test]
        fn has_sensitive_single_match() {
            let mut sensitive_strings = CycloneSensitiveStrings::default();
            sensitive_strings.insert("careful");

            assert!(sensitive_strings.has_sensitive("I should be more careful in the future."));
        }

        #[test]
        fn has_sensitive_multiple_matches() {
            let mut sensitive_strings = CycloneSensitiveStrings::default();
            sensitive_strings.insert("careful");
            sensitive_strings.insert("more");

            assert!(sensitive_strings.has_sensitive("I should be more careful in the future."));
        }

        #[test]
        fn redact_with_empty() {
            let sensitive_strings = CycloneSensitiveStrings::default();

            assert_eq!(
                "nothing changed",
                sensitive_strings.redact("nothing changed")
            );
        }

        #[test]
        fn redact_single_match() {
            let mut sensitive_strings = CycloneSensitiveStrings::default();
            sensitive_strings.insert("careful");
            sensitive_strings.insert("pony");

            assert_eq!(
                "I should be more [redacted] in the future.",
                sensitive_strings.redact("I should be more careful in the future.")
            );
        }

        #[test]
        fn redact_multiple_matches() {
            let mut sensitive_strings = CycloneSensitiveStrings::default();
            sensitive_strings.insert("apple");
            sensitive_strings.insert("pony");

            assert_eq!(
                "One [redacted] said to the other [redacted]: 'I have an [redacted].'",
                sensitive_strings.redact("One pony said to the other pony: 'I have an apple.'")
            );
        }
    }
}
