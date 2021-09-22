mod prop_string;
pub use prop_string::PropString;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum Prop {
    String(PropString),
}
