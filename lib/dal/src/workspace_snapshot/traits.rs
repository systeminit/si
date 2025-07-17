//! These traits are the interface though which all interaction with the underlying graph should
//! occur. Using these traits should not require any knowledge of how the underlying graph is
//! implemented.

pub mod approval_requirement;
pub mod attribute_prototype;
pub mod attribute_prototype_argument;
pub mod attribute_value;
pub mod component;
pub mod diagram;
pub mod entity_kind;
pub mod func;
pub mod prop;
pub mod schema;
pub mod socket;
pub mod static_argument_value;
