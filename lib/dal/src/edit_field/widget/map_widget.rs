use serde::{Deserialize, Serialize};

use crate::edit_field::EditField;

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
pub struct MapWidget {
    entries: Vec<EditField>,
}

impl MapWidget {
    pub fn new(entries: Vec<EditField>) -> Self {
        MapWidget { entries }
    }

    pub fn edit_fields(&self) -> &Vec<EditField> {
        &self.entries
    }
}
