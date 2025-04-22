use serde::{
    Deserialize,
    Serialize,
};
pub use si_id::UserPk;
use strum::Display;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, Display)]
pub enum Actor {
    System,
    User(UserPk),
}
