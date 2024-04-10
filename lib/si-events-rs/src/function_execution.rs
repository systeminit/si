use std::{str::FromStr, string::ParseError};

use ulid::Ulid;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct FunctionExecutionKey {
    value: String,
}

impl FunctionExecutionKey {
    pub fn new(component_id: Ulid, prototype_id: Ulid, action_id: Ulid) -> Self {
        Self {
            value: format!("{}{}{}", component_id, prototype_id, action_id),
        }
    }

    pub fn value(&self) -> &str {
        &self.value
    }
}

impl FromStr for FunctionExecutionKey {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            value: String::from(s),
        })
    }
}
