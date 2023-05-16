use cyclone_core::{
    ActionRunRequest, ComponentKind, ComponentView, ReconciliationRequest, ResolverFunctionRequest,
    SensitiveString, ValidationRequest,
};
use serde_json::Value;

use crate::{DecryptionKey, DecryptionKeyError};

pub trait ListSecrets {
    fn list_secrets(&self, key: &DecryptionKey)
        -> Result<Vec<SensitiveString>, DecryptionKeyError>;
}

pub trait DecryptRequest {
    fn decrypt_request(self, key: &DecryptionKey) -> Result<serde_json::Value, DecryptionKeyError>;
}

impl ListSecrets for ComponentView {
    fn list_secrets(
        &self,
        key: &DecryptionKey,
    ) -> Result<Vec<SensitiveString>, DecryptionKeyError> {
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
                        let encoded = object["encryptedSecret"]
                            .as_str()
                            .ok_or(DecryptionKeyError::EncryptedSecretNotFound)?;
                        let decrypted = key.decode_and_decrypt(encoded)?;
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
    fn decrypt_request(self, key: &DecryptionKey) -> Result<Value, DecryptionKeyError> {
        let mut value = serde_json::to_value(&self)?;
        if self.kind != ComponentKind::Credential {
            return Ok(value);
        }

        let mut work_queue = vec!["".to_owned()]; // JSON pointers
        while let Some(pointer) = work_queue.pop() {
            let new_value = match value.pointer(&pointer) {
                None => return Err(DecryptionKeyError::JSONPointerNotFound(value, pointer)),
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
                        let encoded = object["encryptedSecret"]
                            .as_str()
                            .ok_or(DecryptionKeyError::EncryptedSecretNotFound)?;
                        let decrypted = key.decode_and_decrypt(encoded)?;
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
                None => return Err(DecryptionKeyError::JSONPointerNotFound(value, pointer)),
            };
        }
        Ok(value)
    }
}

impl ListSecrets for ResolverFunctionRequest {
    fn list_secrets(
        &self,
        key: &DecryptionKey,
    ) -> Result<Vec<SensitiveString>, DecryptionKeyError> {
        let mut secrets = self.component.data.list_secrets(key)?;
        for component in &self.component.parents {
            secrets.extend(component.list_secrets(key)?);
        }
        Ok(secrets)
    }
}

impl DecryptRequest for ResolverFunctionRequest {
    fn decrypt_request(self, key: &DecryptionKey) -> Result<serde_json::Value, DecryptionKeyError> {
        let mut value = serde_json::to_value(&self)?;

        let (component, parents) = (self.component.data, self.component.parents);

        match value.pointer_mut("/component/data") {
            Some(v) => *v = component.decrypt_request(key)?,
            None => {
                return Err(DecryptionKeyError::JSONPointerNotFound(
                    value,
                    "/component/data".to_owned(),
                ));
            }
        }

        let mut decrypted_parents = Vec::with_capacity(parents.len());
        for parent in parents {
            decrypted_parents.push(parent.decrypt_request(key)?);
        }
        match value.pointer_mut("/component/parents") {
            Some(v) => *v = serde_json::Value::Array(decrypted_parents),
            None => {
                return Err(DecryptionKeyError::JSONPointerNotFound(
                    value,
                    "/component/parents".to_owned(),
                ));
            }
        }
        Ok(value)
    }
}

impl ListSecrets for ActionRunRequest {
    fn list_secrets(
        &self,
        _key: &DecryptionKey,
    ) -> Result<Vec<SensitiveString>, DecryptionKeyError> {
        // TODO(fnichol): we'll need to populate/consume secrets here shortly
        Ok(vec![])
    }
}

impl DecryptRequest for ActionRunRequest {
    fn decrypt_request(
        self,
        _key: &DecryptionKey,
    ) -> Result<serde_json::Value, DecryptionKeyError> {
        let value = serde_json::to_value(&self)?;
        // TODO(fnichol): we'll need to process the request with decrypted secrets
        Ok(value)
    }
}

impl ListSecrets for ReconciliationRequest {
    fn list_secrets(
        &self,
        _key: &DecryptionKey,
    ) -> Result<Vec<SensitiveString>, DecryptionKeyError> {
        // TODO(fnichol): we'll need to populate/consume secrets here shortly
        Ok(vec![])
    }
}

impl DecryptRequest for ReconciliationRequest {
    fn decrypt_request(
        self,
        _key: &DecryptionKey,
    ) -> Result<serde_json::Value, DecryptionKeyError> {
        let value = serde_json::to_value(&self)?;
        // TODO(fnichol): we'll need to process the request with decrypted secrets
        Ok(value)
    }
}

impl ListSecrets for ValidationRequest {
    fn list_secrets(
        &self,
        _key: &DecryptionKey,
    ) -> Result<Vec<SensitiveString>, DecryptionKeyError> {
        // TODO(fnichol): we'll need to populate/consume secrets here shortly
        Ok(vec![])
    }
}

impl DecryptRequest for ValidationRequest {
    fn decrypt_request(
        self,
        _key: &DecryptionKey,
    ) -> Result<serde_json::Value, DecryptionKeyError> {
        let value = serde_json::to_value(&self)?;
        // TODO(fnichol): we'll need to process the request with decrypted secrets
        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use base64::{engine::general_purpose, Engine};
    use sodiumoxide::crypto::box_::{PublicKey, SecretKey};

    use super::*;

    fn encrypt_and_encode(message: &[u8], pkey: &PublicKey) -> String {
        general_purpose::STANDARD_NO_PAD.encode(sodiumoxide::crypto::sealedbox::seal(message, pkey))
    }

    fn gen_keypair() -> (PublicKey, SecretKey) {
        sodiumoxide::crypto::box_::gen_keypair()
    }

    #[test]
    fn redact() {
        let (pkey, skey) = gen_keypair();
        let decryption_key = DecryptionKey::from(skey);

        let secret = serde_json::to_string(&serde_json::json!({
            "my-super-secret": "Varginha's UFO",
        }))
        .expect("Unable to serialize secret");
        let encoded = encrypt_and_encode(secret.as_bytes(), &pkey);

        assert_eq!(
            decryption_key
                .decode_and_decrypt(&encoded)
                .expect("Unable to decrypt"),
            secret.as_bytes()
        );

        let secrets = ComponentView {
            kind: ComponentKind::Credential,
            properties: serde_json::json!({
                "secret": {
                    "name": "ufo",
                    "secret_kind": "dockerHub",
                    "object_type": "credential",
                    "message": { "cycloneEncryptedDataMarker": true, "encryptedSecret": encoded },
                },
            }),
        }
        .list_secrets(&decryption_key)
        .expect("Unable to list secrets");
        assert_eq!(secrets[0].as_str(), "Varginha's UFO");
    }

    #[test]
    fn decrypt() {
        let (pkey, skey) = gen_keypair();
        let decryption_key = DecryptionKey::from(skey);

        let secret_json = serde_json::json!({
            "my-super-secret": "Varginha's UFO",
        });
        let secret = serde_json::to_string(&secret_json).expect("Unable to serialize secret");
        let encoded = encrypt_and_encode(secret.as_bytes(), &pkey);

        assert_eq!(
            decryption_key
                .decode_and_decrypt(&encoded)
                .expect("Unable to decrypt"),
            secret.as_bytes(),
        );

        let json = ComponentView {
            kind: ComponentKind::Credential,
            properties: serde_json::json!({
                "secret": {
                    "name": "ufo",
                    "secret_kind": "dockerHub",
                    "object_type": "credential",
                    "message": { "cycloneEncryptedDataMarker": true, "encryptedSecret": encoded },
                },
            }),
        }
        .decrypt_request(&decryption_key)
        .expect("Unable to decrypt component view");

        let decrypted_json = serde_json::json!({
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
