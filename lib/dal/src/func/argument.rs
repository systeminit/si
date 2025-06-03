use std::{
    collections::HashMap,
    sync::Arc,
};

use postgres_types::{
    FromSql,
    ToSql,
};
use serde::{
    Deserialize,
    Serialize,
};
use si_events::{
    ContentHash,
    Timestamp,
};
use si_pkg::FuncArgumentKind as PkgFuncArgumentKind;
use strum::{
    AsRefStr,
    Display,
    EnumIter,
    EnumString,
};
use telemetry::prelude::*;
use thiserror::Error;

use crate::{
    DalContext,
    EdgeWeightKind,
    Func,
    FuncError,
    FuncId,
    PropKind,
    TransactionsError,
    attribute::prototype::argument::{
        AttributePrototypeArgument,
        AttributePrototypeArgumentError,
        AttributePrototypeArgumentId,
    },
    change_set::ChangeSetError,
    layer_db_types::{
        FuncArgumentContent,
        FuncArgumentContentV1,
    },
    workspace_snapshot::{
        WorkspaceSnapshotError,
        edge_weight::EdgeWeightKindDiscriminants,
        node_weight::{
            FuncArgumentNodeWeight,
            NodeWeight,
            NodeWeightDiscriminants,
            NodeWeightError,
        },
    },
};

#[remain::sorted]
#[derive(Debug, Error)]
pub enum FuncArgumentError {
    #[error("attribute prototype argument error: {0}")]
    AttributePrototypeArgument(#[from] Box<AttributePrototypeArgumentError>),
    #[error("change set error: {0}")]
    ChangeSet(#[from] ChangeSetError),
    #[error("unable to create func argument with empty name")]
    EmptyNameDuringCreation,
    #[error("func error: {0}")]
    Func(#[from] FuncError),
    #[error("func id not found for func arg id: {0}")]
    FuncIdNotFound(FuncArgumentId),
    #[error("layer db error: {0}")]
    LayerDb(#[from] si_layer_cache::LayerDbError),
    #[error("func {0} ({1}) missing func argument edge")]
    MissingArgumentEdge(String, FuncId),
    #[error("node weight error: {0}")]
    NodeWeight(#[from] NodeWeightError),
    #[error("func argument not found with name {0} for Func {1}")]
    NotFoundByNameForFunc(String, FuncId),
    #[error("pg error: {0}")]
    Pg(#[from] si_data_pg::PgError),
    #[error("error serializing/deserializing json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("func {0} ({1}) has too many args. Expected ({2})")]
    TooManyArguments(String, FuncId, usize),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error("could not acquire lock: {0}")]
    TryLock(#[from] tokio::sync::TryLockError),
    #[error("workspace snapshot: {0}")]
    WorkspaceSnapshot(#[from] WorkspaceSnapshotError),
}

type FuncArgumentResult<T> = Result<T, FuncArgumentError>;

/// To ensure we don't break the enum deserialization
/// with postcard, DO *NOT* add new types to this list in alphabetical order.
/// Add them to the *END* of the enum *ONLY*.
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
    Float,
    Integer,
    Json,
    Map,
    Object,
    String,
}

impl From<FuncArgumentKind> for si_events::FuncArgumentKind {
    fn from(func_argument_kind: FuncArgumentKind) -> Self {
        match func_argument_kind {
            FuncArgumentKind::Any => si_events::FuncArgumentKind::Any,
            FuncArgumentKind::Array => si_events::FuncArgumentKind::Array,
            FuncArgumentKind::Boolean => si_events::FuncArgumentKind::Boolean,
            FuncArgumentKind::Float => si_events::FuncArgumentKind::Float,
            FuncArgumentKind::Integer => si_events::FuncArgumentKind::Integer,
            FuncArgumentKind::Json => si_events::FuncArgumentKind::Json,
            FuncArgumentKind::Map => si_events::FuncArgumentKind::Map,
            FuncArgumentKind::Object => si_events::FuncArgumentKind::Object,
            FuncArgumentKind::String => si_events::FuncArgumentKind::String,
        }
    }
}

impl From<PropKind> for FuncArgumentKind {
    fn from(prop_kind: PropKind) -> Self {
        match prop_kind {
            PropKind::Json => FuncArgumentKind::Json,
            PropKind::Array => FuncArgumentKind::Array,
            PropKind::Boolean => FuncArgumentKind::Boolean,
            PropKind::Integer => FuncArgumentKind::Integer,
            PropKind::Float => FuncArgumentKind::Float,
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
            PkgFuncArgumentKind::Json => FuncArgumentKind::Json,
            PkgFuncArgumentKind::Array => FuncArgumentKind::Array,
            PkgFuncArgumentKind::Boolean => FuncArgumentKind::Boolean,
            PkgFuncArgumentKind::Float => FuncArgumentKind::Float,
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
            FuncArgumentKind::Float => PkgFuncArgumentKind::Float,
            FuncArgumentKind::Integer => PkgFuncArgumentKind::Integer,
            FuncArgumentKind::Map => PkgFuncArgumentKind::Map,
            FuncArgumentKind::Object => PkgFuncArgumentKind::Object,
            FuncArgumentKind::Json => PkgFuncArgumentKind::Json,
            FuncArgumentKind::String => PkgFuncArgumentKind::String,
        }
    }
}

impl From<si_frontend_types::FuncArgumentKind> for FuncArgumentKind {
    fn from(value: si_frontend_types::FuncArgumentKind) -> Self {
        match value {
            si_frontend_types::FuncArgumentKind::Any => FuncArgumentKind::Any,
            si_frontend_types::FuncArgumentKind::Array => FuncArgumentKind::Array,
            si_frontend_types::FuncArgumentKind::Boolean => FuncArgumentKind::Boolean,
            si_frontend_types::FuncArgumentKind::Float => FuncArgumentKind::Float,
            si_frontend_types::FuncArgumentKind::Integer => FuncArgumentKind::Integer,
            si_frontend_types::FuncArgumentKind::Json => FuncArgumentKind::Json,
            si_frontend_types::FuncArgumentKind::Map => FuncArgumentKind::Map,
            si_frontend_types::FuncArgumentKind::Object => FuncArgumentKind::Object,
            si_frontend_types::FuncArgumentKind::String => FuncArgumentKind::String,
        }
    }
}

pub use si_id::FuncArgumentId;

impl From<FuncArgumentKind> for si_frontend_types::FuncArgumentKind {
    fn from(value: FuncArgumentKind) -> Self {
        match value {
            FuncArgumentKind::Any => si_frontend_types::FuncArgumentKind::Any,
            FuncArgumentKind::Array => si_frontend_types::FuncArgumentKind::Array,
            FuncArgumentKind::Boolean => si_frontend_types::FuncArgumentKind::Boolean,
            FuncArgumentKind::Float => si_frontend_types::FuncArgumentKind::Float,
            FuncArgumentKind::Integer => si_frontend_types::FuncArgumentKind::Integer,
            FuncArgumentKind::Json => si_frontend_types::FuncArgumentKind::Json,
            FuncArgumentKind::Map => si_frontend_types::FuncArgumentKind::Map,
            FuncArgumentKind::Object => si_frontend_types::FuncArgumentKind::Object,
            FuncArgumentKind::String => si_frontend_types::FuncArgumentKind::String,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct FuncArgument {
    pub id: FuncArgumentId,
    pub name: String,
    pub kind: FuncArgumentKind,
    pub element_kind: Option<FuncArgumentKind>,
    #[serde(flatten)]
    pub timestamp: Timestamp,
}

impl From<FuncArgument> for FuncArgumentContentV1 {
    fn from(value: FuncArgument) -> Self {
        Self {
            kind: value.kind,
            element_kind: value.element_kind,
            timestamp: value.timestamp,
        }
    }
}

impl FuncArgument {
    pub fn assemble(node_weight: &FuncArgumentNodeWeight, content: &FuncArgumentContentV1) -> Self {
        let content = content.to_owned();

        Self {
            id: node_weight.id().into(),
            name: node_weight.name().into(),
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
        let name = name.into();
        if name.is_empty() {
            return Err(FuncArgumentError::EmptyNameDuringCreation);
        }

        let timestamp = Timestamp::now();

        let content = FuncArgumentContentV1 {
            kind,
            element_kind,
            timestamp,
        };

        let (hash, _) = ctx.layer_db().cas().write(
            Arc::new(FuncArgumentContent::V1(content.clone()).into()),
            None,
            ctx.events_tenancy(),
            ctx.events_actor(),
        )?;

        let workspace_snapshot = ctx.workspace_snapshot()?;
        let id = workspace_snapshot.generate_ulid().await?;
        let lineage_id = workspace_snapshot.generate_ulid().await?;
        let node_weight = NodeWeight::new_func_argument(id, lineage_id, name, hash);

        workspace_snapshot
            .add_or_replace_node(node_weight.clone())
            .await?;
        Func::add_edge_to_argument(ctx, func_id, id.into(), EdgeWeightKind::new_use()).await?;

        let func_argument_node_weight = node_weight.get_func_argument_node_weight()?;

        Ok(FuncArgument::assemble(&func_argument_node_weight, &content))
    }

    pub async fn get_by_id_opt(
        ctx: &DalContext,
        id: FuncArgumentId,
    ) -> FuncArgumentResult<Option<Self>> {
        let (node_weight, hash) = if let Some((node_weight, hash)) =
            Self::get_node_weight_and_content_hash(ctx, id).await?
        {
            (node_weight, hash)
        } else {
            return Ok(None);
        };

        let func_argument = Self::get_by_id_inner(ctx, &hash, &node_weight).await?;
        Ok(Some(func_argument))
    }

    pub async fn get_by_id(ctx: &DalContext, id: FuncArgumentId) -> FuncArgumentResult<Self> {
        let (node_weight, hash) = Self::get_node_weight_and_content_hash_or_error(ctx, id).await?;
        Self::get_by_id_inner(ctx, &hash, &node_weight).await
    }

    async fn get_by_id_inner(
        ctx: &DalContext,
        hash: &ContentHash,
        node_weight: &FuncArgumentNodeWeight,
    ) -> FuncArgumentResult<Self> {
        let content: FuncArgumentContent = ctx.layer_db().cas().try_read_as(hash).await?.ok_or(
            WorkspaceSnapshotError::MissingContentFromStore(node_weight.id()),
        )?;

        // NOTE(nick,jacob,zack): if we had a v2, then there would be migration logic here.
        let FuncArgumentContent::V1(inner) = content;

        Ok(Self::assemble(node_weight, &inner))
    }

    pub async fn get_name_by_id(
        ctx: &DalContext,
        func_arg_id: FuncArgumentId,
    ) -> FuncArgumentResult<String> {
        let node_weight = ctx
            .workspace_snapshot()?
            .get_node_weight(func_arg_id)
            .await?;
        let func_arg_node_weight = node_weight.get_func_argument_node_weight()?;
        let name = func_arg_node_weight.name().to_string();
        Ok(name)
    }

    pub async fn list_ids_for_func(
        ctx: &DalContext,
        func_id: FuncId,
    ) -> FuncArgumentResult<Vec<FuncArgumentId>> {
        let mut func_args = vec![];

        let workspace_snapshot = ctx.workspace_snapshot()?;

        let func_arg_node_idxs = workspace_snapshot
            .outgoing_targets_for_edge_weight_kind(func_id, EdgeWeightKindDiscriminants::Use)
            .await?;

        for idx in func_arg_node_idxs {
            let node_weight = workspace_snapshot
                .get_node_weight(idx)
                .await?
                .get_func_argument_node_weight()?;
            func_args.push(node_weight.id().into())
        }

        Ok(func_args)
    }

    pub async fn single_arg_for_func(
        ctx: &DalContext,
        func_id: FuncId,
    ) -> FuncArgumentResult<FuncArgumentId> {
        let args = FuncArgument::list_ids_for_func(ctx, func_id).await?;

        if args.len() > 1 {
            return Err(FuncArgumentError::TooManyArguments(
                Func::get_by_id(ctx, func_id).await?.name.to_owned(),
                func_id,
                1,
            ));
        }

        match FuncArgument::list_ids_for_func(ctx, func_id).await?.first() {
            Some(&arg_id) => Ok(arg_id),
            None => Err(FuncArgumentError::MissingArgumentEdge(
                Func::get_by_id(ctx, func_id).await?.name.to_owned(),
                func_id,
            )),
        }
    }

    pub async fn get_func_id_for_func_arg_id(
        ctx: &DalContext,
        func_arg_id: FuncArgumentId,
    ) -> FuncArgumentResult<FuncId> {
        let workspace_snapshot = ctx.workspace_snapshot()?;
        let func_id_node_idx = workspace_snapshot
            .incoming_sources_for_edge_weight_kind(func_arg_id, EdgeWeightKindDiscriminants::Use)
            .await?;

        if let Some(idx) = func_id_node_idx.into_iter().next() {
            let node_weight = workspace_snapshot
                .get_node_weight(idx)
                .await?
                .get_func_node_weight()?;
            return Ok(node_weight.id().into());
        }

        Err(FuncArgumentError::FuncIdNotFound(func_arg_id))
    }

    /// List all [`FuncArgument`](Self) for the provided [`FuncId`](crate::FuncId).
    pub async fn list_for_func(ctx: &DalContext, func_id: FuncId) -> FuncArgumentResult<Vec<Self>> {
        let workspace_snapshot = ctx.workspace_snapshot()?;

        let func_arg_node_idxs = workspace_snapshot
            .outgoing_targets_for_edge_weight_kind(func_id, EdgeWeightKindDiscriminants::Use)
            .await?;

        let mut arg_node_weights = vec![];
        let mut arg_content_hashes = vec![];

        for idx in func_arg_node_idxs {
            let node_weight = workspace_snapshot
                .get_node_weight(idx)
                .await?
                .get_func_argument_node_weight()?;

            arg_content_hashes.push(node_weight.content_hash());
            arg_node_weights.push(node_weight);
        }

        let arg_contents: HashMap<ContentHash, FuncArgumentContent> = ctx
            .layer_db()
            .cas()
            .try_read_many_as(arg_content_hashes.as_slice())
            .await?;

        let mut func_args = vec![];
        for weight in arg_node_weights {
            match arg_contents.get(&weight.content_hash()) {
                Some(arg_content) => {
                    let FuncArgumentContent::V1(inner) = arg_content;

                    func_args.push(Self::assemble(&weight, inner));
                }
                None => Err(WorkspaceSnapshotError::MissingContentFromStore(weight.id()))?,
            }
        }

        Ok(func_args)
    }

    /// Find the [`FuncArgument`] by its name for a given [`Func`]. For a given [`Func`], all argument names must be
    /// unique. This method returns `None` if no argument was found.
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
        let func_argument = Self::get_by_id(ctx, id).await?;
        let modified_func_argument = func_argument.modify(ctx, lambda).await?;
        Ok(modified_func_argument)
    }

    pub async fn modify<L>(self, ctx: &DalContext, lambda: L) -> FuncArgumentResult<Self>
    where
        L: FnOnce(&mut Self) -> FuncArgumentResult<()>,
    {
        let mut func_argument = self;

        let before = FuncArgumentContentV1::from(func_argument.clone());
        lambda(&mut func_argument)?;

        let (mut node_weight, _) =
            FuncArgument::get_node_weight_and_content_hash_or_error(ctx, func_argument.id).await?;

        let workspace_snapshot = ctx.workspace_snapshot()?;

        // If the name HAS changed, *and* parts of the FuncArgumentContent
        // have changed, this ends up updating the node for the function twice. This could be
        // optimized to do it only once.
        if func_argument.name.as_str() != node_weight.name() {
            node_weight.set_name(func_argument.name.as_str());

            workspace_snapshot
                .add_or_replace_node(NodeWeight::FuncArgument(node_weight.clone()))
                .await?;
        }
        let updated = FuncArgumentContentV1::from(func_argument.clone());

        if updated != before {
            let (hash, _) = ctx.layer_db().cas().write(
                Arc::new(FuncArgumentContent::V1(updated.clone()).into()),
                None,
                ctx.events_tenancy(),
                ctx.events_actor(),
            )?;
            workspace_snapshot
                .update_content(func_argument.id.into(), hash)
                .await?;
        }

        Ok(Self::assemble(&node_weight, &updated))
    }

    async fn get_node_weight_and_content_hash(
        ctx: &DalContext,
        id: FuncArgumentId,
    ) -> FuncArgumentResult<Option<(FuncArgumentNodeWeight, ContentHash)>> {
        let workspace_snapshot = ctx.workspace_snapshot()?;

        let Some(node_weight) = workspace_snapshot.get_node_weight_opt(id).await else {
            return Ok(None);
        };

        let hash = node_weight.content_hash();
        let func_argument_node_weight = node_weight.get_func_argument_node_weight()?;
        Ok(Some((func_argument_node_weight, hash)))
    }

    async fn get_node_weight_and_content_hash_or_error(
        ctx: &DalContext,
        id: FuncArgumentId,
    ) -> FuncArgumentResult<(FuncArgumentNodeWeight, ContentHash)> {
        let workspace_snapshot = ctx.workspace_snapshot()?;

        let node_weight = workspace_snapshot.get_node_weight(id).await?;

        let hash = node_weight.content_hash();
        let func_argument_node_weight = node_weight.get_func_argument_node_weight()?;
        Ok((func_argument_node_weight, hash))
    }

    pub async fn remove(ctx: &DalContext, id: FuncArgumentId) -> FuncArgumentResult<()> {
        // If a func argument is to be deleted, we need to remove all attribute prototype
        // arguments that use it first.
        for attribute_prototype_argument_id in
            Self::list_attribute_prototype_argument_ids(ctx, id).await?
        {
            AttributePrototypeArgument::remove(ctx, attribute_prototype_argument_id)
                .await
                .map_err(Box::new)?;
        }

        // Now, we can remove the argument.
        ctx.workspace_snapshot()?.remove_node_by_id(id).await?;

        Ok(())
    }

    /// List all [`AttributePrototypeArguments`](AttributePrototypeArgument) (by ID) using the
    /// provided [`FuncArgument`] (by ID).
    pub async fn list_attribute_prototype_argument_ids(
        ctx: &DalContext,
        id: FuncArgumentId,
    ) -> FuncArgumentResult<Vec<AttributePrototypeArgumentId>> {
        let workspace_snapshot = ctx.workspace_snapshot()?;

        let sources = workspace_snapshot
            .incoming_sources_for_edge_weight_kind(id, EdgeWeightKindDiscriminants::Use)
            .await?;

        let mut attribute_prototype_argument_ids = Vec::new();
        for source in sources {
            let node_weight = workspace_snapshot.get_node_weight(source).await?;
            let maybe_attribute_prototype_argument_id = node_weight.id().into();
            if NodeWeightDiscriminants::AttributePrototypeArgument == node_weight.into() {
                attribute_prototype_argument_ids.push(maybe_attribute_prototype_argument_id);
            }
        }

        Ok(attribute_prototype_argument_ids)
    }
}
