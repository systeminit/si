use serde::{
    Deserialize,
    Serialize,
};
use si_id::AuthTokenId;
use strum::{
    AsRefStr,
    Display,
    EnumString,
};

pub type AuthenticationMethod = AuthenticationMethodV1;
pub type AuthenticationMethodRole = AuthenticationMethodRoleV1;

#[derive(AsRefStr, Clone, Debug, Copy, Deserialize, Eq, PartialEq, Serialize, Display, Hash)]
#[serde(tag = "method")]
pub enum AuthenticationMethodV1 {
    /// For revocable SDF tokens (particularly Automation tokens)
    /// Appears as { "method": "Jwt", "role": "Web" | "Automation", "token_id": "<...>" }
    Jwt {
        role: AuthenticationMethodRoleV1,
        token_id: Option<AuthTokenId>,
    },
    /// For backend systems like rebaser
    /// Serializes as { "method": "System" }
    System,
}

#[derive(
    AsRefStr, Clone, Debug, Copy, Deserialize, Eq, PartialEq, Serialize, Display, EnumString, Hash,
)]
pub enum AuthenticationMethodRoleV1 {
    Web,
    Automation,
}
