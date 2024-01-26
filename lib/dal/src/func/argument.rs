use postgres_types::{FromSql, ToSql};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use strum::{AsRefStr, Display, EnumIter, EnumString};
use telemetry::prelude::*;
use thiserror::Error;

use si_pkg::FuncArgumentKind as PkgFuncArgumentKind;

use crate::{
    impl_standard_model, pk, standard_model, standard_model_accessor, AttributePrototypeArgument,
    AttributePrototypeArgumentError, AttributePrototypeId, DalContext, FuncId, HistoryEventError,
    PropKind, StandardModel, StandardModelError, Tenancy, Timestamp, TransactionsError, Visibility,
};

const LIST_FOR_FUNC: &str = include_str!("../queries/func_argument/list_for_func.sql");
const LIST_FOR_FUNC_WITH_PROTOTYPE_ARGUMENTS: &str =
    include_str!("../queries/func_argument/list_for_func_with_prototype_arguments.sql");
const FIND_BY_NAME_FOR_FUNC: &str =
    include_str!("../queries/func_argument/find_by_name_for_func.sql");

#[remain::sorted]
#[derive(Debug, Error)]
pub enum FuncArgumentError {
    #[error("attribute prototype argument error: {0}")]
    AttributePrototypeArgument(#[from] AttributePrototypeArgumentError),
    #[error("history event error: {0}")]
    HistoryEvent(#[from] HistoryEventError),
    #[error("func argument not found with name {0} for Func {1}")]
    NotFoundByNameForFunc(String, FuncId),
    #[error("pg error: {0}")]
    Pg(#[from] si_data_pg::PgError),
    #[error("error serializing/deserializing json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("standard model error: {0}")]
    StandardModelError(#[from] StandardModelError),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
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
    pk: FuncArgumentPk,
    id: FuncArgumentId,
    func_id: FuncId,
    name: String,
    kind: FuncArgumentKind,
    element_kind: Option<FuncArgumentKind>,
    shape: Option<JsonValue>,
    #[serde(flatten)]
    tenancy: Tenancy,
    #[serde(flatten)]
    timestamp: Timestamp,
    #[serde(flatten)]
    visibility: Visibility,
}

impl_standard_model! {
    model: FuncArgument,
    pk: FuncArgumentPk,
    id: FuncArgumentId,
    table_name: "func_arguments",
    history_event_label_base: "func_argument",
    history_event_message_name: "Func Argument"
}

impl FuncArgument {
    pub async fn new(
        ctx: &DalContext,
        name: impl AsRef<str>,
        kind: FuncArgumentKind,
        element_kind: Option<FuncArgumentKind>,
        func_id: FuncId,
    ) -> FuncArgumentResult<Self> {
        let name = name.as_ref();
        let row = ctx
            .txns()
            .await?
            .pg()
            .query_one(
                "SELECT object FROM func_argument_create_v1($1, $2, $3, $4, $5, $6)",
                &[
                    ctx.tenancy(),
                    ctx.visibility(),
                    &func_id,
                    &name,
                    &kind.as_ref(),
                    &element_kind.as_ref().map(|ek| ek.as_ref()),
                ],
            )
            .await?;

        Ok(standard_model::finish_create_from_row(ctx, row).await?)
    }

    standard_model_accessor!(func_id, Pk(FuncId), FuncArgumentResult);
    standard_model_accessor!(name, String, FuncArgumentResult);
    standard_model_accessor!(kind, Enum(FuncArgumentKind), FuncArgumentResult);
    standard_model_accessor!(
        element_kind,
        Option<Enum(FuncArgumentKind)>,
        FuncArgumentResult
    );
    standard_model_accessor!(shape, OptionJson<JsonValue>, FuncArgumentResult);

    /// List all [`FuncArgument`] for the provided [`FuncId`].
    pub async fn list_for_func(ctx: &DalContext, func_id: FuncId) -> FuncArgumentResult<Vec<Self>> {
        let rows = ctx
            .txns()
            .await?
            .pg()
            .query(LIST_FOR_FUNC, &[ctx.tenancy(), ctx.visibility(), &func_id])
            .await?;

        Ok(standard_model::objects_from_rows(rows)?)
    }

    /// List all [`FuncArgument`] for the provided [`FuncId`] along with the [`AttributePrototypeArgument`] that
    /// corresponds to it *if* one exists.
    pub async fn list_for_func_with_prototype_arguments(
        ctx: &DalContext,
        func_id: FuncId,
        attribute_prototype_id: AttributePrototypeId,
    ) -> FuncArgumentResult<Vec<(Self, Option<AttributePrototypeArgument>)>> {
        let rows = ctx
            .txns()
            .await?
            .pg()
            .query(
                LIST_FOR_FUNC_WITH_PROTOTYPE_ARGUMENTS,
                &[
                    ctx.tenancy(),
                    ctx.visibility(),
                    &func_id,
                    &attribute_prototype_id,
                ],
            )
            .await?;

        let mut result = vec![];

        for row in rows.into_iter() {
            let func_argument_json: serde_json::Value = row.try_get("func_argument_object")?;
            let prototype_argument_json: Option<serde_json::Value> =
                row.try_get("prototype_argument_object")?;

            result.push((
                serde_json::from_value(func_argument_json)?,
                match prototype_argument_json {
                    Some(prototype_argument_json) => {
                        Some(serde_json::from_value(prototype_argument_json)?)
                    }
                    None => None,
                },
            ));
        }

        Ok(result)
    }

    pub async fn find_by_name_for_func(
        ctx: &DalContext,
        name: &str,
        func_id: FuncId,
    ) -> FuncArgumentResult<Option<Self>> {
        Ok(
            match ctx
                .txns()
                .await?
                .pg()
                .query_opt(
                    FIND_BY_NAME_FOR_FUNC,
                    &[ctx.tenancy(), ctx.visibility(), &name, &func_id],
                )
                .await?
            {
                Some(row) => standard_model::object_from_row(row)?,
                None => None,
            },
        )
    }

    /// Remove the [`FuncArgument`] along with any [`AttributePrototypeArgument`] rows that reference it. This should be
    /// used instead of the [`delete_by_id`](Self::delete_by_id) method since it keeps the two tables in sync.
    pub async fn remove(
        ctx: &DalContext,
        func_argument_id: &FuncArgumentId,
    ) -> FuncArgumentResult<()> {
        let mut func_arg = match FuncArgument::get_by_id(ctx, func_argument_id).await? {
            Some(func_arg) => func_arg,
            None => return Ok(()),
        };

        for mut prototype_argument in
            AttributePrototypeArgument::list_by_func_argument_id(ctx, *func_argument_id).await?
        {
            prototype_argument.delete_by_id(ctx).await?;
        }

        func_arg.delete_by_id(ctx).await?;

        Ok(())
    }
}
