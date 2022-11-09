use async_trait::async_trait;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use veritech_client::{
    FunctionResult, OutputStream, QualificationCheckComponent, QualificationCheckRequest,
    QualificationCheckResultSuccess,
};

use crate::func::backend::{
    ExtractPayload, FuncBackendError, FuncBackendResult, FuncDispatch, FuncDispatchContext,
};
use crate::qualification::QualificationResult;

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct FuncBackendJsQualificationArgs {
    pub component: QualificationCheckComponent,
}

#[derive(Debug)]
pub struct FuncBackendJsQualification {
    context: FuncDispatchContext,
    request: QualificationCheckRequest,
}

#[async_trait]
impl FuncDispatch for FuncBackendJsQualification {
    type Args = FuncBackendJsQualificationArgs;
    type Output = QualificationCheckResultSuccess;

    fn new(
        context: FuncDispatchContext,
        code_base64: &str,
        handler: &str,
        args: Self::Args,
    ) -> Box<Self> {
        let request = QualificationCheckRequest {
            // Once we start tracking the state of these executions, then this id will be useful,
            // but for now it's passed along and back, and is opaue
            execution_id: "danielcraig".to_string(),
            handler: handler.into(),
            code_base64: code_base64.into(),
            component: args.component,
        };

        Box::new(Self { context, request })
    }

    async fn dispatch(self: Box<Self>) -> FuncBackendResult<FunctionResult<Self::Output>> {
        let (veritech, output_tx) = self.context.into_inner();
        let value = veritech
            .execute_qualification_check(output_tx.clone(), &self.request)
            .await?;
        match &value {
            FunctionResult::Failure(_) => {}
            FunctionResult::Success(value) => {
                if let Some(message) = &value.message {
                    output_tx
                        .send(OutputStream {
                            execution_id: self.request.execution_id,
                            stream: "return".to_owned(),
                            level: "info".to_owned(),
                            group: None,
                            message: message.clone(),
                            timestamp: std::cmp::max(Utc::now().timestamp(), 0) as u64,
                        })
                        .await
                        .map_err(|_| FuncBackendError::SendError)?;
                } else {
                }
            }
        }

        Ok(value)
    }
}

impl ExtractPayload for QualificationCheckResultSuccess {
    type Payload = QualificationResult;

    fn extract(self) -> FuncBackendResult<Self::Payload> {
        Ok(QualificationResult {
            success: self.qualified,
            title: self.title,
            link: self.link,
            sub_checks: self.sub_checks,
        })
    }
}
