//! This module provides the ability to collect resource metadata for all [`Components`](Component) in the workspace.

#![warn(
    bad_style,
    clippy::missing_panics_doc,
    clippy::panic,
    clippy::panic_in_result_fn,
    clippy::unwrap_in_result,
    clippy::unwrap_used,
    dead_code,
    improper_ctypes,
    missing_debug_implementations,
    missing_docs,
    no_mangle_generic_items,
    non_shorthand_field_patterns,
    overflowing_literals,
    path_statements,
    patterns_in_fns_without_body,
    unconditional_recursion,
    unreachable_pub,
    unused,
    unused_allocation,
    unused_comparisons,
    unused_parens,
    while_true
)]

use telemetry::prelude::*;
use thiserror::Error;
use veritech_client::ResourceStatus;

use crate::{
    Component,
    ComponentError,
    ComponentId,
    DalContext,
    component::resource::ResourceData,
};

#[allow(missing_docs)]
#[remain::sorted]
#[derive(Debug, Error)]
pub enum ResourceMetadataError {
    #[error("component error: {0}")]
    Component(#[from] ComponentError),
}

type ResourceMetadataResult<T> = Result<T, ResourceMetadataError>;

/// Collect [`ResourceMetadata`] for every [`Component`] in the workspace.
#[instrument(name = "resource_metadata.list", level = "debug", skip(ctx))]
pub async fn list(ctx: &DalContext) -> ResourceMetadataResult<Vec<si_events::ResourceMetadata>> {
    let component_ids = Component::list_ids(ctx).await?;
    let mut results = Vec::with_capacity(component_ids.len());
    for component_id in component_ids {
        if let Some(data) = Component::resource_by_id(ctx, component_id).await? {
            results.push(assemble_metadata(component_id, data));
        }
    }
    Ok(results)
}

// WARN(nick): do NOT use the payload or inner data. Don't you dare.
fn assemble_metadata(component_id: ComponentId, data: ResourceData) -> si_events::ResourceMetadata {
    si_events::ResourceMetadata {
        component_id,
        status: match data.status {
            ResourceStatus::Error => si_events::ResourceStatus::Error,
            ResourceStatus::Ok => si_events::ResourceStatus::Ok,
            ResourceStatus::Warning => si_events::ResourceStatus::Warning,
        },
        last_synced: data.last_synced,
    }
}
