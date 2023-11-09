use content_store::{Store, StoreError};
use postgres_types::{FromSql, ToSql};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use strum::EnumDiscriminants;
use strum::{AsRefStr, Display, EnumIter, EnumString};
use telemetry::prelude::*;
use thiserror::Error;

use si_pkg::FuncArgumentKind as PkgFuncArgumentKind;

use crate::change_set_pointer::ChangeSetPointerError;
use crate::workspace_snapshot::content_address::ContentAddress;
use crate::workspace_snapshot::content_address::ContentAddressDiscriminants;
use crate::workspace_snapshot::edge_weight::{EdgeWeight, EdgeWeightError, EdgeWeightKind};
use crate::workspace_snapshot::node_weight::{ContentNodeWeight, NodeWeight, NodeWeightError};
use crate::workspace_snapshot::WorkspaceSnapshotError;
use crate::{
    impl_standard_model, pk, standard_model, standard_model_accessor, AttributePrototypeId,
    DalContext, FuncId, HistoryEventError, PropKind, StandardModel, StandardModelError, Tenancy,
    Timestamp, TransactionsError, Visibility,
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
            .lock()
            .await
            .add(&FuncArgumentContent::V1(content.clone()))?;

        let change_set = ctx.change_set_pointer()?;
        let id = change_set.generate_ulid()?;
        let node_weight = NodeWeight::new_content(change_set, id, ContentAddress::FuncArg(hash))?;

        let mut workspace_snapshot = ctx.workspace_snapshot()?.lock().await;

        let func_arg_node_index = workspace_snapshot.add_node(node_weight.clone())?;

        let func_node_index = workspace_snapshot.get_node_index_by_id(func_id.into())?;

        workspace_snapshot.add_edge(
            func_node_index,
            EdgeWeight::new(change_set, EdgeWeightKind::Use)?,
            func_arg_node_index,
        )?;

        let content_node_weight =
            node_weight.get_content_node_weight_of_kind(ContentAddressDiscriminants::FuncArg)?;

        Ok(FuncArgument::assemble(&content_node_weight, &content))
    }

    // List all [`FuncArgument`](Self) for the provided [`FuncId`](crate::FuncId).
    //     pub async fn list_for_func(ctx: &DalContext, func_id: FuncId) -> FuncArgumentResult<Vec<Self>> {
    //     }

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

    //     pub async fn remove(
    //         ctx: &DalContext,
    //         func_argument_id: &FuncArgumentId,
    //     ) -> FuncArgumentResult<()> {
    //         let mut func_arg = match FuncArgument::get_by_id(ctx, func_argument_id).await? {
    //             Some(func_arg) => func_arg,
    //             None => return Ok(()),
    //         };
    //
    //         for mut prototype_argument in
    //             AttributePrototypeArgument::list_by_func_argument_id(ctx, *func_argument_id).await?
    //         {
    //             prototype_argument.delete_by_id(ctx).await?;
    //         }
    //
    //         func_arg.delete_by_id(ctx).await?;
    //
    //         Ok(())
    //     }
}
