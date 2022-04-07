use serde::{Deserialize, Serialize};

use crate::edit_field::EditField;

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
pub struct ArrayWidget {
    entries: Vec<Vec<EditField>>,
}

impl ArrayWidget {
    pub fn new(entries: Vec<Vec<EditField>>) -> Self {
        ArrayWidget { entries }
    }
}

impl From<Vec<Vec<EditField>>> for ArrayWidget {
    fn from(entries: Vec<Vec<EditField>>) -> Self {
        Self::new(entries)
    }
}
