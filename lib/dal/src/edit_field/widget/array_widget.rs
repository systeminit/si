use serde::{Deserialize, Serialize};

use crate::edit_field::EditFields;

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
pub struct ArrayWidget {
    entries: Vec<EditFields>,
}

impl ArrayWidget {
    pub fn new(entries: Vec<EditFields>) -> Self {
        ArrayWidget { entries }
    }
}

impl From<Vec<EditFields>> for ArrayWidget {
    fn from(entries: Vec<EditFields>) -> Self {
        Self::new(entries)
    }
}
