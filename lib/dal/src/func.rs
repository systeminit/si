use std::{
    collections::HashMap,
    str::FromStr,
    string::FromUtf8Error,
    sync::Arc,
};

use argument::{
    FuncArgument,
    FuncArgumentError,
};
use authoring::{
    FuncAuthoringClient,
    FuncAuthoringError,
};
use base64::{
    Engine,
    engine::general_purpose,
};
use binding::{
    FuncBinding,
    FuncBindingError,
};
use chrono::{
    DateTime,
    Utc,
};
use itertools::Itertools;
use serde::{
    Deserialize,
    Serialize,
};
use si_events::{
    CasValue,
    ContentHash,
    Timestamp,
    ulid::Ulid,
};
use si_frontend_types::FuncSummary;
use si_pkg::SpecError;
use strum::IntoEnumIterator;
use telemetry::prelude::*;
use thiserror::Error;
use ulid::Ulid as CoreUlid;

use self::backend::{
    FuncBackendKind,
    FuncBackendResponseType,
};
use crate::{
    ChangeSetId,
    DalContext,
    HelperError,
    TransactionsError,
    WsEvent,
    WsEventResult,
    WsPayload,
    change_set::ChangeSetError,
    func::{
        argument::FuncArgumentId,
        intrinsics::IntrinsicFunc,
    },
    implement_add_edge_to,
    layer_db_types::FuncContent,
    pkg,
    workspace_snapshot::{
        WorkspaceSnapshotError,
        edge_weight::{
            EdgeWeightKind,
            EdgeWeightKindDiscriminants,
        },
        node_weight::{
            FuncNodeWeight,
            NodeWeight,
            NodeWeightError,
            category_node_weight::CategoryNodeKind,
        },
        traits::func::FuncExt as _,
    },
};

pub mod argument;
pub mod authoring;
pub mod backend;
pub mod binding;
pub mod debug;
pub mod intrinsics;
mod kind;
pub mod leaf;
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
        Self::V3(FuncContentV3 {
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
            is_transformation: value.is_transformation,
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

// NOTE: This is here only for backward compatibility
pub use si_id::{
    FuncExecutionPk,
    FuncId,
};

use crate::layer_db_types::FuncContentV3;

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
    pub is_transformation: bool,
}

impl Func {
    pub fn assemble(node_weight: &FuncNodeWeight, content: FuncContentV3) -> Self {
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
            is_transformation: content.is_transformation,
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
    pub async fn upsert_with_id(
        ctx: &DalContext,
        id: FuncId,
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
        is_transformation: bool,
        updated_at: Option<DateTime<Utc>>,
    ) -> FuncResult<Self> {
        let timestamp = {
            let mut timestamp = Timestamp::now();

            if let Some(updated) = updated_at {
                timestamp.updated_at = updated;
                timestamp.created_at = updated;
            }
            timestamp
        };

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

        let content = FuncContentV3 {
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
            is_transformation,
        };

        let (hash, _) = ctx.layer_db().cas().write(
            Arc::new(FuncContent::V3(content.clone()).into()),
            None,
            ctx.events_tenancy(),
            ctx.events_actor(),
        )?;

        let func_kind = FuncKind::new(backend_kind, backend_response_type)?;

        let lineage_id = id.into();
        let node_weight =
            NodeWeight::new_func(id.into(), lineage_id, name.clone().into(), func_kind, hash);

        let workspace_snapshot = ctx.workspace_snapshot()?;
        workspace_snapshot
            .add_or_replace_node(node_weight.clone())
            .await?;

        let func_category_id = workspace_snapshot
            .get_category_node_or_err(CategoryNodeKind::Func)
            .await?;
        Self::add_category_edge(ctx, func_category_id, id, EdgeWeightKind::new_use()).await?;

        let func_node_weight = node_weight.get_func_node_weight()?;

        Ok(Self::assemble(&func_node_weight, content))
    }

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
        is_transformation: bool,
    ) -> FuncResult<Self> {
        let id = ctx.workspace_snapshot()?.generate_ulid().await?.into();

        Self::upsert_with_id(
            ctx,
            id,
            name,
            display_name,
            description,
            link,
            hidden,
            builtin,
            backend_kind,
            backend_response_type,
            handler,
            code_base64,
            is_transformation,
            None,
        )
        .await
    }

    /// Create a debug function with the given code. Debug functions are
    /// emphemeral, are not stored on the graph, but will be recorded in the
    /// func run logs when they are executed.
    pub fn new_debug(
        name: impl Into<String>,
        code: impl Into<String>,
        handler: impl Into<String>,
    ) -> Self {
        let id: FuncId = Ulid::new().into();
        let name = name.into();
        let code = code.into();
        let handler = Some(handler.into());

        let base64_code = general_purpose::STANDARD_NO_PAD.encode(&code);
        let code_blake3 = ContentHash::new(code.as_bytes());

        Self {
            id,
            name,
            kind: FuncKind::Debug,
            timestamp: Timestamp::now(),
            display_name: None,
            description: None,
            link: None,
            hidden: false,
            builtin: false,
            backend_kind: FuncBackendKind::Debug,
            backend_response_type: FuncBackendResponseType::Debug,
            handler,
            code_base64: Some(base64_code),
            code_blake3,
            is_locked: false,
            is_transformation: false,
        }
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
        let workspace_snapshot = ctx.workspace_snapshot()?;
        let Some(node_weight) = workspace_snapshot.get_node_weight_opt(id).await else {
            return Ok(None);
        };
        let func_node_weight = node_weight.get_func_node_weight()?;
        let hash = func_node_weight.content_hash();

        let func = Self::get_by_id_inner(ctx, &hash, &func_node_weight).await?;
        Ok(Some(func))
    }

    pub async fn get_by_id(ctx: &DalContext, id: FuncId) -> FuncResult<Self> {
        let func_node_weight = Self::node_weight(ctx, id).await?;
        Self::get_by_id_inner(ctx, &func_node_weight.content_hash(), &func_node_weight).await
    }

    /// If you know the func_id is supposed to be for an [`IntrinsicFunc`], get which one or error
    pub async fn intrinsic_kind_or_error(
        ctx: &DalContext,
        id: FuncId,
    ) -> FuncResult<IntrinsicFunc> {
        let func = Self::get_by_id(ctx, id).await?;

        Self::intrinsic_kind(ctx, id)
            .await?
            .ok_or(FuncError::IntrinsicFuncNotFound(func.name))
    }

    pub async fn intrinsic_kind(ctx: &DalContext, id: FuncId) -> FuncResult<Option<IntrinsicFunc>> {
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
        let inner = content.extract();

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
            .get_category_node_or_err(CategoryNodeKind::Func)
            .await?;
        let func_indices = workspace_snapshot
            .outgoing_targets_for_edge_weight_kind(
                func_category_id,
                EdgeWeightKindDiscriminants::Use,
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
            .get_category_node_or_err(CategoryNodeKind::Func)
            .await?;
        let func_indices = workspace_snapshot
            .outgoing_targets_for_edge_weight_kind(
                func_category_id,
                EdgeWeightKindDiscriminants::Use,
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

    /// Attempt to find a [`Func`] by ID first, then falling back to name lookup.
    pub async fn find_by_id_or_name(
        ctx: &DalContext,
        unique_id: impl AsRef<str>,
        name: impl AsRef<str>,
    ) -> FuncResult<Option<Func>> {
        let unique_id_str = unique_id.as_ref();

        if let Ok(func_id) = FuncId::from_str(unique_id_str) {
            if let Some(func) = Self::get_by_id_opt(ctx, func_id).await? {
                return Ok(Some(func));
            }
        }

        if let Some(func_id) = Self::find_id_by_name(ctx, name).await? {
            Ok(Some(Self::get_by_id(ctx, func_id).await?))
        } else {
            Ok(None)
        }
    }

    pub fn code_plaintext(&self) -> FuncResult<Option<String>> {
        Ok(match &self.code_base64 {
            Some(base64_code) => Some(String::from_utf8(
                general_purpose::STANDARD_NO_PAD.decode(base64_code)?,
            )?),
            None => None,
        })
    }

    pub async fn is_dynamic(ctx: &DalContext, func_id: FuncId) -> FuncResult<bool> {
        ctx.workspace_snapshot()?.func_is_dynamic(func_id).await
    }

    pub fn is_intrinsic(&self) -> bool {
        IntrinsicFunc::maybe_from_str(&self.name).is_some()
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

    pub async fn node_weight(ctx: &DalContext, func_id: FuncId) -> FuncResult<FuncNodeWeight> {
        let workspace_snapshot = ctx.workspace_snapshot()?;
        Ok(workspace_snapshot
            .get_node_weight(func_id)
            .await?
            .get_func_node_weight()?)
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

        // Variant-level bindings take precedence - if they exist, apply lock check
        // Only skip lock check if there are NO variant-level bindings (only schema overlays)
        let has_variant_bindings = FuncBinding::has_variant_bindings(ctx, func.id)
            .await
            .map_err(Box::new)?;
        if has_variant_bindings {
            func.error_if_locked()?;
        }

        let before = FuncContent::from(func.clone());
        lambda(&mut func)?;

        let mut node_weight = Self::node_weight(ctx, func.id).await?;

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
            .get_category_node_or_err(CategoryNodeKind::Func)
            .await?;

        let func_node_indexes = workspace_snapshot
            .outgoing_targets_for_edge_weight_kind(
                func_category_id,
                EdgeWeightKindDiscriminants::Use,
            )
            .await?;

        let mut func_node_weights = Vec::with_capacity(func_node_indexes.len());
        let mut func_content_hashes = Vec::with_capacity(func_node_indexes.len());
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

        let mut func_node_weights = Vec::with_capacity(func_ids.len());
        let mut func_content_hashes = Vec::with_capacity(func_ids.len());
        for id in func_ids {
            let node_weight = workspace_snapshot
                .get_node_weight(id)
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

        let mut funcs = Vec::with_capacity(func_node_weights.len());
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
            self.is_transformation,
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
            self.is_transformation,
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
                timestamp: arg.timestamp,
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
            is_transformation: self.is_transformation,
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

    /// Get a short, human-readable title suitable for debugging/display.
    pub async fn fmt_title(ctx: &DalContext, id: FuncId) -> String {
        Self::fmt_title_fallible(ctx, id)
            .await
            .unwrap_or_else(|e| e.to_string())
    }
    async fn fmt_title_fallible(ctx: &DalContext, id: FuncId) -> FuncResult<String> {
        Ok(Self::node_weight(ctx, id).await?.name)
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
