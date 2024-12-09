use serde::{Deserialize, Serialize};
use strum::Display;

pub use si_id::UserPk;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, Display)]
pub enum Actor {
    System,
    User(UserPk),
}
