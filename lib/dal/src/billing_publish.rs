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

use billing_events::{BillingEvent, BillingEventChunkInfo, BillingEventKind, BillingEventsError};
use si_events::ResourceMetadata;
use telemetry::prelude::*;
use thiserror::Error;
use ulid::{Generator, MonotonicError};

use crate::{
    resource_metadata::{self, ResourceMetadataError},
    ChangeSet, ChangeSetError, Component, ComponentError, ComponentId, DalContext, SchemaVariant,
    SchemaVariantError, TransactionsError, WorkspacePk,
};

/// The max number of resources in a given chunk when counting resources.
const RESOURCE_CHUNK_LIMIT: usize = 300;

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
    #[error("missing chunk id (index: {0}) in chunk group order: {1:?}")]
    MissingChunkIdInGroup(usize, Vec<si_events::ulid::Ulid>),
    #[error("monotonic error: {0}")]
    Monotonic(#[from] MonotonicError),
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
pub(crate) async fn for_head_change_set_pointer_update(
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

    let kind = BillingEventKind::HeadChangeSetPointerUpdate;
    let metadata = resource_metadata::list(ctx).await?;
    let resource_count_total = metadata.len();
    let workspace_id = change_set.workspace_id()?;

    if resource_count_total > RESOURCE_CHUNK_LIMIT {
        let events = break_metadata_into_chunks(
            change_set,
            workspace_id,
            metadata.as_slice(),
            RESOURCE_CHUNK_LIMIT,
            kind,
        )
        .await?;
        for event in events {
            ctx.services_context()
                .jetstream_streams()
                .billing_events()
                .publish_workspace_update(&workspace_id.to_string(), &event)
                .await?;
        }
    } else {
        let event = BillingEvent {
            workspace_id: workspace_id.into(),
            workspace_snapshot_address: change_set.workspace_snapshot_address,
            change_set_status: change_set.status.into(),
            change_set_id: change_set.id.into(),
            merge_requested_by_user_id: change_set.merge_requested_by_user_id.map(Into::into),

            resource_count_total: Some(resource_count_total),
            resource_count_for_chunk: None,
            resource_metadata: Some(metadata),

            component_id: None,
            component_name: None,
            schema_variant_id: None,
            schema_id: None,
            schema_name: None,

            kind,
            chunk_info: None,
        };

        ctx.services_context()
            .jetstream_streams()
            .billing_events()
            .publish_workspace_update(&workspace_id.to_string(), &event)
            .await?;
    }

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

    let kind = BillingEventKind::ChangeSetStatusUpdate;
    let metadata = resource_metadata::list(ctx).await?;
    let resource_count_total = metadata.len();
    let workspace_id = change_set.workspace_id()?;

    if resource_count_total > RESOURCE_CHUNK_LIMIT {
        let events = break_metadata_into_chunks(
            change_set,
            workspace_id,
            metadata.as_slice(),
            RESOURCE_CHUNK_LIMIT,
            kind,
        )
        .await?;
        for event in events {
            ctx.services_context()
                .jetstream_streams()
                .billing_events()
                .publish_workspace_update(&workspace_id.to_string(), &event)
                .await?;
        }
    } else {
        let event = BillingEvent {
            workspace_id: workspace_id.into(),
            workspace_snapshot_address: change_set.workspace_snapshot_address,
            change_set_status: change_set.status.into(),
            change_set_id: change_set.id.into(),
            merge_requested_by_user_id: change_set.merge_requested_by_user_id.map(Into::into),

            resource_count_total: Some(resource_count_total),
            resource_count_for_chunk: None,
            resource_metadata: Some(metadata),

            component_id: None,
            component_name: None,
            schema_variant_id: None,
            schema_id: None,
            schema_name: None,

            kind,
            chunk_info: None,
        };

        ctx.services_context()
            .jetstream_streams()
            .billing_events()
            .publish_workspace_update(&workspace_id.to_string(), &event)
            .await?;
    }

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
        workspace_id: workspace_id.into(),
        workspace_snapshot_address: change_set.workspace_snapshot_address,
        change_set_status: change_set.status.into(),
        change_set_id: change_set.id.into(),
        merge_requested_by_user_id: change_set.merge_requested_by_user_id.map(Into::into),

        resource_count_total: None,
        resource_count_for_chunk: None,
        resource_metadata: None,

        component_id: Some(component_id.into()),
        component_name: Some(component_name),
        schema_variant_id: Some(schema_variant_id.into()),
        schema_id: Some(schema_id.into()),
        schema_name: Some(schema_name),

        kind: BillingEventKind::ResourceCreate,
        chunk_info: None,
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
        workspace_id: workspace_id.into(),
        workspace_snapshot_address: change_set.workspace_snapshot_address,
        change_set_status: change_set.status.into(),
        change_set_id: change_set.id.into(),
        merge_requested_by_user_id: change_set.merge_requested_by_user_id.map(Into::into),

        resource_count_total: None,
        resource_count_for_chunk: None,
        resource_metadata: None,

        component_id: Some(component_id.into()),
        component_name: Some(component_name),
        schema_variant_id: Some(schema_variant_id.into()),
        schema_id: Some(schema_id.into()),
        schema_name: Some(schema_name),

        kind: BillingEventKind::ResourceDelete,
        chunk_info: None,
    };

    ctx.services_context()
        .jetstream_streams()
        .billing_events()
        .publish_workspace_update(&workspace_id.to_string(), &event)
        .await?;

    Ok(())
}

#[instrument(
    name = "billing_publish.break_into_chunks",
    level = "debug",
    skip(change_set, workspace_id, to_chunk)
)]
async fn break_metadata_into_chunks(
    change_set: &ChangeSet,
    workspace_id: WorkspacePk,
    to_chunk: &[ResourceMetadata],
    chunk_size: usize,
    kind: BillingEventKind,
) -> BillingPublishResult<Vec<BillingEvent>> {
    // First, let's chunk everything and keep track of the total number of chunks and total number
    // of resources.
    let resource_count_total = to_chunk.len();
    let chunks: Vec<&[ResourceMetadata]> = to_chunk.chunks(chunk_size).collect();
    let total = chunks.len();

    // Let's get a stable order of the chunks as well the unique identifier for the entire chunk group.
    let (chunk_group_order, chunk_group_id) = {
        let mut generator = Generator::new();
        let chunk_group_id: si_events::ulid::Ulid = generator.generate()?.into();

        let mut chunk_group_order: Vec<si_events::ulid::Ulid> = Vec::new();
        for _ in 0..chunks.len() {
            chunk_group_order.push(generator.generate()?.into());
        }
        (chunk_group_order, chunk_group_id)
    };

    // Iterate through all chunks and assemble them in order.
    let mut events = Vec::new();
    for (index, chunk) in chunks.iter().enumerate() {
        let chunk_id =
            *chunk_group_order
                .get(index)
                .ok_or(BillingPublishError::MissingChunkIdInGroup(
                    index,
                    chunk_group_order.to_owned(),
                ))?;
        {
            let count = index + 1;
            debug!(
                %chunk_size,
                %chunk_id,
                %chunk_group_id,
                ?kind,
                "assembling chunk {count} of {total}"
            );
        }

        // Assemble the chunk.
        let chunk_metadata = chunk.to_vec();
        events.push(BillingEvent {
            workspace_id: workspace_id.into(),
            workspace_snapshot_address: change_set.workspace_snapshot_address,
            change_set_status: change_set.status.into(),
            change_set_id: change_set.id.into(),
            merge_requested_by_user_id: change_set.merge_requested_by_user_id.map(Into::into),

            resource_count_total: Some(resource_count_total),
            resource_count_for_chunk: Some(chunk_metadata.len()),
            resource_metadata: Some(chunk_metadata),

            component_id: None,
            component_name: None,
            schema_variant_id: None,
            schema_id: None,
            schema_name: None,

            kind,
            chunk_info: Some(BillingEventChunkInfo {
                chunk_id,
                chunk_group_id,
                chunk_group_order: chunk_group_order.to_owned(),
            }),
        });
    }

    Ok(events)
}
