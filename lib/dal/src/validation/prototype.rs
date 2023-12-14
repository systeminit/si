use content_store::Store;
use serde::{Deserialize, Serialize};
use strum::EnumDiscriminants;
use telemetry::prelude::*;
use thiserror::Error;

use crate::change_set_pointer::ChangeSetPointerError;
use crate::func::backend::validation::FuncBackendValidationArgs;
use crate::func::intrinsics::IntrinsicFunc;
use crate::func::FuncError;
use crate::validation::Validation;
use crate::workspace_snapshot::content_address::ContentAddress;
use crate::workspace_snapshot::edge_weight::{EdgeWeight, EdgeWeightError, EdgeWeightKind};
use crate::workspace_snapshot::node_weight::{NodeWeight, NodeWeightError};
use crate::workspace_snapshot::WorkspaceSnapshotError;
use crate::{func::FuncId, pk, DalContext, Func, PropId, Timestamp, TransactionsError};

// pub mod context;

#[remain::sorted]
#[derive(Error, Debug)]
pub enum ValidationPrototypeError {
    #[error("change set error: {0}")]
    ChangeSet(#[from] ChangeSetPointerError),
    #[error("edge weight error: {0}")]
    EdgeWeight(#[from] EdgeWeightError),
    #[error("func error: {0}")]
    Func(#[from] FuncError),
    #[error("node weight error: {0}")]
    NodeWeight(#[from] NodeWeightError),
    #[error("serde json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("store error: {0}")]
    Store(#[from] content_store::StoreError),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error("could not acquire lock: {0}")]
    TryLock(#[from] tokio::sync::TryLockError),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] WorkspaceSnapshotError),
}

pub type ValidationPrototypeResult<T> = Result<T, ValidationPrototypeError>;

pk!(ValidationPrototypeId);

// An ValidationPrototype joins a `Func` to the context in which
// the component that is created with it can use to generate a ValidationResolver.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct ValidationPrototype {
    id: ValidationPrototypeId,
    #[serde(flatten)]
    timestamp: Timestamp,
    func_id: FuncId,
    args: serde_json::Value,
    link: Option<String>,
}

#[derive(EnumDiscriminants, Serialize, Deserialize, PartialEq)]
pub enum ValidationPrototypeContent {
    V1(ValidationPrototypeContentV1),
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ValidationPrototypeContentV1 {
    pub timestamp: Timestamp,
    pub func_id: FuncId,
    pub args: String,
    pub link: Option<String>,
}

impl ValidationPrototype {
    pub fn assemble(
        id: ValidationPrototypeId,
        inner: ValidationPrototypeContentV1,
    ) -> ValidationPrototypeResult<Self> {
        Ok(Self {
            id,
            timestamp: inner.timestamp,
            func_id: inner.func_id,
            args: serde_json::to_value(&inner.args)?,
            link: inner.link,
        })
    }

    pub fn new(
        ctx: &DalContext,
        func_id: FuncId,
        args: serde_json::Value,
        prop_id: PropId,
    ) -> ValidationPrototypeResult<Self> {
        let content = ValidationPrototypeContentV1 {
            timestamp: Timestamp::now(),
            func_id,
            args: serde_json::to_string(&args)?,
            link: None,
        };
        let hash = ctx
            .content_store()
            .try_lock()?
            .add(&ValidationPrototypeContent::V1(content.clone()))?;

        let change_set = ctx.change_set_pointer()?;
        let id = change_set.generate_ulid()?;
        let node_weight =
            NodeWeight::new_content(change_set, id, ContentAddress::ValidationPrototype(hash))?;
        let mut workspace_snapshot = ctx.workspace_snapshot()?.try_lock()?;
        let _node_index = workspace_snapshot.add_node(node_weight)?;

        workspace_snapshot.add_edge(
            prop_id,
            EdgeWeight::new(change_set, EdgeWeightKind::Use)?,
            id,
        )?;

        Self::assemble(id.into(), content)
    }

    pub fn new_intrinsic(
        ctx: &DalContext,
        validation: Validation,
        prop_id: PropId,
    ) -> ValidationPrototypeResult<Self> {
        let func_id = Func::find_intrinsic(ctx, IntrinsicFunc::Validation)?;
        let args = serde_json::to_value(FuncBackendValidationArgs::new(validation))?;
        Self::new(ctx, func_id, args, prop_id)
    }
}

// impl ValidationPrototype {
//     standard_model_accessor!(func_id, Pk(FuncId), ValidationPrototypeResult);
//     standard_model_accessor!(args, Json<JsonValue>, ValidationPrototypeResult);
//     standard_model_accessor!(link, Option<String>, ValidationPrototypeResult);
//     standard_model_accessor!(prop_id, Pk(PropId), ValidationPrototypeResult);
//     standard_model_accessor!(schema_id, Pk(SchemaId), ValidationPrototypeResult);
//     standard_model_accessor!(
//         schema_variant_id,
//         Pk(SchemaVariantId),
//         ValidationPrototypeResult
//     );

//     pub fn context(&self) -> ValidationPrototypeContext {
//         ValidationPrototypeContext::new_unchecked(
//             self.prop_id,
//             self.schema_variant_id,
//             self.schema_id,
//         )
//     }

//     /// List all [`ValidationPrototypes`](Self) for a given [`Prop`](crate::Prop).
//     #[instrument(skip_all)]
//     pub async fn list_for_prop(
//         ctx: &DalContext,
//         prop_id: PropId,
//     ) -> ValidationPrototypeResult<Vec<Self>> {
//         let rows = ctx
//             .txns()
//             .await?
//             .pg()
//             .query(LIST_FOR_PROP, &[ctx.tenancy(), ctx.visibility(), &prop_id])
//             .await?;
//         let object = objects_from_rows(rows)?;
//         Ok(object)
//     }

//     /// List all [`ValidationPrototypes`](Self) for all [`Props`](crate::Prop) in a
//     /// [`SchemaVariant`](crate::SchemaVariant).
//     ///
//     /// _You can access the [`PropId`](crate::Prop) via the [`ValidationPrototypeContext`], if
//     /// needed._
//     #[instrument(skip_all)]
//     pub async fn list_for_schema_variant(
//         ctx: &DalContext,
//         schema_variant_id: SchemaVariantId,
//     ) -> ValidationPrototypeResult<Vec<Self>> {
//         let rows = ctx
//             .txns()
//             .await?
//             .pg()
//             .query(
//                 LIST_FOR_SCHEMA_VARIANT,
//                 &[ctx.tenancy(), ctx.visibility(), &schema_variant_id],
//             )
//             .await?;
//         let object = objects_from_rows(rows)?;
//         Ok(object)
//     }

//     /// List all [`ValidationPrototypes`](Self) for a [`Func`](crate::Func)
//     #[instrument(skip_all)]
//     pub async fn list_for_func(
//         ctx: &DalContext,
//         func_id: FuncId,
//     ) -> ValidationPrototypeResult<Vec<Self>> {
//         let rows = ctx
//             .txns()
//             .await?
//             .pg()
//             .query(LIST_FOR_FUNC, &[ctx.tenancy(), ctx.visibility(), &func_id])
//             .await?;

//         Ok(objects_from_rows(rows)?)
//     }

//     pub async fn find_for_context(
//         ctx: &DalContext,
//         context: ValidationPrototypeContext,
//     ) -> ValidationPrototypeResult<Vec<Self>> {
//         let rows = ctx
//             .txns()
//             .await?
//             .pg()
//             .query(
//                 FIND_FOR_CONTEXT,
//                 &[
//                     ctx.tenancy(),
//                     ctx.visibility(),
//                     &context.prop_id(),
//                     &context.schema_variant_id(),
//                     &context.schema_id(),
//                 ],
//             )
//             .await?;

//         Ok(objects_from_rows(rows)?)
//     }

//     pub async fn prop(&self, ctx: &DalContext) -> ValidationPrototypeResult<Prop> {
//         Prop::get_by_id(ctx, &self.prop_id())
//             .await?
//             .ok_or(ValidationPrototypeError::PropNotFound(self.prop_id()))
//     }
// }
