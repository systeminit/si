mod prop_string;
pub use prop_string::PropString;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum Prop {
    String(PropString),
}

impl Prop {
    pub fn id(&self) -> &str {
        match self {
            Prop::String(p) => p.id.as_str(),
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Prop::String(p) => p.name.as_ref(),
        }
    }

    pub fn parent_id(&self) -> Option<&str> {
        match self {
            Prop::String(p) => p.parent_id.as_deref(),
        }
    }
}
