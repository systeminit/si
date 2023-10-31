use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;
use veritech_client::{
    BeforeFunctionRequest, FunctionResult, ReconciliationRequest, ReconciliationResultSuccess,
};

use crate::func::backend::{ExtractPayload, FuncBackendResult, FuncDispatch, FuncDispatchContext};
use crate::AttributeValueId;

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ReconciliationDiffDomain {
    pub id: AttributeValueId,
    pub value: serde_json::Value,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ReconciliationDiff {
    pub normalized_resource: Option<serde_json::Value>,
    pub resource: serde_json::Value,
    pub domain: ReconciliationDiffDomain,
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct FuncBackendJsReconciliationArgs(HashMap<String, ReconciliationDiff>);

#[derive(Debug)]
pub struct FuncBackendJsReconciliation {
    pub context: FuncDispatchContext,
    pub request: ReconciliationRequest,
}

#[async_trait]
impl FuncDispatch for FuncBackendJsReconciliation {
    type Args = FuncBackendJsReconciliationArgs;
    type Output = ReconciliationResultSuccess;

    fn new(
        context: FuncDispatchContext,
        code_base64: &str,
        handler: &str,
        args: Self::Args,
        before: Vec<BeforeFunctionRequest>,
    ) -> Box<Self> {
        let request = ReconciliationRequest {
            // Once we start tracking the state of these executions, then this id will be useful,
            // but for now it's passed along and back, and is opaue
            execution_id: "freeronaldinhogauchojsreconciliation".to_string(),
            handler: handler.into(),
            code_base64: code_base64.into(),
            args: serde_json::to_value(args).unwrap(),
            before,
        };

        Box::new(Self { context, request })
    }

    /// This private function dispatches the assembled request to veritech for execution.
    /// This is the "last hop" function in the dal before using the veritech client directly.
    async fn dispatch(self: Box<Self>) -> FuncBackendResult<FunctionResult<Self::Output>> {
        let (veritech, output_tx) = self.context.into_inner();
        let value = veritech
            .execute_reconciliation(output_tx.clone(), &self.request)
            .await?;
        let value = match value {
            FunctionResult::Failure(failure) => FunctionResult::Success(Self::Output {
                execution_id: failure.execution_id,
                updates: Default::default(),
                actions: Default::default(),
                message: Some(failure.error.message.clone()),
            }),
            FunctionResult::Success(value) => FunctionResult::Success(value),
        };

        Ok(value)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct ReconciliationResult {
    pub updates: HashMap<AttributeValueId, serde_json::Value>,
    pub actions: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub message: Option<String>,
}

impl ExtractPayload for ReconciliationResultSuccess {
    type Payload = ReconciliationResult;

    fn extract(self) -> FuncBackendResult<Self::Payload> {
        Ok(ReconciliationResult {
            updates: self
                .updates
                .into_iter()
                .map(|(k, v)| Ok((AttributeValueId::from_str(&k)?, v)))
                .collect::<FuncBackendResult<HashMap<_, _>>>()?,
            actions: self.actions,
            message: self.message,
        })
    }
}
