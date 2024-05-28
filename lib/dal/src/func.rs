use base64::{engine::general_purpose, Engine};
use serde::{Deserialize, Serialize};
use si_events::CasValue;
use si_events::{ulid::Ulid, ContentHash};
use std::collections::HashMap;
use std::string::FromUtf8Error;
use std::sync::Arc;
use strum::IntoEnumIterator;
use telemetry::prelude::*;
use thiserror::Error;

use crate::change_set::ChangeSetError;
use crate::func::argument::FuncArgumentId;
use crate::func::associations::FuncAssociationsError;
use crate::func::intrinsics::IntrinsicFunc;
use crate::layer_db_types::{FuncContent, FuncContentV1};
use crate::workspace_snapshot::edge_weight::{
    EdgeWeightError, EdgeWeightKind, EdgeWeightKindDiscriminants,
};
use crate::workspace_snapshot::graph::WorkspaceSnapshotGraphError;
use crate::workspace_snapshot::node_weight::category_node_weight::CategoryNodeKind;
use crate::workspace_snapshot::node_weight::{FuncNodeWeight, NodeWeight, NodeWeightError};
use crate::workspace_snapshot::WorkspaceSnapshotError;
use crate::{
    id, implement_add_edge_to, pk, ChangeSetId, DalContext, HelperError, Timestamp,
    TransactionsError, WsEvent, WsEventResult, WsPayload,
};

use self::backend::{FuncBackendKind, FuncBackendResponseType};

pub mod argument;
pub mod authoring;
pub mod backend;
pub mod intrinsics;
pub mod runner;
pub mod summary;
pub mod view;

mod associations;
mod kind;

pub use associations::AttributePrototypeArgumentBag;
pub use associations::AttributePrototypeBag;
pub use associations::FuncAssociations;
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
    #[error("edge weight error: {0}")]
    EdgeWeight(#[from] EdgeWeightError),
    #[error("func associations error: {0}")]
    FuncAssociations(#[from] Box<FuncAssociationsError>),
    #[error("func name already in use {0}")]
    FuncNameInUse(String),
    #[error("func to be deleted has associations: {0}")]
    FuncToBeDeletedHasAssociations(FuncId),
    #[error("helper error: {0}")]
    Helper(#[from] HelperError),
    #[error("cannot find intrinsic func {0}")]
    IntrinsicFuncNotFound(String),
    #[error("intrinsic spec creation error {0}")]
    IntrinsicSpecCreation(String),
    #[error("layer db error: {0}")]
    LayerDb(#[from] si_layer_cache::LayerDbError),
    #[error("node weight error: {0}")]
    NodeWeight(#[from] NodeWeightError),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error("could not acquire lock: {0}")]
    TryLock(#[from] tokio::sync::TryLockError),
    #[error("unable to determine the function type for backend kind ({0}) and backend response type ({1})")]
    UnknownFunctionType(FuncBackendKind, FuncBackendResponseType),
    #[error("utf8 error: {0}")]
    Utf8(#[from] FromUtf8Error),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] WorkspaceSnapshotError),
}

pub type FuncResult<T> = Result<T, FuncError>;

impl From<Func> for FuncContentV1 {
    fn from(value: Func) -> Self {
        Self {
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
        }
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

id!(FuncId);

// NOTE: This is here only for backward compatibility
pk!(FuncExecutionPk);

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
}

impl Func {
    pub fn assemble(node_weight: &FuncNodeWeight, content: &FuncContentV1) -> Self {
        let content = content.to_owned();
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
            let (hash, _) = ctx
                .layer_db()
                .cas()
                .write(
                    Arc::new(code_cas_value.into()),
                    None,
                    ctx.events_tenancy(),
                    ctx.events_actor(),
                )
                .await?;
            hash
        } else {
            // Why are we doing this? Because the struct gods demand it. I have feelings.
            ContentHash::new("".as_bytes())
        };

        let content = FuncContentV1 {
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
        };

        let (hash, _) = ctx
            .layer_db()
            .cas()
            .write(
                Arc::new(FuncContent::V1(content.clone()).into()),
                None,
                ctx.events_tenancy(),
                ctx.events_actor(),
            )
            .await?;

        let func_kind = FuncKind::new(backend_kind, backend_response_type)?;

        let change_set = ctx.change_set()?;
        let id = change_set.generate_ulid()?;
        let node_weight =
            NodeWeight::new_func(change_set, id, name.clone().into(), func_kind, hash)?;

        let workspace_snapshot = ctx.workspace_snapshot()?;
        workspace_snapshot.add_node(node_weight.clone()).await?;

        let func_category_id = workspace_snapshot
            .get_category_node_or_err(None, CategoryNodeKind::Func)
            .await?;
        Self::add_category_edge(ctx, func_category_id, id.into(), EdgeWeightKind::new_use())
            .await?;

        let func_node_weight = node_weight.get_func_node_weight()?;

        Ok(Self::assemble(&func_node_weight, &content))
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

    pub async fn get_by_id(ctx: &DalContext, id: FuncId) -> FuncResult<Option<Self>> {
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

    pub async fn get_by_id_or_error(ctx: &DalContext, id: FuncId) -> FuncResult<Self> {
        let (func_node_weight, hash) =
            Self::get_node_weight_and_content_hash_or_error(ctx, id).await?;
        Self::get_by_id_inner(ctx, &hash, &func_node_weight).await
    }

    async fn get_by_id_inner(
        ctx: &DalContext,
        hash: &ContentHash,
        func_node_weight: &FuncNodeWeight,
    ) -> FuncResult<Self> {
        let content: FuncContent = ctx.layer_db().cas().try_read_as(hash).await?.ok_or(
            WorkspaceSnapshotError::MissingContentFromStore(func_node_weight.id()),
        )?;

        // NOTE(nick,jacob,zack): if we had a v2, then there would be migration logic here.
        let FuncContent::V1(inner) = content;

        Ok(Self::assemble(func_node_weight, &inner))
    }

    pub async fn find_by_name(
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
                | IntrinsicFunc::SetJson
                | IntrinsicFunc::SetMap
                | IntrinsicFunc::SetObject
                | IntrinsicFunc::SetString
                | IntrinsicFunc::Unset => false,
                IntrinsicFunc::Identity | IntrinsicFunc::Validation => true,
            },
            None => true,
        }
    }

    pub async fn modify_by_id<L>(ctx: &DalContext, id: FuncId, lambda: L) -> FuncResult<Func>
    where
        L: FnOnce(&mut Func) -> FuncResult<()>,
    {
        let func = Func::get_by_id_or_error(ctx, id).await?;
        let modified_func = func.modify(ctx, lambda).await?;
        Ok(modified_func)
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

    pub async fn modify<L>(self, ctx: &DalContext, lambda: L) -> FuncResult<Self>
    where
        L: FnOnce(&mut Self) -> FuncResult<()>,
    {
        let mut func = self;

        let before = FuncContentV1::from(func.clone());
        lambda(&mut func)?;

        let (mut node_weight, _) =
            Self::get_node_weight_and_content_hash_or_error(ctx, func.id).await?;

        let workspace_snapshot = ctx.workspace_snapshot()?;

        // If the name HAS changed, *and* parts of the FuncContent
        // have changed, this ends up updating the node for the function twice. This could be
        // optimized to do it only once.
        if func.name.as_str() != node_weight.name() {
            let original_node_index = workspace_snapshot.get_node_index_by_id(func.id).await?;

            node_weight.set_name(func.name.as_str());

            workspace_snapshot
                .add_node(NodeWeight::Func(
                    node_weight.new_with_incremented_vector_clock(ctx.change_set()?)?,
                ))
                .await?;

            workspace_snapshot
                .replace_references(original_node_index)
                .await?;
        }
        let updated = FuncContentV1::from(func.clone());

        if updated != before {
            let (hash, _) = ctx
                .layer_db()
                .cas()
                .write(
                    Arc::new(FuncContent::V1(updated.clone()).into()),
                    None,
                    ctx.events_tenancy(),
                    ctx.events_actor(),
                )
                .await?;
            workspace_snapshot
                .update_content(ctx.change_set()?, func.id.into(), hash)
                .await?;
        }

        Ok(Self::assemble(&node_weight, &updated))
    }

    /// Deletes the [`Func`] and returns the name.
    pub async fn delete_by_id(ctx: &DalContext, id: FuncId) -> FuncResult<String> {
        let func = Self::get_by_id_or_error(ctx, id).await?;

        // Check that we can remove the func.
        let (maybe_associations, _) = FuncAssociations::from_func(ctx, &func)
            .await
            .map_err(Box::new)?;
        if let Some(associations) = maybe_associations {
            let has_associations = match associations {
                FuncAssociations::Action {
                    schema_variant_ids,
                    kind: _,
                } => !schema_variant_ids.is_empty(),
                FuncAssociations::Attribute { prototypes } => !prototypes.is_empty(),
                FuncAssociations::CodeGeneration {
                    schema_variant_ids,
                    component_ids,
                    inputs: _,
                } => !schema_variant_ids.is_empty() || !component_ids.is_empty(),
                FuncAssociations::Qualification {
                    schema_variant_ids,
                    component_ids,
                    inputs: _,
                } => !schema_variant_ids.is_empty() || !component_ids.is_empty(),
                FuncAssociations::Authentication { schema_variant_ids } => {
                    !schema_variant_ids.is_empty()
                }
            };

            if has_associations {
                return Err(FuncError::FuncToBeDeletedHasAssociations(id));
            }
        };

        // Now, we can remove the func.
        let workspace_snapshot = ctx.workspace_snapshot()?;
        let change_set = ctx.change_set()?;
        workspace_snapshot.remove_node_by_id(change_set, id).await?;

        Ok(func.name)
    }

    pub async fn find_intrinsic(ctx: &DalContext, intrinsic: IntrinsicFunc) -> FuncResult<FuncId> {
        let name = intrinsic.name();
        Self::find_by_name(ctx, name)
            .await?
            .ok_or(FuncError::IntrinsicFuncNotFound(name.to_owned()))
    }

    pub async fn list(ctx: &DalContext) -> FuncResult<Vec<Self>> {
        let workspace_snapshot = ctx.workspace_snapshot()?;

        let mut funcs = vec![];
        let func_category_id = workspace_snapshot
            .get_category_node_or_err(None, CategoryNodeKind::Func)
            .await?;

        let func_node_indexes = workspace_snapshot
            .outgoing_targets_for_edge_weight_kind(
                func_category_id,
                EdgeWeightKindDiscriminants::Use,
            )
            .await?;

        let mut func_node_weights = vec![];
        let mut func_content_hash = vec![];
        for index in func_node_indexes {
            let node_weight = workspace_snapshot
                .get_node_weight(index)
                .await?
                .get_func_node_weight()?;
            func_content_hash.push(node_weight.content_hash());
            func_node_weights.push(node_weight);
        }

        let func_contents: HashMap<ContentHash, FuncContent> = ctx
            .layer_db()
            .cas()
            .try_read_many_as(func_content_hash.as_slice())
            .await?;

        for node_weight in func_node_weights {
            match func_contents.get(&node_weight.content_hash()) {
                Some(func_content) => {
                    // NOTE(nick,jacob,zack): if we had a v2, then there would be migration logic here.
                    let FuncContent::V1(inner) = func_content;

                    funcs.push(Func::assemble(&node_weight, inner));
                }
                None => Err(WorkspaceSnapshotError::MissingContentFromStore(
                    node_weight.id(),
                ))?,
            }
        }

        Ok(funcs)
    }

    pub async fn duplicate(&self, ctx: &DalContext, new_name: String) -> FuncResult<Self> {
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
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FuncWsEventPayload {
    func_id: FuncId,
    change_set_id: ChangeSetId,
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
}
