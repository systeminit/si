use base64::{engine::general_purpose, Engine};
use content_store::{ContentHash, Store, StoreError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::string::FromUtf8Error;
use strum::{EnumDiscriminants, IntoEnumIterator};
use telemetry::prelude::*;
use thiserror::Error;
use ulid::Ulid;

use crate::change_set_pointer::ChangeSetPointerError;
use crate::func::intrinsics::IntrinsicFunc;
use crate::workspace_snapshot::edge_weight::{
    EdgeWeight, EdgeWeightError, EdgeWeightKind, EdgeWeightKindDiscriminants,
};
use crate::workspace_snapshot::node_weight::category_node_weight::CategoryNodeKind;
use crate::workspace_snapshot::node_weight::{FuncNodeWeight, NodeWeight, NodeWeightError};
use crate::workspace_snapshot::WorkspaceSnapshotError;
use crate::{pk, DalContext, Timestamp, TransactionsError};

use self::backend::{FuncBackendKind, FuncBackendResponseType};

#[remain::sorted]
#[derive(Error, Debug)]
pub enum FuncError {
    #[error("base64 decode error: {0}")]
    Base64Decode(#[from] base64::DecodeError),
    #[error("change set error: {0}")]
    ChangeSet(#[from] ChangeSetPointerError),
    #[error("edge weight error: {0}")]
    EdgeWeight(#[from] EdgeWeightError),
    #[error("cannot find intrinsic func {0}")]
    IntrinsicFuncNotFound(String),
    #[error("intrinsic spec creation error {0}")]
    IntrinsicSpecCreation(String),
    #[error("node weight error: {0}")]
    NodeWeight(#[from] NodeWeightError),
    #[error("store error: {0}")]
    Store(#[from] StoreError),
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

pub mod argument;
pub mod backend;
// pub before;
// pub mod binding;
// pub mod binding_return_value;
// pub mod execution;
// pub mod identity;
pub mod intrinsics;

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
            handler: value.handler,
            code_base64: value.code_base64,
            code_blake3: value.code_blake3,
        }
    }
}

#[derive(EnumDiscriminants, Serialize, Deserialize, PartialEq)]
// TODO(nick,jacob,zack): decide if this will work with postcard.
// #[serde(tag = "version")]
pub enum FuncContent {
    V1(FuncContentV1),
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct FuncContentV1 {
    pub timestamp: Timestamp,
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub link: Option<String>,
    pub hidden: bool,
    pub builtin: bool,
    pub backend_response_type: FuncBackendResponseType,
    pub handler: Option<String>,
    pub code_base64: Option<String>,
    /// A hash of the code above
    pub code_blake3: ContentHash,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct FuncMetadataView {
    pub display_name: String,
    pub description: Option<String>,
    pub link: Option<String>,
}

pub fn is_intrinsic(name: &str) -> bool {
    intrinsics::IntrinsicFunc::iter().any(|intrinsic| intrinsic.name() == name)
}

pk!(FuncId);

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
    pub timestamp: Timestamp,
    pub name: String,
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
            timestamp: content.timestamp,
            name: node_weight.name().to_owned(),
            display_name: content.display_name,
            description: content.description,
            link: content.link,
            hidden: content.hidden,
            builtin: content.builtin,
            backend_kind: node_weight.backend_kind(),
            backend_response_type: content.backend_response_type,
            handler: content.handler,
            code_base64: content.code_base64,
            code_blake3: content.code_blake3,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn new(
        ctx: &DalContext,
        name: impl Into<String>,
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

        let code_base64 = code_base64.map(Into::into);

        let code_blake3 = ContentHash::new(code_base64.as_deref().unwrap_or("").as_bytes());

        let content = FuncContentV1 {
            timestamp,
            display_name: display_name.map(Into::into),
            description: description.map(Into::into),
            link: link.map(Into::into),
            hidden,
            builtin,
            backend_response_type,
            handler: handler.map(Into::into),
            code_base64,
            code_blake3,
        };

        let hash = ctx
            .content_store()
            .try_lock()?
            .add(&FuncContent::V1(content.clone()))?;

        let change_set = ctx.change_set_pointer()?;
        let id = change_set.generate_ulid()?;
        let node_weight = NodeWeight::new_func(change_set, id, name.into(), backend_kind, hash)?;
        let mut workspace_snapshot = ctx.workspace_snapshot()?.try_lock()?;
        let _node_index = workspace_snapshot.add_node(node_weight.clone())?;

        let func_category_id =
            workspace_snapshot.get_category_node(None, CategoryNodeKind::Func)?;
        workspace_snapshot.add_edge(
            func_category_id,
            EdgeWeight::new(change_set, EdgeWeightKind::Use)?,
            id,
        )?;

        let func_node_weight = node_weight.get_func_node_weight()?;

        Ok(Self::assemble(&func_node_weight, &content))
    }

    pub async fn get_by_id(ctx: &DalContext, id: FuncId) -> FuncResult<Self> {
        let (node_weight, content) = Self::get_node_weight_and_content(ctx, id).await?;
        Ok(Self::assemble(&node_weight, &content))
    }

    pub fn find_by_name(ctx: &DalContext, name: impl AsRef<str>) -> FuncResult<Option<FuncId>> {
        let mut workspace_snapshot = ctx.workspace_snapshot()?.try_lock()?;
        let func_category_id =
            workspace_snapshot.get_category_node(None, CategoryNodeKind::Func)?;
        let func_indices = workspace_snapshot.outgoing_targets_for_edge_weight_kind(
            func_category_id,
            EdgeWeightKindDiscriminants::Use,
        )?;
        let name = name.as_ref();
        for func_index in func_indices {
            let node_weight = workspace_snapshot.get_node_weight(func_index)?;
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

    pub async fn modify_by_id<L>(ctx: &DalContext, id: FuncId, lambda: L) -> FuncResult<Func>
    where
        L: FnOnce(&mut Func) -> FuncResult<()>,
    {
        let func = Func::get_by_id(ctx, id).await?;
        let modified_func = func.modify(ctx, lambda).await?;
        Ok(modified_func)
    }

    pub async fn get_node_weight_and_content(
        ctx: &DalContext,
        func_id: FuncId,
    ) -> FuncResult<(FuncNodeWeight, FuncContentV1)> {
        let (func_node_weight, hash) = Self::get_node_weight_and_content_hash(ctx, func_id).await?;

        let content: FuncContent = ctx.content_store().try_lock()?.get(&hash).await?.ok_or(
            WorkspaceSnapshotError::MissingContentFromStore(func_id.into()),
        )?;

        // NOTE(nick,jacob,zack): if we had a v2, then there would be migration logic here.
        let FuncContent::V1(inner) = content;

        Ok((func_node_weight, inner))
    }

    async fn get_node_weight_and_content_hash(
        ctx: &DalContext,
        func_id: FuncId,
    ) -> FuncResult<(FuncNodeWeight, ContentHash)> {
        let mut workspace_snapshot = ctx.workspace_snapshot()?.try_lock()?;
        let id: Ulid = func_id.into();
        let node_index = workspace_snapshot.get_node_index_by_id(id)?;
        let node_weight = workspace_snapshot.get_node_weight(node_index)?;

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

        let (mut node_weight, _) = Func::get_node_weight_and_content_hash(ctx, func.id).await?;
        let mut workspace_snapshot = ctx.workspace_snapshot()?.try_lock()?;

        // If both either the name or backend_kind have changed, *and* parts of the FuncContent
        // have changed, this ends up updating the node for the function twice. This could be
        // optimized to do it only once.
        if func.name.as_str() != node_weight.name()
            || func.backend_kind != node_weight.backend_kind()
        {
            let original_node_index = workspace_snapshot.get_node_index_by_id(func.id)?;

            node_weight
                .set_name(func.name.as_str())
                .set_backend_kind(func.backend_kind);

            workspace_snapshot.add_node(NodeWeight::Func(node_weight.clone()))?;

            workspace_snapshot.replace_references(original_node_index)?;
        }
        let updated = FuncContentV1::from(func.clone());

        if updated != before {
            let hash = ctx
                .content_store()
                .try_lock()?
                .add(&FuncContent::V1(updated.clone()))?;
            workspace_snapshot.update_content(ctx.change_set_pointer()?, func.id.into(), hash)?;
        }

        Ok(Func::assemble(&node_weight, &updated))
    }

    pub fn remove(ctx: &DalContext, id: FuncId) -> FuncResult<()> {
        // to remove a func we must remove all incoming edges to it. It will then be
        // garbage collected out of the graph

        let mut workspace_snapshot = ctx.workspace_snapshot()?.try_lock()?;

        let arg_node_idx = workspace_snapshot.get_node_index_by_id(id)?;

        let users = workspace_snapshot
            .incoming_sources_for_edge_weight_kind(id, EdgeWeightKind::Use.into())?;

        let change_set = ctx.change_set_pointer()?;
        for user in users {
            workspace_snapshot.remove_edge(
                change_set,
                user,
                arg_node_idx,
                EdgeWeightKind::Use.into(),
            )?;
        }

        // Removes the actual node from the graph
        workspace_snapshot.remove_node_by_id(id)?;

        Ok(())
    }

    pub fn find_intrinsic(ctx: &DalContext, intrinsic: IntrinsicFunc) -> FuncResult<FuncId> {
        let name = intrinsic.name();
        Self::find_by_name(ctx, name)?.ok_or(FuncError::IntrinsicFuncNotFound(name.to_owned()))
    }

    pub async fn list(ctx: &DalContext) -> FuncResult<Vec<Self>> {
        let mut workspace_snapshot = ctx.workspace_snapshot()?.try_lock()?;

        let mut funcs = vec![];
        let func_category_id =
            workspace_snapshot.get_category_node(None, CategoryNodeKind::Func)?;

        let func_node_indexes = workspace_snapshot.outgoing_targets_for_edge_weight_kind(
            func_category_id,
            EdgeWeightKindDiscriminants::Use,
        )?;

        let mut func_node_weights = vec![];
        let mut func_content_hash = vec![];
        for index in func_node_indexes {
            let node_weight = workspace_snapshot
                .get_node_weight(index)?
                .get_func_node_weight()?;
            func_content_hash.push(node_weight.content_hash());
            func_node_weights.push(node_weight);
        }

        let func_contents: HashMap<ContentHash, FuncContent> = ctx
            .content_store()
            .try_lock()?
            .get_bulk(func_content_hash.as_slice())
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
}

// impl Func {
//     #[instrument(skip_all)]
//     pub async fn new(
//         ctx: &DalContext,
//         name: impl AsRef<str>,
//         backend_kind: FuncBackendKind,
//         backend_response_type: FuncBackendResponseType,
//     ) -> FuncResult<Self> {
//         let name = name.as_ref();
//         let row = ctx
//             .txns()
//             .await?
//             .pg()
//             .query_one(
//                 "SELECT object FROM func_create_v1($1, $2, $3, $4, $5)",
//                 &[
//                     ctx.tenancy(),
//                     ctx.visibility(),
//                     &name,
//                     &backend_kind.as_ref(),
//                     &backend_response_type.as_ref(),
//                 ],
//             )
//             .await?;
//         let object = standard_model::finish_create_from_row(ctx, row).await?;
//         Ok(object)
//     }

//     /// Creates a new [`Func`] from [`self`](Func). All relevant fields are duplicated, but rows
//     /// existing on relationship tables (e.g. "belongs_to" or "many_to_many") are not.
//     pub async fn duplicate(&self, ctx: &DalContext) -> FuncResult<Self> {
//         // Generate a unique name and make sure it's not in use
//         let mut new_unique_name;
//         loop {
//             new_unique_name = format!("{}{}", self.name(), generate_unique_id(4));
//             if Self::find_by_name(ctx, &new_unique_name).await?.is_none() {
//                 break;
//             };
//         }

//         let mut new_func = Self::new(
//             ctx,
//             new_unique_name,
//             *self.backend_kind(),
//             *self.backend_response_type(),
//         )
//         .await?;

//         // Duplicate all fields on the func that do not come in through the constructor.
//         new_func.set_display_name(ctx, self.display_name()).await?;
//         new_func.set_description(ctx, self.description()).await?;
//         new_func.set_link(ctx, self.link()).await?;
//         new_func.set_hidden(ctx, self.hidden).await?;
//         new_func.set_builtin(ctx, self.builtin).await?;
//         new_func.set_handler(ctx, self.handler()).await?;
//         new_func.set_code_base64(ctx, self.code_base64()).await?;

//         Ok(new_func)
//     }

//     #[allow(clippy::result_large_err)]
//     pub fn code_plaintext(&self) -> FuncResult<Option<String>> {
//         Ok(match self.code_base64() {
//             Some(base64_code) => Some(String::from_utf8(
//                 general_purpose::STANDARD_NO_PAD.decode(base64_code)?,
//             )?),
//             None => None,
//         })
//     }

//     pub async fn is_builtin(&self, ctx: &DalContext) -> FuncResult<bool> {
//         let row = ctx
//             .txns()
//             .await?
//             .pg()
//             .query_opt(
//                 "SELECT id FROM funcs WHERE id = $1 and tenancy_workspace_pk = $2 LIMIT 1",
//                 &[self.id(), &WorkspacePk::NONE],
//             )
//             .await?;

//         Ok(row.is_some())
//     }

//     pub async fn set_code_plaintext(
//         &mut self,
//         ctx: &DalContext,
//         code: Option<&'_ str>,
//     ) -> FuncResult<()> {
//         self.set_code_base64(
//             ctx,
//             code.as_ref()
//                 .map(|code| general_purpose::STANDARD_NO_PAD.encode(code)),
//         )
//         .await
//     }

//     pub fn metadata_view(&self) -> FuncMetadataView {
//         FuncMetadataView {
//             display_name: self.display_name().unwrap_or_else(|| self.name()).into(),
//             description: self.description().map(Into::into),
//             link: self.description().map(Into::into),
//         }
//     }

//     pub async fn for_binding(ctx: &DalContext, func_binding: &FuncBinding) -> FuncResult<Self> {
//         let row = ctx
//             .txns()
//             .await?
//             .pg()
//             .query_one(
//                 "SELECT row_to_json(funcs.*) AS object
//                 FROM funcs_v1($1, $2) AS funcs
//                 INNER JOIN func_binding_belongs_to_func_v1($1, $2) AS func_binding_belongs_to_func
//                     ON funcs.id = func_binding_belongs_to_func.belongs_to_id
//                 WHERE func_binding_belongs_to_func.object_id = $3",
//                 &[ctx.tenancy(), ctx.visibility(), func_binding.id()],
//             )
//             .await?;
//         let object = standard_model::finish_create_from_row(ctx, row).await?;
//         Ok(object)
//     }

//     pub async fn find_by_name(ctx: &DalContext, name: &str) -> FuncResult<Option<Self>> {
//         Ok(Self::find_by_attr(ctx, "name", &name).await?.pop())
//     }

//     /// Returns `true` if this function is one handled internally by the `dal`, `false` if the
//     /// function is one that will be executed by `veritech`
//     pub fn is_intrinsic(&self) -> bool {
//         is_intrinsic(self.name())
//     }

//     standard_model_accessor!(name, String, FuncResult);
//     standard_model_accessor!(display_name, Option<String>, FuncResult);
//     standard_model_accessor!(description, Option<String>, FuncResult);
//     standard_model_accessor!(link, Option<String>, FuncResult);
//     standard_model_accessor!(hidden, bool, FuncResult);
//     standard_model_accessor!(builtin, bool, FuncResult);
//     standard_model_accessor!(backend_kind, Enum(FuncBackendKind), FuncResult);
//     standard_model_accessor!(
//         backend_response_type,
//         Enum(FuncBackendResponseType),
//         FuncResult
//     );
//     standard_model_accessor!(handler, Option<String>, FuncResult);
//     standard_model_accessor!(code_base64, Option<String>, FuncResult);
//     standard_model_accessor_ro!(code_sha256, String);
// }
