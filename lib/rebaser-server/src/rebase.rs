use dal::change_set::{ChangeSet, ChangeSetError, ChangeSetId};
use dal::workspace_snapshot::conflict::Conflict;
use dal::workspace_snapshot::graph::ConflictsAndUpdates;
use dal::workspace_snapshot::update::Update;
use dal::workspace_snapshot::vector_clock::{HasVectorClocks, VectorClockId};
use dal::workspace_snapshot::{NodeId, NodeInformation, WorkspaceSnapshotError};
use dal::{
    DalContext, EdgeWeight, EdgeWeightKindDiscriminants, TransactionsError, WorkspacePk,
    WorkspaceSnapshot, WsEventError,
};
use si_events::WorkspaceSnapshotAddress;
use si_layer_cache::activities::rebase::RebaseStatus;
use si_layer_cache::activities::ActivityRebaseRequest;
use si_layer_cache::LayerDbError;
use telemetry::prelude::*;
use thiserror::Error;
use tokio::time::Instant;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum RebaseError {
    #[error("workspace snapshot error: {0}")]
    ChangeSet(#[from] ChangeSetError),
    #[error("layerdb error: {0}")]
    LayerDb(#[from] LayerDbError),
    #[error("missing change set")]
    MissingChangeSet(ChangeSetId),
    #[error("to_rebase snapshot has no recently seen vector clock for its change set {0}")]
    MissingVectorClockForChangeSet(ChangeSetId),
    #[error("snapshot has no recently seen vector clock for any change set")]
    MissingVectorClockForSnapshot,
    #[error("missing workspace snapshot for change set ({0}) (the change set likely isn't pointing at a workspace snapshot)")]
    MissingWorkspaceSnapshotForChangeSet(ChangeSetId),
    #[error("serde json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] WorkspaceSnapshotError),
    #[error("ws event error: {0}")]
    WsEvent(#[from] WsEventError),
}

type RebaseResult<T> = Result<T, RebaseError>;

#[instrument(name = "rebase.perform_rebase", level = "info", skip_all, fields(
    si.change_set.id = Empty,
    si.workspace.id = Empty,
    si.conflicts = Empty,
    si.updates = Empty,
    si.conflicts.count = Empty,
    si.updates.count = Empty,
))]
pub async fn perform_rebase(
    ctx: &mut DalContext,
    message: &ActivityRebaseRequest,
) -> RebaseResult<RebaseStatus> {
    let span = Span::current();
    span.record(
        "si.change_set.id",
        &message.metadata.tenancy.change_set_id.to_string(),
    );
    span.record(
        "si.workspace.id",
        &message.metadata.tenancy.workspace_pk.to_string(),
    );
    let start = Instant::now();
    // Gather everything we need to detect conflicts and updates from the inbound message.
    let mut to_rebase_change_set =
        ChangeSet::find(ctx, message.payload.to_rebase_change_set_id.into())
            .await?
            .ok_or(RebaseError::MissingChangeSet(
                message.payload.to_rebase_change_set_id.into(),
            ))?;
    let to_rebase_workspace_snapshot_address =
        to_rebase_change_set.workspace_snapshot_address.ok_or(
            RebaseError::MissingWorkspaceSnapshotForChangeSet(to_rebase_change_set.id),
        )?;
    debug!("before snapshot fetch and parse: {:?}", start.elapsed());
    let to_rebase_workspace_snapshot =
        WorkspaceSnapshot::find(ctx, to_rebase_workspace_snapshot_address).await?;
    let onto_workspace_snapshot: WorkspaceSnapshot =
        WorkspaceSnapshot::find(ctx, message.payload.onto_workspace_snapshot_address).await?;
    info!(
        "to_rebase_id (change set pointer): {}, onto_id (what we were sent): {}",
        to_rebase_workspace_snapshot_address,
        onto_workspace_snapshot.id().await
    );
    debug!("after snapshot fetch and parse: {:?}", start.elapsed());

    // Perform the conflicts and updates detection.
    //let onto_vector_clock_id: VectorClockId = message.payload.onto_vector_clock_id;

    // Choose the most recent vector clock for the to_rebase change set for conflict detection
    let to_rebase_vector_clock_id = to_rebase_workspace_snapshot
        .max_recently_seen_clock_id(Some(to_rebase_change_set.id))
        .await?
        .ok_or(RebaseError::MissingVectorClockForChangeSet(
            to_rebase_change_set.id,
        ))?;

    let onto_vector_clock_id = onto_workspace_snapshot
        .max_recently_seen_clock_id(None)
        .await?
        .ok_or(RebaseError::MissingVectorClockForChangeSet(
            to_rebase_change_set.id,
        ))?;

    let mut conflicts_and_updates = to_rebase_workspace_snapshot
        .detect_conflicts_and_updates(
            to_rebase_vector_clock_id,
            &onto_workspace_snapshot,
            onto_vector_clock_id,
        )
        .await?;

    info!(
        "count: conflicts ({}) and updates ({}), {:?}",
        conflicts_and_updates.conflicts.len(),
        conflicts_and_updates.updates.len(),
        start.elapsed()
    );

    let len_before = conflicts_and_updates.conflicts.len();
    if !conflicts_and_updates.conflicts.is_empty() {
        conflicts_and_updates = fix_prototype_race_conflicts(
            conflicts_and_updates.clone(),
            &to_rebase_workspace_snapshot,
        )
        .await?;
    }

    if conflicts_and_updates.conflicts.len() < len_before {
        info!("automatically resolved prototype edge exclusive edge mismatch");
    }
    let onto_workspace_snapshot_address = onto_workspace_snapshot.id().await;
    info!(
        ?onto_workspace_snapshot_address,
        ?to_rebase_workspace_snapshot_address,
        "NICK WRITING"
    );
    onto_workspace_snapshot
        .write_to_disk(format!("{onto_workspace_snapshot_address}-onto").as_str())
        .await;
    to_rebase_workspace_snapshot
        .write_to_disk(format!("{to_rebase_workspace_snapshot_address}-to-rebase").as_str())
        .await;

    // If there are conflicts, immediately assemble a reply message that conflicts were found.
    // Otherwise, we can perform updates and assemble a "success" reply message.
    let message: RebaseStatus = if conflicts_and_updates.conflicts.is_empty() {
        to_rebase_workspace_snapshot
            .perform_updates(
                to_rebase_vector_clock_id,
                &onto_workspace_snapshot,
                conflicts_and_updates.updates.as_slice(),
            )
            .await?;

        info!("updates complete: {:?}", start.elapsed());

        if !conflicts_and_updates.updates.is_empty() {
            // Once all updates have been performed, we can write out, mark everything as recently seen
            // and update the pointer.
            let workspace_pk = ctx.tenancy().workspace_pk().unwrap_or(WorkspacePk::NONE);
            let vector_clock_id = VectorClockId::new(
                to_rebase_change_set.id.into_inner(),
                workspace_pk.into_inner(),
            );

            to_rebase_workspace_snapshot
                .collapse_vector_clocks(ctx)
                .await?;

            let to_rebase_workspace_snapshot_id = to_rebase_workspace_snapshot
                .write(ctx, vector_clock_id)
                .await?;
            info!("snapshot written: {:?}", start.elapsed());
            to_rebase_change_set
                .update_pointer(ctx, to_rebase_workspace_snapshot_id)
                .await?;
            info!("post updates rebase snapshot id (what the action dispatch should use): {}", to_rebase_workspace_snapshot_id);
            info!("pointer updated: {:?}", start.elapsed());
        }
        let updates_count = conflicts_and_updates.updates.len();
        let updates_performed = serde_json::to_value(conflicts_and_updates.updates)?.to_string();

        span.record("si.updates", updates_performed.clone());
        span.record("si.updates.count", updates_count.to_string());
        RebaseStatus::Success { updates_performed }
    } else {
        let conflicts_count = conflicts_and_updates.conflicts.len();
        let conflicts_found = serde_json::to_value(conflicts_and_updates.conflicts)?.to_string();
        span.record("si.conflicts", conflicts_found.clone());
        span.record("si.conflicts.count", conflicts_count.to_string());
        RebaseStatus::ConflictsFound {
            conflicts_found,
            updates_found_and_skipped: serde_json::to_value(conflicts_and_updates.updates)?
                .to_string(),
        }
    };

    info!("rebase performed: {:?}", start.elapsed());

    // Before replying to the requester, we must commit.
    ctx.commit_no_rebase().await?;

    {
        let ictx = ctx.clone();
        tokio::spawn(async move {
            if let Err(error) =
                evict_unused_snapshots(&ictx, &to_rebase_workspace_snapshot_address).await
            {
                error!(?error, "Eviction error: {:?}", error);
            }
            if let Err(error) =
                evict_unused_snapshots(&ictx, &to_rebase_workspace_snapshot.id().await).await
            {
                error!(?error, "Eviction error: {:?}", error);
            }
            if let Err(error) =
                evict_unused_snapshots(&ictx, &onto_workspace_snapshot.id().await).await
            {
                error!(?error, "Eviction error: {:?}", error);
            }
        });
    }

    Ok(message)
}

pub(crate) async fn evict_unused_snapshots(
    ctx: &DalContext,
    workspace_snapshot_address: &WorkspaceSnapshotAddress,
) -> RebaseResult<()> {
    if !ChangeSet::workspace_snapshot_address_in_use(ctx, workspace_snapshot_address).await? {
        ctx.layer_db()
            .workspace_snapshot()
            .evict(
                workspace_snapshot_address,
                ctx.events_tenancy(),
                ctx.events_actor(),
            )
            .await?;
    }
    Ok(())
}

/// If the same user modifies two attributes in two+ components in quick
/// succession, they can race against themselves, producing a conflict where
/// an out of date attribute that was *not* changed in "onto" attempts to
/// stomp on the more up to date attribute in "to_rebase". We can fix this
/// by just removing the new edge update if there are no other updates for
/// the source id.
async fn fix_prototype_race_conflicts(
    mut conflicts_and_updates: ConflictsAndUpdates,
    to_rebase_snapshot: &WorkspaceSnapshot,
) -> RebaseResult<ConflictsAndUpdates> {
    let original_conflicts = conflicts_and_updates.conflicts.clone();
    for conflict in &original_conflicts {
        match conflict {
            Conflict::ExclusiveEdgeMismatch {
                source, edge_kind, ..
            } if edge_kind == &EdgeWeightKindDiscriminants::Prototype => {
                let mut new_edge_updates = find_new_edge_updates_for_source(
                    &conflicts_and_updates.updates,
                    source.id,
                    *edge_kind,
                );
                if new_edge_updates.len() != 1 {
                    // We can't resolve this one automatically because there
                    // is either no new edge update or more than one for
                    // this kind
                    continue;
                }
                let to_rebase_edge = to_rebase_snapshot
                    .edges_directed_for_edge_weight_kind(
                        source.id,
                        dal::workspace_snapshot::Direction::Outgoing,
                        *edge_kind,
                    )
                    .await?
                    .pop()
                    .map(|(edge_weight, _, _)| edge_weight);

                if let (Some(to_rebase_edge), Some((_, _, onto_edge_weight))) =
                    (to_rebase_edge, new_edge_updates.pop())
                {
                    let to_rebase_clock = to_rebase_edge.vector_clock_write().max(None);
                    let onto_clock = onto_edge_weight.vector_clock_write().max(None);
                    if let (
                        Some((to_rebase_clock_id, to_rebase_clock_stamp)),
                        Some((onto_clock_id, onto_clock_stamp)),
                    ) = (to_rebase_clock, onto_clock)
                    {
                        if to_rebase_clock_id == onto_clock_id
                            && to_rebase_clock_stamp > onto_clock_stamp
                        {
                            conflicts_and_updates =
                                remove_new_edge_update_and_conflict_for_source_if_safe(
                                    conflicts_and_updates,
                                    source.id,
                                    *edge_kind,
                                );
                        }
                    }
                }
            }
            _ => {}
        }
    }

    Ok(conflicts_and_updates)
}

// NOTE: This is only safe when used by the fix_prototype_race_conflicts function...
fn remove_new_edge_update_and_conflict_for_source_if_safe(
    mut conflicts_and_updates: ConflictsAndUpdates,
    source_id: NodeId,
    new_edge_kind: EdgeWeightKindDiscriminants,
) -> ConflictsAndUpdates {
    let is_it_safe = !conflicts_and_updates
        .updates
        .iter()
        .any(|update| match update {
            Update::NewEdge {
                source,
                edge_weight,
                ..
            } => source.id == source_id && new_edge_kind != edge_weight.kind().into(),
            Update::RemoveEdge { source, .. } => source.id == source_id,
            Update::ReplaceSubgraph { onto, .. } => onto.id == source_id,
            _ => false,
        });

    if !is_it_safe {
        return conflicts_and_updates;
    }

    conflicts_and_updates.updates.retain(|update| match update {
        Update::NewEdge {
            source,
            edge_weight,
            ..
        } if source.id == source_id => new_edge_kind != edge_weight.kind().into(),
        _ => true,
    });

    conflicts_and_updates
        .conflicts
        .retain(|conflict| match conflict {
            Conflict::ExclusiveEdgeMismatch {
                source, edge_kind, ..
            } if source.id == source_id => edge_kind != &new_edge_kind,
            _ => true,
        });

    conflicts_and_updates
}

fn find_new_edge_updates_for_source(
    updates: &[Update],
    source_id: NodeId,
    kind: EdgeWeightKindDiscriminants,
) -> Vec<(NodeInformation, NodeInformation, EdgeWeight)> {
    updates
        .iter()
        .filter_map(|update| match update {
            Update::NewEdge {
                source,
                destination,
                edge_weight,
            } if source.id == source_id && kind == edge_weight.kind().into() => Some((
                source.to_owned(),
                destination.to_owned(),
                edge_weight.to_owned(),
            )),
            _ => None,
        })
        .collect()
}
