use crate::{sensitive_container::ListSecrets, MaybeSensitive, SensitiveString};
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
    pub properties: MaybeSensitive<Value>,
}

impl ListSecrets for ComponentView {
    // TODO: in the future we will want to only filter the props with WidgetKind::SecretSelect
    // For now we don't have the metadata, so we try to find a DecryptedSecret from a ComponentKind::Credential
    // We only censor some critical DecryptedSecret metadata and the secret's JSON values
    fn list_secrets(&self) -> Vec<SensitiveString> {
        if let MaybeSensitive::Sensitive(properties) = &self.properties {
            let mut credentials: Vec<SensitiveString> = vec![];

            // We need to first parse the tree for the secrets and then list them
            let mut secret_objects = vec![];
            let mut is_inside_secret_object = false;

            let mut work_queue = vec![&**properties];

            while let Some(work) = work_queue.pop() {
                match work {
                    Value::Array(values) => work_queue.extend(values),
                    Value::Object(object) => {
                        // We try to find DecryptedSecret from its keys as we lack the proper metadata
                        // Note: if we ever edit dal::DecryptedSecret we will have to edit this function too
                        let is_decrypted_secret =
                            object.get("secret_kind").map_or(false, |v| v.is_string())
                                && object.get("object_type").map_or(false, |v| v.is_string())
                                && object.get("message").map_or(false, |v| v.is_object())
                                && object.get("name").map_or(false, |v| v.is_string());

                        if !is_inside_secret_object && is_decrypted_secret {
                            // We only want to censor message of a DecryptedSecret
                            // But message will be a Value::Object, so we need to encrypt it's values too
                            // Note: do we want to encrypt the message's keys?
                            secret_objects.push(&object["message"]);
                        } else {
                            object.values().for_each(|v| work_queue.push(v));
                        }
                    }

                    // Scalar values don't make sense outside of a secret's message JSON object
                    Value::String(value) if is_inside_secret_object => {
                        credentials.push(value.clone().into())
                    }
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
            credentials
        } else {
            vec![]
        }
    }
}

impl Default for ComponentView {
    fn default() -> Self {
        Self {
            name: Default::default(),
            system: Default::default(),
            kind: Default::default(),
            properties: MaybeSensitive::Plain(serde_json::json!({})),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{sensitive_container::ListSecrets, ComponentKind, ComponentView, MaybeSensitive};

    #[tokio::test]
    async fn redact() {
        let secrets = ComponentView {
            name: "Redacting".to_owned(),
            system: None,
            kind: ComponentKind::Credential,
            properties: MaybeSensitive::Sensitive(
                serde_json::json!({
                    "secret": {
                        "name": "ufo",
                        "secret_kind": "dockerHub",
                        "object_type": "credential",
                        "message": {
                            "my-super-secret": "Varginha's UFO",
                        },
                    },
                })
                .into(),
            ),
        }
        .list_secrets();
        assert_eq!(secrets[0].as_str(), "Varginha's UFO");
    }
}
