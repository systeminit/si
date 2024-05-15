use std::{collections::HashMap, sync::Arc};

use serde::{Deserialize, Serialize};
use si_events::ContentHash;
use thiserror::Error;

use crate::{
    layer_db_types::{NoteContent, NoteContentV1},
    pk,
    workspace_snapshot::{
        content_address::{ContentAddress, ContentAddressDiscriminants},
        node_weight::{category_node_weight::CategoryNodeKind, NodeWeight, NodeWeightError},
    },
    ChangeSetError, DalContext, EdgeWeight, EdgeWeightError, EdgeWeightKind,
    EdgeWeightKindDiscriminants, Timestamp, TransactionsError, WorkspaceSnapshotError,
};

#[remain::sorted]
#[derive(Error, Debug)]
pub enum NoteError {
    #[error("change set error: {0}")]
    ChangeSet(#[from] ChangeSetError),
    #[error("edge weight error: {0}")]
    EdgeWeight(#[from] EdgeWeightError),
    #[error("layer db error: {0}")]
    LayerDb(#[from] si_layer_cache::LayerDbError),
    #[error("node weight error: {0}")]
    NodeWeight(#[from] NodeWeightError),
    #[error("serde json error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error("could not acquire lock: {0}")]
    TryLock(#[from] tokio::sync::TryLockError),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] WorkspaceSnapshotError),
}

pub type NoteResult<T> = Result<T, NoteError>;

pk!(NoteId);

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct Note {
    id: NoteId,
    #[serde(flatten)]
    timestamp: Timestamp,
    x: String,
    y: String,
    created_by_email: String,
    note: String,
}

impl Note {
    pub fn assemble(id: NoteId, inner: NoteContentV1) -> Self {
        Self {
            id,
            timestamp: inner.timestamp,
            x: inner.x,
            y: inner.y,
            created_by_email: inner.created_by_email,
            note: inner.note,
        }
    }

    pub fn id(&self) -> NoteId {
        self.id
    }

    pub fn note(&self) -> String {
        self.note.clone()
    }

    pub fn x(&self) -> String {
        self.x.clone()
    }

    pub fn y(&self) -> String {
        self.y.clone()
    }

    pub async fn new(
        ctx: &DalContext,
        x: String,
        y: String,
        note: String,
        created_by_email: String,
    ) -> NoteResult<Self> {
        let timestamp = Timestamp::now();
        let content = NoteContentV1 {
            timestamp,
            note,
            x,
            y,
            created_by_email,
        };

        let (hash, _) = ctx
            .layer_db()
            .cas()
            .write(
                Arc::new(NoteContent::V1(content.clone()).into()),
                None,
                ctx.events_tenancy(),
                ctx.events_actor(),
            )
            .await?;

        let change_set = ctx.change_set()?;
        let id = change_set.generate_ulid()?;
        let node_weight = NodeWeight::new_content(change_set, id, ContentAddress::Note(hash))?;

        let workspace_snapshot = ctx.workspace_snapshot()?;
        workspace_snapshot.add_node(node_weight).await?;

        let note_index_id = workspace_snapshot
            .get_category_node(None, CategoryNodeKind::Note)
            .await?;
        workspace_snapshot
            .add_edge(
                note_index_id,
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())?,
                id,
            )
            .await?;

        Ok(Note::assemble(id.into(), content))
    }

    pub async fn get_by_id(ctx: &DalContext, id: NoteId) -> NoteResult<Self> {
        let workspace_snapshot = ctx.workspace_snapshot()?;

        let ulid: si_events::ulid::Ulid = id.into();
        let node_index = workspace_snapshot.get_node_index_by_id(ulid).await?;
        let node_weight = workspace_snapshot.get_node_weight(node_index).await?;
        let hash = node_weight.content_hash();

        let content: NoteContent = ctx
            .layer_db()
            .cas()
            .try_read_as(&hash)
            .await?
            .ok_or(WorkspaceSnapshotError::MissingContentFromStore(ulid))?;

        // If we had a v2, then there would be migration logic here.
        let NoteContent::V1(inner) = content;

        Ok(Note::assemble(id, inner))
    }

    pub async fn delete_by_id(ctx: &DalContext, id: NoteId) -> NoteResult<()> {
        let workspace_snapshot = ctx.workspace_snapshot()?;
        let change_set = ctx.change_set()?;
        workspace_snapshot.remove_node_by_id(change_set, id).await?;

        Ok(())
    }

    pub async fn list(ctx: &DalContext) -> NoteResult<Vec<Self>> {
        let workspace_snapshot = ctx.workspace_snapshot()?;

        let mut notes = vec![];
        let note_category_index_id = workspace_snapshot
            .get_category_node(None, CategoryNodeKind::Note)
            .await?;

        let note_node_indicies = workspace_snapshot
            .outgoing_targets_for_edge_weight_kind(
                note_category_index_id,
                EdgeWeightKindDiscriminants::Use,
            )
            .await?;

        let mut node_weights = vec![];
        let mut content_hashes = vec![];

        for note_index in note_node_indicies {
            let node_weight = workspace_snapshot
                .get_node_weight(note_index)
                .await?
                .get_content_node_weight_of_kind(ContentAddressDiscriminants::Note)?;
            content_hashes.push(node_weight.content_hash());
            node_weights.push(node_weight);
        }

        let content_map: HashMap<ContentHash, NoteContent> = ctx
            .layer_db()
            .cas()
            .try_read_many_as(content_hashes.as_slice())
            .await?;

        for node_weight in node_weights {
            match content_map.get(&node_weight.content_hash()) {
                Some(note_content) => {
                    let NoteContent::V1(inner) = note_content;

                    notes.push(Self::assemble(node_weight.id().into(), inner.to_owned()))
                }
                None => Err(WorkspaceSnapshotError::MissingContentFromStore(
                    node_weight.id(),
                ))?,
            }
        }

        Ok(notes)
    }
}
