use serde::{Deserialize, Serialize};

#[derive(Default, Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
pub struct CheckboxWidget {}

impl CheckboxWidget {
    pub fn new() -> Self {
        Self::default()
    }
}
