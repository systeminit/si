use serde::{Deserialize, Serialize};

use crate::edit_field::EditFields;

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
pub struct HeaderWidget {
    edit_fields: EditFields,
}

impl HeaderWidget {
    pub fn new(edit_fields: EditFields) -> Self {
        HeaderWidget { edit_fields }
    }
}
