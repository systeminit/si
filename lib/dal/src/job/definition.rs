mod action;
pub mod compute_validation;
pub mod dependent_values_update;

pub use action::ActionJob;
pub use dependent_values_update::DependentValuesUpdate;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub enum AttributeValueBasedJobIdentifier {
    DependentValuesUpdate,
    ComputeValidation,
}

impl AttributeValueBasedJobIdentifier {
    /// List job kinds in the order they should be fetched and executed
    pub fn in_priority_order() -> [AttributeValueBasedJobIdentifier; 2] {
        [Self::DependentValuesUpdate, Self::ComputeValidation]
    }
}
