use std::str::FromStr;

use serde::{Deserialize, Serialize};
use strum::Display;
use ulid::Ulid;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, Display)]
pub enum Actor {
    System,
    User(UserPk),
}

#[derive(Copy, Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct UserPk(Ulid);

impl UserPk {
    pub fn new() -> UserPk {
        UserPk(Ulid::new())
    }

    pub fn into_inner(self) -> Ulid {
        self.0
    }
}

impl Default for UserPk {
    fn default() -> Self {
        Self::new()
    }
}

impl FromStr for UserPk {
    type Err = ulid::DecodeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(Ulid::from_str(s)?))
    }
}

impl From<ulid::Ulid> for UserPk {
    fn from(value: ulid::Ulid) -> Self {
        Self(value)
    }
}
