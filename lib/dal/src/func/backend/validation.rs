use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct ValidationError {
    pub message: String,
    /// This really should be an enum at some point, but we need to figure out the set of values it should be first.
    pub level: Option<String>,
    /// This really should be an enum at some point, but we need to figure out the set of values it should be first.
    pub kind: Option<String>,
    pub link: Option<String>,
}
