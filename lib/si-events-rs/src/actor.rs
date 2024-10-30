use serde::{Deserialize, Serialize};
use strum::Display;

use crate::id;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, Display)]
pub enum Actor {
    System,
    User(UserPk),
}

id!(UserPk);
