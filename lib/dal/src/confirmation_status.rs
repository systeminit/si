//! This module contains the status of a confirmation execution as well as how to publish
//! the status in a [`WsEvent`](crate::WsEvent).

use crate::{ComponentId, ConfirmationPrototypeId, DalContext, SystemId, WsEvent, WsPayload};
use serde::{Deserialize, Serialize};

/// The status of an execution using a [`ConfirmationPrototype`](crate::ConfirmationPrototype).
#[derive(Deserialize, Serialize, Debug, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum ConfirmationStatus {
    /// An error occurred when executing the [`ConfirmationPrototype`](crate::ConfirmationPrototype).
    /// This does not indicate that the confirmation failed.
    Error,
    /// The [`ConfirmationPrototype`](crate::ConfirmationPrototype) executed successfully, but the
    /// confirmation failed.
    Failure,
    // TODO(nick): not yet used.
    Pending,
    /// The [`ConfirmationPrototype`](crate::ConfirmationPrototype) execution has begun.
    Running,
    /// The [`ConfirmationPrototype`](crate::ConfirmationPrototype) executed successfully and the
    /// confirmation succeeded.
    Success,
}

/// The payload of a [`WsEvent`](crate::WsEvent) representing the status of an execution using
/// a [`ConfirmationPrototype`](crate::ConfirmationPrototype).
#[derive(Deserialize, Serialize, Debug, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ConfirmationStatusUpdate {
    /// The [`Component`](crate::Component) for the
    /// [`ConfirmationPrototype`](crate::ConfirmationPrototype).
    component_id: ComponentId,
    /// The [`System`](crate::System) for the
    /// [`ConfirmationPrototype`](crate::ConfirmationPrototype).
    system_id: SystemId,
    /// The ID of the [`ConfirmationPrototype`](crate::ConfirmationPrototype) using for execution.
    confirmation_prototype_id: ConfirmationPrototypeId,
    /// The summary of the status of the [`ConfirmationPrototype`](crate::ConfirmationPrototype)
    /// execution.
    status: ConfirmationStatus,
    /// An optional error message typically used when the status is [`ConfirmationStatus::Error`].
    error_message: Option<String>,
}

impl WsEvent {
    pub fn confirmation_status_update(
        ctx: &DalContext,
        component_id: ComponentId,
        system_id: SystemId,
        confirmation_prototype_id: ConfirmationPrototypeId,
        status: ConfirmationStatus,
        error_message: Option<String>,
    ) -> Self {
        WsEvent::new(
            ctx,
            WsPayload::ConfirmationStatusUpdate(ConfirmationStatusUpdate {
                component_id,
                system_id,
                confirmation_prototype_id,
                status,
                error_message,
            }),
        )
    }
}
