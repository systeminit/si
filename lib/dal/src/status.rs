//! Status system that can send real time updates for activity to external
//! consumers, such as the web frontend.

use chrono::{
    DateTime,
    Utc,
};
use serde::{
    Deserialize,
    Serialize,
};
use thiserror::Error;

use crate::{
    ComponentId,
    DalContext,
    WsEvent,
    WsEventResult,
    WsPayload,
    attribute::{
        prototype::AttributePrototypeError,
        value::AttributeValueError,
    },
    prop::PropError,
};

#[remain::sorted]
#[derive(Error, Debug)]
pub enum StatusUpdateError {
    #[error("attribute prototype error: {0}")]
    AttributePrototype(#[from] AttributePrototypeError),
    #[error("attribute value error: {0}")]
    AttributeValue(#[from] AttributeValueError),
    #[error("prop error: {0}")]
    Prop(#[from] PropError),
}

pub type StatusUpdateResult<T> = Result<T, StatusUpdateError>;

/// The state of a status update message.
#[remain::sorted]
#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum StatusMessageState {
    /// A status update has finished
    StatusFinished,
    /// A status update has started
    StatusStarted,
}

/// A status message which encapsulates a new status for some subset of entries.
#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum StatusUpdate {
    /// Updates sent by the dependent values update job
    #[serde(rename_all = "camelCase")]
    DependentValueUpdate {
        status: StatusMessageState,
        component_id: ComponentId,
        timestamp: DateTime<Utc>,
    },
    /// Updates sent by the rebaser
    #[serde(rename_all = "camelCase")]
    Rebase {
        status: StatusMessageState,
        timestamp: DateTime<Utc>,
    },
}

/// A computed set of metadata relating to an [`AttributeValue`].
impl StatusUpdate {
    /// Create a status update message for a dependent values update
    pub fn new_dvu(status: StatusMessageState, component_id: ComponentId) -> Self {
        Self::DependentValueUpdate {
            status,
            component_id,
            timestamp: Utc::now(),
        }
    }

    /// Create a status update message for a rebase operation
    pub fn new_rebase(status: StatusMessageState) -> Self {
        Self::Rebase {
            status,
            timestamp: Utc::now(),
        }
    }
}

impl WsEvent {
    /// Creates a new `WsEvent` for a [`StatusUpdate`].
    ///
    /// # Errors
    ///
    /// Returns [`Err`] if no user exists for a user pk or if there is a connection issue with the
    /// database.
    pub async fn status_update(ctx: &DalContext, status: StatusUpdate) -> WsEventResult<Self> {
        WsEvent::new(ctx, WsPayload::StatusUpdate(status)).await
    }
}
