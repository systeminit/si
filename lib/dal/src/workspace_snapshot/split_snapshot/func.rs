use async_trait::async_trait;
use si_id::FuncId;

use crate::{
    func::{
        FuncResult,
        intrinsics::IntrinsicFunc,
    },
    workspace_snapshot::{
        split_snapshot::SplitSnapshot,
        traits::func::FuncExt,
    },
};

#[async_trait]
impl FuncExt for SplitSnapshot {
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
