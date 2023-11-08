use content_store::{ContentHash, Store, StoreError};
use postgres_types::{FromSql, ToSql};
use serde::{Deserialize, Serialize};
use si_pkg::FuncArgumentKind as PkgFuncArgumentKind;
use std::collections::HashMap;
use strum::EnumDiscriminants;
use strum::{AsRefStr, Display, EnumIter, EnumString};
use telemetry::prelude::*;
use thiserror::Error;
use ulid::Ulid;

use crate::change_set_pointer::ChangeSetPointerError;
use crate::workspace_snapshot::content_address::ContentAddress;
use crate::workspace_snapshot::content_address::ContentAddressDiscriminants;
use crate::workspace_snapshot::edge_weight::{
    EdgeWeight, EdgeWeightError, EdgeWeightKind, EdgeWeightKindDiscriminants,
};
use crate::workspace_snapshot::node_weight::{ContentNodeWeight, NodeWeight, NodeWeightError};
use crate::workspace_snapshot::WorkspaceSnapshotError;
use crate::{
    pk, DalContext, FuncId, HistoryEventError, PropKind, StandardModelError, Timestamp,
    TransactionsError,
};

#[remain::sorted]
#[derive(Debug, Error)]
pub enum FuncArgumentError {
    #[error(transparent)]
    ChangeSetPointer(#[from] ChangeSetPointerError),
    #[error("edge weight error: {0}")]
    EdgeWeight(#[from] EdgeWeightError),
    #[error("history event error: {0}")]
    HistoryEvent(#[from] HistoryEventError),
    #[error("intrinsic func {0} ({1}) missing func argument edge")]
    IntrinsicMissingFuncArgumentEdge(String, FuncId),
    #[error(transparent)]
    NodeWeight(#[from] NodeWeightError),
    #[error("func argument not found with name {0} for Func {1}")]
    NotFoundByNameForFunc(String, FuncId),
    #[error("pg error: {0}")]
    Pg(#[from] si_data_pg::PgError),
    #[error("error serializing/deserializing json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("standard model error: {0}")]
    StandardModelError(#[from] StandardModelError),
    #[error(transparent)]
    Store(#[from] StoreError),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error("could not acquire lock: {0}")]
    TryLock(#[from] tokio::sync::TryLockError),
    #[error("workspace snapshot: {0}")]
    WorkspaceSnapshot(#[from] WorkspaceSnapshotError),
}

type FuncArgumentResult<T> = Result<T, FuncArgumentError>;

#[remain::sorted]
#[derive(
    Deserialize,
    Serialize,
    AsRefStr,
    Display,
    EnumIter,
    EnumString,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    ToSql,
    FromSql,
)]
pub enum FuncArgumentKind {
    Any,
    Array,
    Boolean,
    Integer,
    Map,
    Object,
    String,
}

impl From<PropKind> for FuncArgumentKind {
    fn from(prop_kind: PropKind) -> Self {
        match prop_kind {
            PropKind::Array => FuncArgumentKind::Array,
            PropKind::Boolean => FuncArgumentKind::Boolean,
            PropKind::Integer => FuncArgumentKind::Integer,
            PropKind::Object => FuncArgumentKind::Object,
            PropKind::String => FuncArgumentKind::String,
            PropKind::Map => FuncArgumentKind::Map,
        }
    }
}

impl From<PkgFuncArgumentKind> for FuncArgumentKind {
    fn from(value: PkgFuncArgumentKind) -> Self {
        match value {
            PkgFuncArgumentKind::Any => FuncArgumentKind::Any,
            PkgFuncArgumentKind::Array => FuncArgumentKind::Array,
            PkgFuncArgumentKind::Boolean => FuncArgumentKind::Boolean,
            PkgFuncArgumentKind::Integer => FuncArgumentKind::Integer,
            PkgFuncArgumentKind::Map => FuncArgumentKind::Map,
            PkgFuncArgumentKind::Object => FuncArgumentKind::Object,
            PkgFuncArgumentKind::String => FuncArgumentKind::String,
        }
    }
}

impl From<FuncArgumentKind> for PkgFuncArgumentKind {
    fn from(value: FuncArgumentKind) -> Self {
        match value {
            FuncArgumentKind::Any => PkgFuncArgumentKind::Any,
            FuncArgumentKind::Array => PkgFuncArgumentKind::Array,
            FuncArgumentKind::Boolean => PkgFuncArgumentKind::Boolean,
            FuncArgumentKind::Integer => PkgFuncArgumentKind::Integer,
            FuncArgumentKind::Map => PkgFuncArgumentKind::Map,
            FuncArgumentKind::Object => PkgFuncArgumentKind::Object,
            FuncArgumentKind::String => PkgFuncArgumentKind::String,
        }
    }
}

pk!(FuncArgumentPk);
pk!(FuncArgumentId);

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct FuncArgument {
    pub id: FuncArgumentId,
    pub name: String,
    pub kind: FuncArgumentKind,
    pub element_kind: Option<FuncArgumentKind>,
    #[serde(flatten)]
    pub timestamp: Timestamp,
}

#[derive(EnumDiscriminants, Serialize, Deserialize, PartialEq)]
pub enum FuncArgumentContent {
    V1(FuncArgumentContentV1),
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct FuncArgumentContentV1 {
    pub name: String,
    pub kind: FuncArgumentKind,
    pub element_kind: Option<FuncArgumentKind>,
    pub timestamp: Timestamp,
}

impl From<FuncArgument> for FuncArgumentContentV1 {
    fn from(value: FuncArgument) -> Self {
        Self {
            name: value.name,
            kind: value.kind,
            element_kind: value.element_kind,
            timestamp: value.timestamp,
        }
    }
}

impl FuncArgument {
    pub fn assemble(node_weight: &ContentNodeWeight, content: &FuncArgumentContentV1) -> Self {
        let content = content.to_owned();

        Self {
            id: node_weight.id().into(),
            name: content.name,
            kind: content.kind,
            element_kind: content.element_kind,
            timestamp: content.timestamp,
        }
    }

    pub async fn new(
        ctx: &DalContext,
        name: impl Into<String>,
        kind: FuncArgumentKind,
        element_kind: Option<FuncArgumentKind>,
        func_id: FuncId,
    ) -> FuncArgumentResult<Self> {
        let timestamp = Timestamp::now();

        let content = FuncArgumentContentV1 {
            name: name.into(),
            kind,
            element_kind,
            timestamp,
        };

        let hash = ctx
            .content_store()
            .try_lock()?
            .add(&FuncArgumentContent::V1(content.clone()))?;

        let change_set = ctx.change_set_pointer()?;
        let id = change_set.generate_ulid()?;
        let node_weight = NodeWeight::new_content(change_set, id, ContentAddress::FuncArg(hash))?;

        let mut workspace_snapshot = ctx.workspace_snapshot()?.try_lock()?;

        let _func_arg_node_index = workspace_snapshot.add_node(node_weight.clone())?;

        workspace_snapshot.add_edge(
            func_id.into(),
            EdgeWeight::new(change_set, EdgeWeightKind::Use)?,
            id,
        )?;

        let content_node_weight =
            node_weight.get_content_node_weight_of_kind(ContentAddressDiscriminants::FuncArg)?;

        Ok(FuncArgument::assemble(&content_node_weight, &content))
    }

    pub async fn get_by_id(ctx: &DalContext, id: FuncArgumentId) -> FuncArgumentResult<Self> {
        let mut workspace_snapshot = ctx.workspace_snapshot()?.try_lock()?;
        let id: ulid::Ulid = id.into();
        let node_index = workspace_snapshot.get_node_index_by_id(id)?;
        let node_weight = workspace_snapshot.get_node_weight(node_index)?;
        let hash = node_weight.content_hash();

        let content: FuncArgumentContent = ctx
            .content_store()
            .try_lock()?
            .get(&hash)
            .await?
            .ok_or(WorkspaceSnapshotError::MissingContentFromStore(id))?;

        // NOTE(nick,jacob,zack): if we had a v2, then there would be migration logic here.
        let FuncArgumentContent::V1(inner) = content;

        let arg_node_weight =
            node_weight.get_content_node_weight_of_kind(ContentAddressDiscriminants::FuncArg)?;

        Ok(FuncArgument::assemble(&arg_node_weight, &inner))
    }

    pub fn list_ids_for_func(
        ctx: &DalContext,
        func_id: FuncId,
    ) -> FuncArgumentResult<Vec<FuncArgumentId>> {
        let mut func_args = vec![];

        let mut workspace_snapshot = ctx.workspace_snapshot()?.try_lock()?;

        let func_node_idx = workspace_snapshot.get_node_index_by_id(func_id)?;

        let func_arg_node_idxs = workspace_snapshot
            .outgoing_targets_for_edge_weight_kind_by_index(
                func_node_idx,
                EdgeWeightKindDiscriminants::Use,
            )?;

        for idx in func_arg_node_idxs {
            let node_weight = workspace_snapshot.get_node_weight(idx)?;
            func_args.push(node_weight.id().into())
        }

        Ok(func_args)
    }

    /// List all [`FuncArgument`](Self) for the provided [`FuncId`](crate::FuncId).
    pub async fn list_for_func(ctx: &DalContext, func_id: FuncId) -> FuncArgumentResult<Vec<Self>> {
        let mut func_args = vec![];

        let mut workspace_snapshot = ctx.workspace_snapshot()?.try_lock()?;

        let func_node_idx = workspace_snapshot.get_node_index_by_id(func_id)?;

        let func_arg_node_idxs = workspace_snapshot
            .outgoing_targets_for_edge_weight_kind_by_index(
                func_node_idx,
                EdgeWeightKindDiscriminants::Use,
            )?;

        let mut arg_node_weights = vec![];
        let mut arg_content_hashes = vec![];

        for idx in func_arg_node_idxs {
            let node_weight = workspace_snapshot
                .get_node_weight(idx)?
                .get_content_node_weight_of_kind(ContentAddressDiscriminants::FuncArg)?;

            arg_content_hashes.push(node_weight.content_hash());
            arg_node_weights.push(node_weight);
        }

        let arg_contents: HashMap<ContentHash, FuncArgumentContent> = ctx
            .content_store()
            .try_lock()?
            .get_bulk(arg_content_hashes.as_slice())
            .await?;

        for weight in arg_node_weights {
            match arg_contents.get(&weight.content_hash()) {
                Some(arg_content) => {
                    let FuncArgumentContent::V1(inner) = arg_content;

                    func_args.push(FuncArgument::assemble(&weight, inner));
                }
                None => Err(WorkspaceSnapshotError::MissingContentFromStore(weight.id()))?,
            }
        }

        Ok(func_args)
    }

    pub async fn find_by_name_for_func(
        ctx: &DalContext,
        name: impl AsRef<str>,
        func_id: FuncId,
    ) -> FuncArgumentResult<Option<Self>> {
        let name = name.as_ref();

        for arg in FuncArgument::list_for_func(ctx, func_id).await? {
            if arg.name.as_str() == name {
                return Ok(Some(arg));
            }
        }

        Ok(None)
    }

    pub async fn modify_by_id<L>(
        ctx: &DalContext,
        id: FuncArgumentId,
        lambda: L,
    ) -> FuncArgumentResult<FuncArgument>
    where
        L: FnOnce(&mut FuncArgument) -> FuncArgumentResult<()>,
    {
        let mut workspace_snapshot = ctx.workspace_snapshot()?.try_lock()?;

        let ulid: Ulid = id.into();

        let arg_node_idx = workspace_snapshot.get_node_index_by_id(ulid)?;
        let arg_nw = workspace_snapshot.get_node_weight(arg_node_idx)?;
        let hash = arg_nw.content_hash();

        let content: FuncArgumentContent = ctx
            .content_store()
            .try_lock()?
            .get(&hash)
            .await?
            .ok_or(WorkspaceSnapshotError::MissingContentFromStore(ulid))?;

        let FuncArgumentContent::V1(inner) = content;

        let arg_content_nw =
            arg_nw.get_content_node_weight_of_kind(ContentAddressDiscriminants::FuncArg)?;

        let mut func_arg = FuncArgument::assemble(&arg_content_nw, &inner);

        lambda(&mut func_arg)?;

        let updated = FuncArgumentContentV1::from(func_arg.clone());
        if updated != inner {
            let hash = ctx
                .content_store()
                .try_lock()?
                .add(&FuncArgumentContent::V1(updated.clone()))?;

            workspace_snapshot.update_content(ctx.change_set_pointer()?, ulid, hash)?;
        }

        Ok(FuncArgument::assemble(&arg_content_nw, &updated))
    }

    pub fn remove(ctx: &DalContext, id: FuncArgumentId) -> FuncArgumentResult<()> {
        // to remove a func argument we must remove all incoming edges to it. It will then be
        // garbage collected out of the graph

        let mut workspace_snapshot = ctx.workspace_snapshot()?.try_lock()?;
        workspace_snapshot.remove_incoming_edges_of_kind(
            ctx.change_set_pointer()?,
            id,
            EdgeWeightKindDiscriminants::Use,
        )?;

        // TODO: Note we must also delete the attribute prototype arguments that depend on this func
        // argument

        Ok(())
    }

    //     /// List all [`FuncArgument`](Self) for the provided [`FuncId`](crate::FuncId) along with the
    //     /// [`AttributePrototypeArgument`](crate::AttributePrototypeArgument) that corresponds to it
    //     /// *if* one exists.
    //     pub async fn list_for_func_with_prototype_arguments(
    //         ctx: &DalContext,
    //         func_id: FuncId,
    //         attribute_prototype_id: AttributePrototypeId,
    //     ) -> FuncArgumentResult<Vec<(Self, Option<AttributePrototypeArgument>)>> {
    //         let rows = ctx
    //             .txns()
    //             .await?
    //             .pg()
    //             .query(
    //         Ok(
    //             match ctx
    //                 .txns()
    //                 .await?
    //                 .pg()
    //                 .query_opt(
    //                     FIND_BY_NAME_FOR_FUNC,
    //                     &[ctx.tenancy(), ctx.visibility(), &name, &func_id],
    //                 )
    //                 .await?
    //             {
    //                 Some(row) => standard_model::object_from_row(row)?,
    //                 None => None,
    //             },
    //         )
    //                 LIST_FOR_FUNC_WITH_PROTOTYPE_ARGUMENTS,
    //                 &[
    //                     ctx.tenancy(),
    //                     ctx.visibility(),
    //                     &func_id,
    //                     &attribute_prototype_id,
    //                 ],
    //             )
    //             .await?;
    //
    //         let mut result = vec![];
    //
    //         for row in rows.into_iter() {
    //             let func_argument_json: serde_json::Value = row.try_get("func_argument_object")?;
    //             let prototype_argument_json: Option<serde_json::Value> =
    //                 row.try_get("prototype_argument_object")?;
    //
    //             result.push((
    //                 serde_json::from_value(func_argument_json)?,
    //                 match prototype_argument_json {
    //                     Some(prototype_argument_json) => {
    //                         Some(serde_json::from_value(prototype_argument_json)?)
    //                     }
    //                     None => None,
    //                 },
    //             ));
    //         }
    //
    //         Ok(result)
    //     }

    //     pub async fn find_by_name_for_func(
    //         ctx: &DalContext,
    //         name: &str,
    //         func_id: FuncId,
    //     ) -> FuncArgumentResult<Option<Self>> {
    //     }
}
