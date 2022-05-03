use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct FuncBackendIdentityArgs {
    pub identity: serde_json::Value,
}
