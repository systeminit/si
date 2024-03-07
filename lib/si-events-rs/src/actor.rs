use std::str::FromStr;

use serde::{Deserialize, Serialize};
use ulid::Ulid;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum Actor {
    System,
    User(UserPk),
}

#[derive(Copy, Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct UserPk(Ulid);

impl UserPk {
    pub fn into_inner(self) -> Ulid {
        self.0
    }
}

impl FromStr for UserPk {
    type Err = ulid::DecodeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(Ulid::from_str(s)?))
    }
}
