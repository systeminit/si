use argument::{FuncArgument, FuncArgumentError};
use authoring::{FuncAuthoringClient, FuncAuthoringError};
use base64::{engine::general_purpose, Engine};
use binding::{FuncBinding, FuncBindingError};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use si_events::CasValue;
use si_events::{ulid::Ulid, ContentHash};
use si_frontend_types::FuncSummary;
use si_pkg::SpecError;
use std::collections::HashMap;
use std::string::FromUtf8Error;
use std::sync::Arc;
use strum::IntoEnumIterator;
use telemetry::prelude::*;
use thiserror::Error;
use ulid::Ulid as CoreUlid;

use crate::change_set::ChangeSetError;
use crate::func::argument::FuncArgumentId;
use crate::func::intrinsics::IntrinsicFunc;
use crate::layer_db_types::{FuncContent, FuncContentV2};
use crate::workspace_snapshot::edge_weight::{EdgeWeightKind, EdgeWeightKindDiscriminants};
use crate::workspace_snapshot::graph::WorkspaceSnapshotGraphError;
use crate::workspace_snapshot::node_weight::category_node_weight::CategoryNodeKind;
use crate::workspace_snapshot::node_weight::{FuncNodeWeight, NodeWeight, NodeWeightError};
use crate::workspace_snapshot::WorkspaceSnapshotError;
use crate::{
    implement_add_edge_to, pkg, ChangeSetId, DalContext, HelperError, Timestamp, TransactionsError,
    WsEvent, WsEventResult, WsPayload,
};

use self::backend::{FuncBackendKind, FuncBackendResponseType};

pub mod argument;
pub mod authoring;
pub mod backend;
pub mod binding;
pub mod intrinsics;
mod kind;
pub mod runner;

pub use kind::FuncKind;

#[remain::sorted]
#[derive(Error, Debug)]
pub enum FuncError {
    #[error("attribute value error: {0}")]
    AttributeValue(String),
    #[error("base64 decode error: {0}")]
    Base64Decode(#[from] base64::DecodeError),
    #[error("change set error: {0}")]
    ChangeSet(#[from] ChangeSetError),
    #[error("chrono parse error: {0}")]
    ChronoParse(#[from] chrono::ParseError),
    #[error("func argument error: {0}")]
    FuncArgument(#[from] Box<FuncArgumentError>),
    #[error("func authoring client error: {0}")]
    FuncAuthoringClient(#[from] Box<FuncAuthoringError>),
    #[error("func bindings error: {0}")]
    FuncBinding(#[from] Box<FuncBindingError>),
    #[error("func bindings can't be found: {0}")]
    FuncBindingsLookup(FuncId),
    #[error("cannot modify locked func: {0}")]
    FuncLocked(FuncId),
    #[error("func name already in use {0}")]
    FuncNameInUse(String),
    #[error("func to be deleted has bindings: {0}")]
    FuncToBeDeletedHasBindings(FuncId),
    #[error("helper error: {0}")]
    Helper(#[from] HelperError),
    #[error("cannot find intrinsic func {0}")]
    IntrinsicFuncNotFound(String),
    #[error("intrinsic spec creation error: {0}")]
    IntrinsicSpecCreation(#[source] SpecError),
    #[error("layer db error: {0}")]
    LayerDb(#[from] si_layer_cache::LayerDbError),
    #[error("node weight error: {0}")]
    NodeWeight(#[from] NodeWeightError),
    #[error("si pkg error: {0}")]
    Pkg(#[from] Box<pkg::PkgError>),
    #[error("pkg error: {0}")]
    SiPkg(#[from] si_pkg::SiPkgError),
    #[error("pkg spec error: {0}")]
    Spec(#[from] si_pkg::SpecError),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error("could not acquire lock: {0}")]
    TryLock(#[from] tokio::sync::TryLockError),
    #[error("utf8 error: {0}")]
    Utf8(#[from] FromUtf8Error),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] WorkspaceSnapshotError),
}

pub type FuncResult<T> = Result<T, FuncError>;

impl From<Func> for FuncContent {
    fn from(value: Func) -> Self {
        Self::V2(FuncContentV2 {
            timestamp: value.timestamp,
            display_name: value.display_name,
            description: value.description,
            link: value.link,
            hidden: value.hidden,
            builtin: value.builtin,
            backend_response_type: value.backend_response_type,
            backend_kind: value.backend_kind,
            handler: value.handler,
            code_base64: value.code_base64,
            code_blake3: value.code_blake3,
            is_locked: value.is_locked,
        })
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct FuncMetadataView {
    pub display_name: String,
    pub description: Option<String>,
    pub link: Option<String>,
}

pub fn is_intrinsic(name: &str) -> bool {
    IntrinsicFunc::iter().any(|intrinsic| intrinsic.name() == name)
}

pub use si_id::FuncId;

// NOTE: This is here only for backward compatibility
pub use si_id::FuncExecutionPk;

/// A `Func` is the declaration of the existence of a function. It has a name,
/// and corresponds to a given function backend (and its associated return types).
///
/// `handler` is the name of the entry point into the code in `code_base64`.
/// For example, if we had a code block of
/// `function myValidator(actual, expected) { return true; }` in `code_base64`,
/// the `handler` value should be `myValidator`.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct Func {
    pub id: FuncId,
    pub name: String,
    pub kind: FuncKind,

    pub timestamp: Timestamp,
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub link: Option<String>,
    pub hidden: bool,
    pub builtin: bool,
    pub backend_kind: FuncBackendKind,
    pub backend_response_type: FuncBackendResponseType,
    pub handler: Option<String>,
    pub code_base64: Option<String>,
    pub code_blake3: ContentHash,
    pub is_locked: bool,
}

impl Func {
    pub fn assemble(node_weight: &FuncNodeWeight, content: FuncContentV2) -> Self {
        Self {
            id: node_weight.id().into(),
            name: node_weight.name().to_owned(),
            kind: node_weight.func_kind(),

            timestamp: content.timestamp,
            display_name: content.display_name,
            description: content.description,
            link: content.link,
            hidden: content.hidden,
            builtin: content.builtin,
            backend_kind: content.backend_kind,
            backend_response_type: content.backend_response_type,
            handler: content.handler,
            code_base64: content.code_base64,
            code_blake3: content.code_blake3,
            is_locked: content.is_locked,
        }
    }

    implement_add_edge_to!(
        source_id: FuncId,
        destination_id: FuncArgumentId,
        add_fn: add_edge_to_argument,
        discriminant: EdgeWeightKindDiscriminants::Use,
        result: FuncResult,
    );
    implement_add_edge_to!(
        source_id: Ulid,
        destination_id: FuncId,
        add_fn: add_category_edge,
        discriminant: EdgeWeightKindDiscriminants::Use,
        result: FuncResult,
    );

    #[allow(clippy::too_many_arguments)]
    pub async fn new(
        ctx: &DalContext,
        name: impl Into<String> + Clone,
        display_name: Option<impl Into<String>>,
        description: Option<impl Into<String>>,
        link: Option<impl Into<String>>,
        hidden: bool,
        builtin: bool,
        backend_kind: FuncBackendKind,
        backend_response_type: FuncBackendResponseType,
        handler: Option<impl Into<String>>,
        code_base64: Option<impl Into<String>>,
    ) -> FuncResult<Self> {
        let timestamp = Timestamp::now();
        let _finalized_once = false;

        let code_base64: Option<String> = code_base64.map(Into::into);
        let code_blake3 = if let Some(code) = code_base64.as_ref() {
            let code_json_value: serde_json::Value = code.clone().into();
            let code_cas_value: CasValue = code_json_value.into();
            let (hash, _) = ctx.layer_db().cas().write(
                Arc::new(code_cas_value.into()),
                None,
                ctx.events_tenancy(),
                ctx.events_actor(),
            )?;
            hash
        } else {
            // Why are we doing this? Because the struct gods demand it. I have feelings.
            ContentHash::new("".as_bytes())
        };

        let content = FuncContentV2 {
            timestamp,
            display_name: display_name.map(Into::into),
            description: description.map(Into::into),
            link: link.map(Into::into),
            hidden,
            builtin,
            backend_response_type,
            backend_kind,
            handler: handler.map(Into::into),
            code_base64,
            code_blake3,
            is_locked: false,
        };

        let (hash, _) = ctx.layer_db().cas().write(
            Arc::new(FuncContent::V2(content.clone()).into()),
            None,
            ctx.events_tenancy(),
            ctx.events_actor(),
        )?;

        let func_kind = FuncKind::new(backend_kind, backend_response_type)?;

        let id = ctx.workspace_snapshot()?.generate_ulid().await?;
        let lineage_id = ctx.workspace_snapshot()?.generate_ulid().await?;
        let node_weight =
            NodeWeight::new_func(id, lineage_id, name.clone().into(), func_kind, hash);

        let workspace_snapshot = ctx.workspace_snapshot()?;
        workspace_snapshot
            .add_or_replace_node(node_weight.clone())
            .await?;

        let func_category_id = workspace_snapshot
            .get_category_node_or_err(None, CategoryNodeKind::Func)
            .await?;
        Self::add_category_edge(ctx, func_category_id, id.into(), EdgeWeightKind::new_use())
            .await?;

        let func_node_weight = node_weight.get_func_node_weight()?;

        Ok(Self::assemble(&func_node_weight, content))
    }

    pub async fn lock(self, ctx: &DalContext) -> FuncResult<Func> {
        self.modify(ctx, |func| {
            func.is_locked = true;
            Ok(())
        })
        .await
    }

    pub fn metadata_view(&self) -> FuncMetadataView {
        FuncMetadataView {
            display_name: self
                .display_name
                .as_deref()
                .unwrap_or(self.name.as_str())
                .into(),
            description: self.description.as_deref().map(Into::into),
            link: None,
        }
    }

    pub async fn get_by_id_opt(ctx: &DalContext, id: FuncId) -> FuncResult<Option<Self>> {
        let (func_node_weight, hash) = if let Some((func_node_weight, hash)) =
            Self::get_node_weight_and_content_hash(ctx, id).await?
        {
            (func_node_weight, hash)
        } else {
            return Ok(None);
        };

        let func = Self::get_by_id_inner(ctx, &hash, &func_node_weight).await?;
        Ok(Some(func))
    }

    pub async fn get_by_id(ctx: &DalContext, id: FuncId) -> FuncResult<Self> {
        let (func_node_weight, hash) =
            Self::get_node_weight_and_content_hash_or_error(ctx, id).await?;
        Self::get_by_id_inner(ctx, &hash, &func_node_weight).await
    }

    /// If you know the func_id is supposed to be for an [`IntrinsicFunc`], get which one or error
    pub async fn get_intrinsic_kind_by_id_or_error(
        ctx: &DalContext,
        id: FuncId,
    ) -> FuncResult<IntrinsicFunc> {
        let func = Self::get_by_id(ctx, id).await?;

        Self::get_intrinsic_kind_by_id(ctx, id)
            .await?
            .ok_or(FuncError::IntrinsicFuncNotFound(func.name))
    }

    pub async fn get_intrinsic_kind_by_id(
        ctx: &DalContext,
        id: FuncId,
    ) -> FuncResult<Option<IntrinsicFunc>> {
        let func = Self::get_by_id(ctx, id).await?;
        Ok(IntrinsicFunc::maybe_from_str(func.name.clone()))
    }

    async fn get_by_id_inner(
        ctx: &DalContext,
        hash: &ContentHash,
        func_node_weight: &FuncNodeWeight,
    ) -> FuncResult<Self> {
        let content: FuncContent = ctx.layer_db().cas().try_read_as(hash).await?.ok_or(
            WorkspaceSnapshotError::MissingContentFromStore(func_node_weight.id()),
        )?;

        // migrate if necessary!
        let inner: FuncContentV2 = content.extract();

        Ok(Self::assemble(func_node_weight, inner))
    }

    /// Attempt to find the [`FuncId`](Func) by name.
    ///
    /// _Warning:_ [`Func`] names are intentionally not unique. This is a greedy algorithm!
    pub async fn find_id_by_name(
        ctx: &DalContext,
        name: impl AsRef<str>,
    ) -> FuncResult<Option<FuncId>> {
        let workspace_snapshot = ctx.workspace_snapshot()?;
        let func_category_id = workspace_snapshot
            .get_category_node_or_err(None, CategoryNodeKind::Func)
            .await?;
        let func_indices = workspace_snapshot
            .outgoing_targets_for_edge_weight_kind(
                func_category_id,
                EdgeWeightKind::new_use().into(),
            )
            .await?;
        let name = name.as_ref();
        for func_index in func_indices {
            let node_weight = workspace_snapshot.get_node_weight(func_index).await?;
            if let NodeWeight::Func(inner_weight) = node_weight {
                if inner_weight.name() == name {
                    return Ok(Some(inner_weight.id().into()));
                }
            }
        }
        Ok(None)
    }

    /// Attempt to find the [`FuncId`](Func) by name and [kind](FuncKind).
    ///
    /// _Warning:_ [`Func`] names are intentionally not unique. This is a greedy algorithm!
    pub async fn find_id_by_name_and_kind(
        ctx: &DalContext,
        name: impl AsRef<str>,
        kind: FuncKind,
    ) -> FuncResult<Option<FuncId>> {
        let workspace_snapshot = ctx.workspace_snapshot()?;
        let func_category_id = workspace_snapshot
            .get_category_node_or_err(None, CategoryNodeKind::Func)
            .await?;
        let func_indices = workspace_snapshot
            .outgoing_targets_for_edge_weight_kind(
                func_category_id,
                EdgeWeightKind::new_use().into(),
            )
            .await?;
        let name = name.as_ref();
        for func_index in func_indices {
            let node_weight = workspace_snapshot.get_node_weight(func_index).await?;
            if let NodeWeight::Func(inner_weight) = node_weight {
                if inner_weight.name() == name && inner_weight.func_kind() == kind {
                    return Ok(Some(inner_weight.id().into()));
                }
            }
        }
        Ok(None)
    }

    pub fn code_plaintext(&self) -> FuncResult<Option<String>> {
        Ok(match &self.code_base64 {
            Some(base64_code) => Some(String::from_utf8(
                general_purpose::STANDARD_NO_PAD.decode(base64_code)?,
            )?),
            None => None,
        })
    }

    pub fn is_dynamic(&self) -> bool {
        Self::is_dynamic_for_name_string(&self.name)
    }

    pub fn is_intrinsic(&self) -> bool {
        IntrinsicFunc::maybe_from_str(&self.name).is_some()
    }

    /// A non-dynamic Func is an Intrinsic func that returns a fixed value, set by a StaticArgumentValue in the graph
    /// opposingly, a dynamic Func is a func that returns a non statically predictable value, possibly user defined.
    ///
    /// It's important to note that not all Intrinsic funcs are non-dynamic. Identity, for instance, is dynamic.
    pub fn is_dynamic_for_name_string(name: &str) -> bool {
        match IntrinsicFunc::maybe_from_str(name) {
            Some(intrinsic) => match intrinsic {
                IntrinsicFunc::SetArray
                | IntrinsicFunc::SetBoolean
                | IntrinsicFunc::SetInteger
                | IntrinsicFunc::SetFloat
                | IntrinsicFunc::SetJson
                | IntrinsicFunc::SetMap
                | IntrinsicFunc::SetObject
                | IntrinsicFunc::SetString
                | IntrinsicFunc::Unset => false,
                IntrinsicFunc::Identity
                | IntrinsicFunc::NormalizeToArray
                | IntrinsicFunc::ResourcePayloadToValue
                | IntrinsicFunc::Validation => true,
            },
            None => true,
        }
    }

    pub async fn modify_by_id<L>(ctx: &DalContext, id: FuncId, lambda: L) -> FuncResult<Func>
    where
        L: FnOnce(&mut Func) -> FuncResult<()>,
    {
        let func = Func::get_by_id(ctx, id).await?;
        let modified_func = func.modify(ctx, lambda).await?;
        Ok(modified_func)
    }

    pub fn error_if_locked(&self) -> FuncResult<()> {
        if self.is_locked {
            return Err(FuncError::FuncLocked(self.id));
        }
        Ok(())
    }

    async fn get_node_weight_and_content_hash(
        ctx: &DalContext,
        func_id: FuncId,
    ) -> FuncResult<Option<(FuncNodeWeight, ContentHash)>> {
        let workspace_snapshot = ctx.workspace_snapshot()?;
        let id: Ulid = func_id.into();
        let node_index = match workspace_snapshot.get_node_index_by_id(id).await {
            Ok(node_index) => node_index,
            Err(WorkspaceSnapshotError::WorkspaceSnapshotGraph(
                WorkspaceSnapshotGraphError::NodeWithIdNotFound(_),
            )) => return Ok(None),
            Err(err) => return Err(err.into()),
        };
        let node_weight = workspace_snapshot.get_node_weight(node_index).await?;

        let hash = node_weight.content_hash();
        let func_node_weight = node_weight.get_func_node_weight()?;
        Ok(Some((func_node_weight, hash)))
    }

    async fn get_node_weight_and_content_hash_or_error(
        ctx: &DalContext,
        func_id: FuncId,
    ) -> FuncResult<(FuncNodeWeight, ContentHash)> {
        let workspace_snapshot = ctx.workspace_snapshot()?;
        let id: Ulid = func_id.into();
        let node_index = workspace_snapshot.get_node_index_by_id(id).await?;
        let node_weight = workspace_snapshot.get_node_weight(node_index).await?;

        let hash = node_weight.content_hash();
        let func_node_weight = node_weight.get_func_node_weight()?;
        Ok((func_node_weight, hash))
    }

    /// This _unsafely_ unlocks the [`Func`].
    ///
    /// **Warning:** this should only be used on a case-by-case basis and is dangerous. We should
    /// create a _copy_ of a [`Func`] when an unlocked one is desired, by default. If unsure, do
    /// not use this.
    pub async fn unsafe_unlock_without_copy(self, ctx: &DalContext) -> FuncResult<()> {
        let mut func = self;

        let before = FuncContent::from(func.clone());
        func.is_locked = false;
        let updated = FuncContent::from(func.clone());

        if updated != before {
            let (hash, _) = ctx.layer_db().cas().write(
                Arc::new(updated.into()),
                None,
                ctx.events_tenancy(),
                ctx.events_actor(),
            )?;
            ctx.workspace_snapshot()?
                .update_content(func.id.into(), hash)
                .await?;
        }

        Ok(())
    }

    pub async fn modify<L>(self, ctx: &DalContext, lambda: L) -> FuncResult<Self>
    where
        L: FnOnce(&mut Self) -> FuncResult<()>,
    {
        let mut func = self;
        func.error_if_locked()?;
        let before = FuncContent::from(func.clone());
        lambda(&mut func)?;

        let (mut node_weight, _) =
            Self::get_node_weight_and_content_hash_or_error(ctx, func.id).await?;

        let workspace_snapshot = ctx.workspace_snapshot()?;

        // If the name HAS changed, *and* parts of the FuncContent
        // have changed, this ends up updating the node for the function twice. This could be
        // optimized to do it only once.
        if func.name.as_str() != node_weight.name() {
            node_weight.set_name(func.name.as_str());
            workspace_snapshot
                .add_or_replace_node(NodeWeight::Func(node_weight.clone()))
                .await?;
        }
        let updated = FuncContent::from(func.clone());

        if updated != before {
            let (hash, _) = ctx.layer_db().cas().write(
                Arc::new((updated.clone()).into()),
                None,
                ctx.events_tenancy(),
                ctx.events_actor(),
            )?;
            workspace_snapshot
                .update_content(func.id.into(), hash)
                .await?;
        }

        Ok(Self::assemble(&node_weight, updated.extract()))
    }

    /// Deletes the [`Func`] and returns the name.
    pub async fn delete_by_id(ctx: &DalContext, id: FuncId) -> FuncResult<String> {
        let func = Self::get_by_id(ctx, id).await?;
        // Check that we can remove the func.
        if !FuncBinding::for_func_id(ctx, id)
            .await
            .map_err(Box::new)?
            .is_empty()
        {
            return Err(FuncError::FuncToBeDeletedHasBindings(id));
        }

        // Now, we can remove the func.
        let workspace_snapshot = ctx.workspace_snapshot()?;
        workspace_snapshot.remove_node_by_id(id).await?;

        Ok(func.name)
    }

    pub async fn find_intrinsic(ctx: &DalContext, intrinsic: IntrinsicFunc) -> FuncResult<FuncId> {
        let name = intrinsic.name();
        Self::find_id_by_name_and_kind(ctx, name, FuncKind::Intrinsic)
            .await?
            .ok_or(FuncError::IntrinsicFuncNotFound(name.to_owned()))
    }

    /// List all [`Funcs`](Func) in the workspace
    pub async fn list_all(ctx: &DalContext) -> FuncResult<Vec<Self>> {
        let workspace_snapshot = ctx.workspace_snapshot()?;

        let func_category_id = workspace_snapshot
            .get_category_node_or_err(None, CategoryNodeKind::Func)
            .await?;

        let func_node_indexes = workspace_snapshot
            .outgoing_targets_for_edge_weight_kind(
                func_category_id,
                EdgeWeightKindDiscriminants::Use,
            )
            .await?;

        let mut func_node_weights = Vec::new();
        let mut func_content_hashes = Vec::new();
        for index in func_node_indexes {
            let node_weight = workspace_snapshot
                .get_node_weight(index)
                .await?
                .get_func_node_weight()?;
            func_content_hashes.push(node_weight.content_hash());
            func_node_weights.push(node_weight);
        }

        Self::list_inner(ctx, func_node_weights, func_content_hashes).await
    }

    /// List all [`Funcs`](Func) in the workspace that are either unlocked, attached to a default
    /// [`SchemaVariant`] or attached to an unlocked Schema Variant
    pub async fn list_for_default_and_editing(ctx: &DalContext) -> FuncResult<Vec<Self>> {
        let funcs = Self::list_all(ctx).await?;
        let mut pruned_funcs = vec![];
        for func in funcs {
            if func.is_locked {
                match FuncBinding::get_bindings_for_default_schema_variants(ctx, func.id).await {
                    Ok(b) => {
                        if !b.is_empty() {
                            pruned_funcs.push(func);
                        } else {
                            let bindings = FuncBinding::for_func_id(ctx, func.id)
                                .await
                                .map_err(|_| FuncError::FuncBindingsLookup(func.id))?;
                            if bindings.is_empty() {
                                pruned_funcs.push(func)
                            }
                        }
                    }
                    Err(err) => {
                        error!(?err, "could not get bindings for func id {}", func.id)
                    }
                }
            } else {
                pruned_funcs.push(func);
            }
        }
        Ok(pruned_funcs)
    }

    /// List all [`Funcs`](Func) corresponding to the provided [`FuncIds`](Func).
    pub async fn list_from_ids(ctx: &DalContext, func_ids: &[FuncId]) -> FuncResult<Vec<Self>> {
        let workspace_snapshot = ctx.workspace_snapshot()?;

        let mut func_node_weights = Vec::new();
        let mut func_content_hashes = Vec::new();
        for id in func_ids {
            let node_weight = workspace_snapshot
                .get_node_weight_by_id(id)
                .await?
                .get_func_node_weight()?;
            func_content_hashes.push(node_weight.content_hash());
            func_node_weights.push(node_weight);
        }

        Self::list_inner(ctx, func_node_weights, func_content_hashes).await
    }

    async fn list_inner(
        ctx: &DalContext,
        func_node_weights: Vec<FuncNodeWeight>,
        func_content_hashes: Vec<ContentHash>,
    ) -> FuncResult<Vec<Self>> {
        let func_contents: HashMap<ContentHash, FuncContent> = ctx
            .layer_db()
            .cas()
            .try_read_many_as(func_content_hashes.as_slice())
            .await?;

        let mut funcs = Vec::new();
        for node_weight in func_node_weights {
            match func_contents.get(&node_weight.content_hash()) {
                Some(func_content) => {
                    // migrates if needed!
                    let content = func_content.clone().extract();
                    funcs.push(Func::assemble(&node_weight, content));
                }
                None => Err(WorkspaceSnapshotError::MissingContentFromStore(
                    node_weight.id(),
                ))?,
            }
        }

        Ok(funcs)
    }

    /// Creates an exact clone of the current func that is not locked, including recreating all
    /// [`FuncArgument`]s
    pub async fn create_unlocked_func_copy(&self, ctx: &DalContext) -> FuncResult<Self> {
        let new_func = Self::new(
            ctx,
            self.name.clone(),
            self.display_name.clone(),
            self.description.clone(),
            self.link.clone(),
            self.hidden,
            false,
            self.backend_kind,
            self.backend_response_type,
            self.handler.clone(),
            self.code_base64.clone(),
        )
        .await?;

        for arg in FuncArgument::list_for_func(ctx, self.id)
            .await
            .map_err(Box::new)?
        {
            // create new func args for the new func
            FuncArgument::new(ctx, arg.name, arg.kind, arg.element_kind, new_func.id)
                .await
                .map_err(Box::new)?;
        }
        FuncArgument::list_for_func(ctx, new_func.id)
            .await
            .map_err(Box::new)?;
        Ok(new_func)
    }

    pub async fn clone_func_with_new_name(
        &self,
        ctx: &DalContext,
        new_name: String,
    ) -> FuncResult<Self> {
        if new_name == self.name.clone() {
            return Err(FuncError::FuncNameInUse(new_name));
        }

        let duplicated_func = Self::new(
            ctx,
            new_name,
            self.display_name.clone(),
            self.description.clone(),
            self.link.clone(),
            self.hidden,
            false,
            self.backend_kind,
            self.backend_response_type,
            self.handler.clone(),
            self.code_base64.clone(),
        )
        .await?;

        Ok(duplicated_func)
    }

    pub async fn into_frontend_type(&self, ctx: &DalContext) -> FuncResult<FuncSummary> {
        let bindings: Vec<FuncBinding> = FuncBinding::for_func_id(ctx, self.id)
            .await
            .map_err(Box::new)?;

        self.into_frontend_type_sideload_bindings(ctx, bindings)
            .await
    }

    pub async fn into_frontend_type_sideload_bindings(
        &self,
        ctx: &DalContext,
        bindings: Vec<FuncBinding>,
    ) -> FuncResult<FuncSummary> {
        let bindings: Vec<si_frontend_types::FuncBinding> =
            bindings.into_iter().map(Into::into).collect_vec();

        let args = FuncArgument::list_for_func(ctx, self.id)
            .await
            .map_err(Box::new)?;
        let mut arguments = vec![];
        for arg in args {
            arguments.push(si_frontend_types::FuncArgument {
                id: Some(arg.id),
                name: arg.name.clone(),
                kind: arg.kind.into(),
                element_kind: arg.element_kind.map(Into::into),
                timestamp: arg.timestamp.into(),
            });
        }

        let types = self.get_types(ctx).await?;
        Ok(FuncSummary {
            func_id: self.id,
            kind: self.kind.into(),
            name: self.name.clone(),
            backend_kind: self.backend_kind.into(),
            display_name: self.display_name.clone(),
            description: self.description.clone(),
            is_locked: self.is_locked,
            bindings,
            arguments,
            types: Some(types),
        })
    }
    // helper to get updated types to fire WSEvents so SDF can decide when these events need to fire
    pub async fn get_types(&self, ctx: &DalContext) -> FuncResult<String> {
        let types = [
            FuncAuthoringClient::compile_return_types(
                self.backend_response_type,
                self.backend_kind,
            ),
            FuncAuthoringClient::compile_types_from_bindings(ctx, self.id)
                .await
                .map_err(Box::new)?
                .as_str(),
            FuncAuthoringClient::compile_langjs_types(),
        ]
        .join("\n");
        Ok(types)
    }
}

#[derive(Clone, Deserialize, Serialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FuncWsEventPayload {
    func_id: FuncId,
    change_set_id: ChangeSetId,
}
#[derive(Clone, Deserialize, Serialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FuncWsEventFuncSummary {
    change_set_id: ChangeSetId,
    func_summary: si_frontend_types::FuncSummary,
    client_ulid: Option<CoreUlid>,
}
#[derive(Clone, Deserialize, Serialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FuncWsEventCodeSaved {
    change_set_id: ChangeSetId,
    func_code: si_frontend_types::FuncCode,
    generated: bool,
}
#[derive(Clone, Deserialize, Serialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FuncWsEventGeneratingAwsCliCommand {
    command: String,
    subcommand: String,
}
#[derive(Clone, Deserialize, Serialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FuncWsEventGenerating {
    func_id: FuncId,
    command: FuncWsEventGeneratingAwsCliCommand,
}

impl WsEvent {
    pub async fn func_arguments_saved(ctx: &DalContext, func_id: FuncId) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::FuncArgumentsSaved(FuncWsEventPayload {
                func_id,
                change_set_id: ctx.change_set_id(),
            }),
        )
        .await
    }

    pub async fn func_deleted(ctx: &DalContext, func_id: FuncId) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::FuncDeleted(FuncWsEventPayload {
                func_id,
                change_set_id: ctx.change_set_id(),
            }),
        )
        .await
    }

    pub async fn func_saved(ctx: &DalContext, func_id: FuncId) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::FuncSaved(FuncWsEventPayload {
                func_id,
                change_set_id: ctx.change_set_id(),
            }),
        )
        .await
    }

    pub async fn func_updated(
        ctx: &DalContext,
        func_summary: si_frontend_types::FuncSummary,
        client_ulid: Option<CoreUlid>,
    ) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::FuncUpdated(FuncWsEventFuncSummary {
                change_set_id: ctx.change_set_id(),
                func_summary,
                client_ulid,
            }),
        )
        .await
    }

    pub async fn func_created(
        ctx: &DalContext,
        func_summary: si_frontend_types::FuncSummary,
    ) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::FuncCreated(FuncWsEventFuncSummary {
                change_set_id: ctx.change_set_id(),
                func_summary,
                client_ulid: None,
            }),
        )
        .await
    }

    pub async fn func_code_saved(
        ctx: &DalContext,
        func_code: si_frontend_types::FuncCode,
        generated: bool,
    ) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::FuncCodeSaved(FuncWsEventCodeSaved {
                change_set_id: ctx.change_set_id(),
                func_code,
                generated,
            }),
        )
        .await
    }

    pub async fn func_generating(
        ctx: &DalContext,
        func_id: FuncId,
        command: String,
        subcommand: String,
    ) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::FuncGenerating(FuncWsEventGenerating {
                func_id,
                command: FuncWsEventGeneratingAwsCliCommand {
                    command,
                    subcommand,
                },
            }),
        )
        .await
    }
}
