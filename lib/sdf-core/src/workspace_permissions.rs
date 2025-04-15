use serde_with::{DeserializeFromStr, SerializeDisplay};
use strum::{Display, EnumString, VariantNames};

#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    SerializeDisplay,
    Display,
    DeserializeFromStr,
    EnumString,
    VariantNames,
    PartialEq,
    Eq,
)]
#[strum(serialize_all = "camelCase")]
pub enum WorkspacePermissionsMode {
    #[default]
    Closed,
    Allowlist,
    Open,
}

impl WorkspacePermissionsMode {
    #[must_use]
    pub const fn variants() -> &'static [&'static str] {
        <WorkspacePermissionsMode as strum::VariantNames>::VARIANTS
    }
}

pub type WorkspacePermissions = String;
