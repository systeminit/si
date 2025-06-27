use color_eyre::eyre::eyre;
use dal::{
    DalContext,
    Func,
    FuncId,
};

use crate::{
    Result,
    expected::ExpectFunc,
};

/// Look up a function by its key (name or id).
pub async fn id(ctx: &DalContext, key: impl FuncKey) -> Result<FuncId> {
    FuncKey::id(ctx, key).await
}

///
/// Things that you can pass to reference funcs (name or id)
///
#[allow(async_fn_in_trait)]
pub trait FuncKey {
    ///
    /// Turn this into a real FuncId
    ///
    async fn id(ctx: &DalContext, key: Self) -> Result<FuncId>;
}
impl FuncKey for FuncId {
    async fn id(_: &DalContext, key: Self) -> Result<FuncId> {
        Ok(key)
    }
}
// "FuncName" finds the component with that name
impl FuncKey for &str {
    async fn id(ctx: &DalContext, key: Self) -> Result<FuncId> {
        Func::find_id_by_name(ctx, key)
            .await?
            .ok_or_else(|| eyre!("Function not found: {}", key))
    }
}
impl FuncKey for ExpectFunc {
    async fn id(_: &DalContext, key: Self) -> Result<FuncId> {
        Ok(key.id())
    }
}
impl FuncKey for Func {
    async fn id(_: &DalContext, key: Self) -> Result<FuncId> {
        Ok(key.id)
    }
}
