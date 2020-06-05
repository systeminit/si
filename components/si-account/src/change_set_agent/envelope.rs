use serde::{Deserialize, Serialize};
use serde_json;
use si_data::Storable;

use thiserror::Error;

pub type Result<T> = std::result::Result<T, ChangeSetEnvelopeError>;

#[derive(Error, Debug)]
pub enum ChangeSetEnvelopeError {
    #[error("cannot serialize the envelope payload: {0}")]
    Serialize(#[from] serde_json::error::Error),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ChangeSetAction {
    Create,
    Edit,
    Action(String),
}

impl std::fmt::Display for ChangeSetAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChangeSetAction::Create => write!(f, "create"),
            ChangeSetAction::Edit => write!(f, "edit"),
            ChangeSetAction::Action(act) => write!(f, "action/{}", act),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeSetEnvelope {
    pub change_set_action: ChangeSetAction,
    pub concrete_type: String,
    pub payload: String,
}

impl ChangeSetEnvelope {
    pub fn new<T: Serialize + Storable>(
        change_set_action: ChangeSetAction,
        concrete_type: impl Into<String>,
        payload: &T,
    ) -> Result<ChangeSetEnvelope> {
        let serialized_payload = serde_json::to_string(payload)?;

        Ok(ChangeSetEnvelope {
            change_set_action,
            concrete_type: concrete_type.into(),
            payload: serialized_payload,
        })
    }
}
