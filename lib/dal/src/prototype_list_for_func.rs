use crate::{
    standard_model::objects_from_rows, DalContext, FuncId, StandardModel, StandardModelError,
    TenancyError, TransactionsError,
};
use async_trait::async_trait;
use serde::de::DeserializeOwned;
use thiserror::Error;

#[remain::sorted]
#[derive(Error, Debug)]
pub enum PrototypeListForFuncError {
    #[error(transparent)]
    ContextTransaction(#[from] TransactionsError),
    #[error(transparent)]
    Pg(#[from] si_data_pg::PgError),
    #[error(transparent)]
    PgPool(#[from] si_data_pg::PgPoolError),
    #[error("json serialization error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error(transparent)]
    StandardModel(#[from] StandardModelError),
    #[error("tenancy error: {0}")]
    Tenancy(#[from] TenancyError),
}

pub type PrototypeListForFuncResult<T> = Result<T, PrototypeListForFuncError>;

#[async_trait]
pub trait PrototypeListForFunc
where
    Self: Sized + DeserializeOwned + StandardModel,
{
    async fn list_for_func(
        ctx: &DalContext,
        func_id: FuncId,
    ) -> PrototypeListForFuncResult<Vec<Self>>;
}

pub async fn prototype_list_for_func<T: DeserializeOwned>(
    ctx: &DalContext,
    table_name: &str,
    func_id: FuncId,
) -> PrototypeListForFuncResult<Vec<T>> {
    let txns = ctx.txns().await?;
    let rows = txns
        .pg()
        .query(
            "SELECT * FROM prototype_list_for_func_v1($1, $2, $3, $4)",
            &[&table_name, ctx.tenancy(), ctx.visibility(), &func_id],
        )
        .await?;

    Ok(objects_from_rows(rows)?)
}

#[macro_export]
macro_rules! impl_prototype_list_for_func {
    (model: $model:ident) => {
        #[async_trait::async_trait]
        impl $crate::PrototypeListForFunc for $model {
            async fn list_for_func(
                ctx: &DalContext,
                func_id: FuncId,
            ) -> $crate::PrototypeListForFuncResult<Vec<Self>> {
                $crate::prototype_list_for_func::prototype_list_for_func(
                    ctx,
                    $model::table_name(),
                    func_id,
                )
                .await
            }
        }
    };
}
