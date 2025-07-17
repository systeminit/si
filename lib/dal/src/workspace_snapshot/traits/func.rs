use async_trait::async_trait;
use si_id::FuncId;

use crate::{
    WorkspaceSnapshot,
    func::{
        FuncResult,
        intrinsics::IntrinsicFunc,
    },
};

#[async_trait]
pub trait FuncExt {
    /// A non-dynamic Func is an Intrinsic func that returns a fixed value, set by a StaticArgumentValue in the graph
    /// opposingly, a dynamic Func is a func that returns a non statically predictable value, possibly user defined.
    ///
    /// It's important to note that not all Intrinsic funcs are non-dynamic. Identity, for instance, is dynamic.
    async fn func_is_dynamic(&self, func_id: FuncId) -> FuncResult<bool>;
}

#[async_trait]
impl FuncExt for WorkspaceSnapshot {
    async fn func_is_dynamic(&self, func_id: FuncId) -> FuncResult<bool> {
        let func = self
            .get_node_weight(func_id)
            .await?
            .get_func_node_weight()?;
        match IntrinsicFunc::maybe_from_str(func.name()) {
            None => Ok(true),
            Some(intrinsic_func) => Ok(intrinsic_func.is_dynamic()),
        }
    }
}
