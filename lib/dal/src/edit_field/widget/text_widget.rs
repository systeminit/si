use serde::{Deserialize, Serialize};

#[derive(Default, Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
pub struct TextWidget {}

impl TextWidget {
    pub fn new() -> Self {
        Self::default()
    }
}
