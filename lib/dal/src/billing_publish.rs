//! This module provides the ability to publish billing events.

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

use billing_events::{BillingEvent, BillingEventKind, BillingEventsError};
use chrono::Utc;
use si_events::FuncRunId;
use telemetry::prelude::*;
use thiserror::Error;

use crate::{
    resource_metadata::{self, ResourceMetadataError},
    ChangeSet, ChangeSetError, Component, ComponentError, ComponentId, DalContext, SchemaVariant,
    SchemaVariantError, TransactionsError,
};

#[allow(missing_docs)]
#[remain::sorted]
#[derive(Debug, Error)]
pub enum BillingPublishError {
    #[error("billing events error: {0}")]
    BillingEvents(#[from] BillingEventsError),
    #[error("change set error: {0}")]
    ChangeSet(#[from] ChangeSetError),
    #[error("component error: {0}")]
    Component(#[from] ComponentError),
    #[error("resource metadata error: {0}")]
    ResourceMetadata(#[from] ResourceMetadataError),
    #[error("schema variant error: {0}")]
    SchemaVariant(#[from] SchemaVariantError),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
}

type BillingPublishResult<T> = Result<T, BillingPublishError>;

#[instrument(
    name = "billing_publish.for_head_change_set_pointer_update",
    level = "debug",
    skip(ctx, change_set)
)]
/// Publishes a billing event with resource count if the change set that was updated is HEAD
pub async fn for_head_change_set_pointer_update(
    ctx: &DalContext,
    change_set: &ChangeSet,
) -> BillingPublishResult<()> {
    if change_set.workspace_id.is_none() || !change_set.is_head(ctx).await? {
        trace!(
            ?change_set,
            "skipping billing event publishing since change set is not HEAD"
        );
        return Ok(());
    };

    // NOTE(nick): the metadata itself is not sent over the wire at the time of writing due to unbounded size in the
    // message payload. We should store this locally and batch up in a separate process.
    let metadata = resource_metadata::list(ctx).await?;
    let resource_count = metadata.len();

    let workspace_id = change_set.workspace_id()?;
    let event = BillingEvent {
        workspace_id,
        workspace_snapshot_address: change_set.workspace_snapshot_address,
        event_timestamp: Utc::now(),
        change_set_status: change_set.status.into(),
        change_set_id: change_set.id,
        merge_requested_by_user_id: change_set.merge_requested_by_user_id,

        resource_count: Some(resource_count),

        component_id: None,
        component_name: None,
        schema_variant_id: None,
        schema_id: None,
        schema_name: None,
        func_run_id: None,

        kind: BillingEventKind::HeadChangeSetPointerUpdate,
    };

    ctx.services_context()
        .jetstream_streams()
        .billing_events()
        .publish_workspace_update(&workspace_id.to_string(), &event)
        .await?;

    Ok(())
}

#[instrument(
    name = "billing_publish.for_change_set_status_update",
    level = "debug",
    skip(ctx, change_set)
)]
pub(crate) async fn for_change_set_status_update(
    ctx: &DalContext,
    change_set: &ChangeSet,
) -> BillingPublishResult<()> {
    if change_set.workspace_id.is_none() || !change_set.is_head(ctx).await? {
        trace!(
            ?change_set,
            "skipping billing event publishing since change set is not HEAD"
        );
        return Ok(());
    };

    // NOTE(nick): the metadata itself is not sent over the wire at the time of writing due to unbounded size in the
    // message payload. We should store this locally and batch up in a separate process.
    let metadata = resource_metadata::list(ctx).await?;
    let resource_count = metadata.len();

    let workspace_id = change_set.workspace_id()?;
    let event = BillingEvent {
        workspace_id,
        workspace_snapshot_address: change_set.workspace_snapshot_address,
        event_timestamp: Utc::now(),
        change_set_status: change_set.status.into(),
        change_set_id: change_set.id,
        merge_requested_by_user_id: change_set.merge_requested_by_user_id,

        resource_count: Some(resource_count),

        component_id: None,
        component_name: None,
        schema_variant_id: None,
        schema_id: None,
        schema_name: None,
        func_run_id: None,

        kind: BillingEventKind::ChangeSetStatusUpdate,
    };

    ctx.services_context()
        .jetstream_streams()
        .billing_events()
        .publish_workspace_update(&workspace_id.to_string(), &event)
        .await?;

    Ok(())
}

#[instrument(
    name = "billing_publish.for_resource_create",
    level = "debug",
    skip(ctx)
)]
pub(crate) async fn for_resource_create(
    ctx: &DalContext,
    component_id: ComponentId,
    func_run_id: FuncRunId,
) -> BillingPublishResult<()> {
    let change_set = ctx.change_set()?;

    if change_set.workspace_id.is_none() || !change_set.is_head(ctx).await? {
        trace!(
            ?change_set,
            "skipping billing event publishing since change set is not HEAD"
        );
        return Ok(());
    };

    let component_name = Component::name_by_id(ctx, component_id).await?;
    let schema_variant_id = Component::schema_variant_id(ctx, component_id).await?;
    let schema = SchemaVariant::schema_for_schema_variant_id(ctx, schema_variant_id).await?;
    let schema_id = schema.id();
    let schema_name = schema.name().to_string();

    let workspace_id = change_set.workspace_id()?;
    let event = BillingEvent {
        workspace_id,
        workspace_snapshot_address: change_set.workspace_snapshot_address,
        event_timestamp: Utc::now(),
        change_set_status: change_set.status.into(),
        change_set_id: change_set.id,
        merge_requested_by_user_id: change_set.merge_requested_by_user_id,

        resource_count: None,

        component_id: Some(component_id),
        component_name: Some(component_name),
        schema_variant_id: Some(schema_variant_id),
        schema_id: Some(schema_id),
        schema_name: Some(schema_name),
        func_run_id: Some(func_run_id),

        kind: BillingEventKind::ResourceCreate,
    };

    ctx.services_context()
        .jetstream_streams()
        .billing_events()
        .publish_workspace_update(&workspace_id.to_string(), &event)
        .await?;

    Ok(())
}

#[instrument(
    name = "billing_publish.for_resource_delete",
    level = "debug",
    skip(ctx)
)]
pub(crate) async fn for_resource_delete(
    ctx: &DalContext,
    component_id: ComponentId,
    func_run_id: FuncRunId,
) -> BillingPublishResult<()> {
    let change_set = ctx.change_set()?;

    if change_set.workspace_id.is_none() || !change_set.is_head(ctx).await? {
        trace!(
            ?change_set,
            "skipping billing event publishing since change set is not HEAD"
        );
        return Ok(());
    };

    let component_name = Component::name_by_id(ctx, component_id).await?;
    let schema_variant_id = Component::schema_variant_id(ctx, component_id).await?;
    let schema = SchemaVariant::schema_for_schema_variant_id(ctx, schema_variant_id).await?;
    let schema_id = schema.id();
    let schema_name = schema.name().to_string();

    let workspace_id = change_set.workspace_id()?;
    let event = BillingEvent {
        workspace_id,
        workspace_snapshot_address: change_set.workspace_snapshot_address,
        event_timestamp: Utc::now(),
        change_set_status: change_set.status.into(),
        change_set_id: change_set.id,
        merge_requested_by_user_id: change_set.merge_requested_by_user_id,

        resource_count: None,

        component_id: Some(component_id),
        component_name: Some(component_name),
        schema_variant_id: Some(schema_variant_id),
        schema_id: Some(schema_id),
        schema_name: Some(schema_name),
        func_run_id: Some(func_run_id),

        kind: BillingEventKind::ResourceDelete,
    };

    ctx.services_context()
        .jetstream_streams()
        .billing_events()
        .publish_workspace_update(&workspace_id.to_string(), &event)
        .await?;

    Ok(())
}
