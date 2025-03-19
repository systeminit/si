//! These traits are the interface though which all interaction with the underlying graph should
//! occur. Using these traits should not require any knowledge of how the underlying graph is
//! implemented.

pub mod approval_requirement;
pub mod diagram;
pub mod entity_kind;
pub mod prop;
pub mod schema;
pub mod socket;
