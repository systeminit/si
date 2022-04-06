use serde::{Deserialize, Serialize};

use crate::edit_field::EditField;

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
pub struct HeaderWidget {
    edit_fields: Vec<EditField>,
}

impl HeaderWidget {
    pub fn new(edit_fields: Vec<EditField>) -> Self {
        HeaderWidget { edit_fields }
    }

    pub fn edit_fields(&self) -> &Vec<EditField> {
        &self.edit_fields
    }
}
