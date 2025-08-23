use serde::{
    Deserialize,
    Serialize,
};
pub use si_id::{
    ActionId,
    ActionPrototypeId,
};
use strum::{
    AsRefStr,
    Display,
    EnumDiscriminants,
};

#[remain::sorted]
#[derive(AsRefStr, Debug, Copy, Clone, Deserialize, Serialize, PartialEq, Eq, Display, Hash)]
pub enum ActionKind {
    /// Create the "outside world" version of the modeled object.
    Create,
    /// Destroy the "outside world" version of the modeled object referenced in the resource.
    Destroy,
    /// This [`Action`][crate::Action] will only ever be manually queued.
    Manual,
    /// Refresh the resource to reflect the current state of the modeled object in the "outside
    /// world".
    Refresh,
    /// Update the version of the modeled object in the "outside world" to match the state of the
    /// model.
    Update,
}

#[derive(
    Debug,
    Copy,
    Clone,
    Deserialize,
    Serialize,
    EnumDiscriminants,
    PartialEq,
    Eq,
    Display,
    Hash,
    Default,
)]
#[strum_discriminants(derive(strum::Display, Serialize, Deserialize))]
pub enum ActionState {
    /// Action has been determined to be eligible to run, and has had its job sent to the job
    /// queue.
    Dispatched,
    /// Action failed during execution. See the job history for details.
    Failed,
    /// Action is "queued", but should not be considered as eligible to run, until moved to the
    /// `Queued` state.
    OnHold,
    /// Action is available to be dispatched once all of its prerequisites have succeeded, and been
    /// removed from the graph.
    #[default]
    Queued,
    /// Action has been dispatched, and started execution in the job system. See the job history
    /// for details.
    Running,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Display, Serialize, Deserialize)]
pub enum ActionResultState {
    Success,
    Failure,
    Unknown,
}
