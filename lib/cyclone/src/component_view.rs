use crate::key_pair::{DecryptRequest, KeyPairError, KEY_PAIR};
use crate::sensitive_container::{ListSecrets, SensitiveString};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Copy)]
#[serde(rename_all = "camelCase")]
pub enum ComponentKind {
    Standard,
    Credential,
}

impl Default for ComponentKind {
    fn default() -> Self {
        Self::Standard
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemView {
    pub name: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComponentView {
    pub name: String,
    pub system: Option<SystemView>,
    pub kind: ComponentKind,
    pub properties: Value,
}

impl ListSecrets for ComponentView {
    fn list_secrets(&self) -> Result<Vec<SensitiveString>, KeyPairError> {
        if self.kind != ComponentKind::Credential {
            return Ok(vec![]);
        }

        let mut credentials: Vec<SensitiveString> = vec![];

        // We need to first parse the tree for the secrets and then list them
        let mut secret_objects = vec![];
        let mut is_inside_secret_object = false;

        let mut work_queue = vec![self.properties.clone()];

        while let Some(work) = work_queue.pop() {
            match work {
                Value::Array(values) => work_queue.extend(values),
                Value::Object(object) => {
                    let is_decrypted_secret = object
                        .get("cycloneEncryptedDataMarker")
                        .map_or(false, |v| v.as_bool() == Some(true))
                        && object
                            .get("encryptedSecret")
                            .map_or(false, |v| v.is_string());

                    if !is_inside_secret_object && is_decrypted_secret {
                        let encrypted = object["encryptedSecret"]
                            .as_str()
                            .ok_or(KeyPairError::EncryptedSecretNotFound)?;
                        let decrypted = KEY_PAIR.decrypt(&base64::decode(encrypted)?)?;
                        secret_objects.push(serde_json::de::from_slice::<Value>(&decrypted)?);
                    } else {
                        object.into_iter().for_each(|(_, v)| work_queue.push(v));
                    }
                }

                Value::String(value) if is_inside_secret_object => {
                    credentials.push(value.clone().into())
                }
                // We don't care for scalar values outside of a secret's message JSON object
                Value::String(_) => {}

                // For now credentials can only be strings, although we should reconsider it
                Value::Null => {}
                Value::Bool(_) => {}
                Value::Number(_) => {}
            }

            // We should only process secrets at the end, as they behave differently
            if work_queue.is_empty() {
                if let Some(obj) = secret_objects.pop() {
                    is_inside_secret_object = true;
                    work_queue.push(obj);
                }
            }
        }
        Ok(credentials)
    }
}

impl DecryptRequest for ComponentView {
    fn decrypt_request(self) -> Result<Value, KeyPairError> {
        let mut value = serde_json::to_value(&self)?;
        if self.kind != ComponentKind::Credential {
            return Ok(value);
        }

        let mut work_queue = vec!["".to_owned()]; // JSON pointers
        while let Some(pointer) = work_queue.pop() {
            let new_value = match value.pointer(&pointer) {
                None => return Err(KeyPairError::JSONPointerNotFound(value, pointer)),
                Some(Value::Array(values)) => {
                    let iter = values
                        .iter()
                        .enumerate()
                        .map(|(index, _)| format!("{pointer}/{index}"));
                    work_queue.extend(iter);
                    continue;
                }
                Some(Value::Object(object)) => {
                    let is_decrypted_secret = object
                        .get("cycloneEncryptedDataMarker")
                        .map_or(false, |v| v.as_bool() == Some(true))
                        && object
                            .get("encryptedSecret")
                            .map_or(false, |v| v.is_string());

                    if is_decrypted_secret {
                        let encrypted = object["encryptedSecret"]
                            .as_str()
                            .ok_or(KeyPairError::EncryptedSecretNotFound)?;
                        let decrypted = KEY_PAIR.decrypt(&base64::decode(encrypted)?)?;
                        serde_json::de::from_slice(&decrypted)?
                    } else {
                        work_queue.extend(object.iter().map(|(key, _)| format!("{pointer}/{key}")));
                        continue;
                    }
                }

                // Scalar values will never be decrypted
                Some(Value::String(_)) => continue,
                Some(Value::Null) => continue,
                Some(Value::Bool(_)) => continue,
                Some(Value::Number(_)) => continue,
            };
            match value.pointer_mut(&pointer) {
                Some(v) => *v = new_value,
                None => return Err(KeyPairError::JSONPointerNotFound(value, pointer)),
            };
        }
        Ok(value)
    }
}

impl Default for ComponentView {
    fn default() -> Self {
        Self {
            name: Default::default(),
            system: Default::default(),
            kind: Default::default(),
            properties: serde_json::json!({}),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redact() {
        let secret = serde_json::to_string(&serde_json::json!({
            "my-super-secret": "Varginha's UFO",
        }))
        .expect("Unable to serialize secret");
        let encrypted =
            sodiumoxide::crypto::sealedbox::seal(secret.as_bytes(), &KEY_PAIR.public_key);
        assert_eq!(
            KEY_PAIR.decrypt(&encrypted).expect("Unable to decrypt"),
            secret.as_bytes()
        );
        let encrypted = base64::encode(&encrypted);

        let secrets = ComponentView {
            name: "Redacting".to_owned(),
            system: None,
            kind: ComponentKind::Credential,
            properties: serde_json::json!({
                "secret": {
                    "name": "ufo",
                    "secret_kind": "dockerHub",
                    "object_type": "credential",
                    "message": { "cycloneEncryptedDataMarker": true, "encryptedSecret": encrypted },
                },
            }),
        }
        .list_secrets()
        .expect("Unable to list secrets");
        assert_eq!(secrets[0].as_str(), "Varginha's UFO");
    }

    #[test]
    fn decrypt() {
        let secret_json = serde_json::json!({
            "my-super-secret": "Varginha's UFO",
        });
        let secret = serde_json::to_string(&secret_json).expect("Unable to serialize secret");
        let encrypted =
            sodiumoxide::crypto::sealedbox::seal(secret.as_bytes(), &KEY_PAIR.public_key);
        assert_eq!(
            KEY_PAIR.decrypt(&encrypted).expect("Unable to decrypt"),
            secret.as_bytes(),
        );
        let encrypted = base64::encode(&encrypted);

        let json = ComponentView {
            name: "Decrypting".to_owned(),
            system: None,
            kind: ComponentKind::Credential,
            properties: serde_json::json!({
                "secret": {
                    "name": "ufo",
                    "secret_kind": "dockerHub",
                    "object_type": "credential",
                    "message": { "cycloneEncryptedDataMarker": true, "encryptedSecret": encrypted },
                },
            }),
        }
        .decrypt_request()
        .expect("Unable to decrypt component view");

        let decrypted_json = serde_json::json!({
            "name": "Decrypting",
            "system": null,
            "kind": "credential",
            "properties": {
                "secret": {
                    "name": "ufo",
                    "secret_kind": "dockerHub",
                    "object_type": "credential",
                    "message": secret_json,
                },
            },
        });
        assert_eq!(json, decrypted_json);
    }
}
