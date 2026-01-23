use chrono::{
    DateTime,
    Utc,
};
use dal::{
    ActionPrototypeId,
    ChangeSetId,
    ComponentId,
    DalContext,
};
use serde::{
    Deserialize,
    Serialize,
};
use si_db::FuncRunLogDb;
use si_events::{
    ActionKind,
    ActionResultState,
    CasValue,
    FuncBackendKind,
    FuncBackendResponseType,
    FuncKind,
    FuncRun,
    FuncRunLog,
    FuncRunState,
    OutputLine,
};
use si_id::{
    ActionId,
    AttributeValueId,
    FuncRunId,
    FuncRunLogId,
};
use utoipa::ToSchema;

use crate::service::v1::FuncsResult;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct FuncRunViewV1 {
    #[schema(value_type = String, example = "01JQCJ0AAXGX5M9QY10AVF4GK1")]
    id: FuncRunId,
    #[schema(value_type = String, example = "Success")]
    state: FuncRunState,
    #[schema(value_type = Option<String>, example = "null")]
    attribute_value_id: Option<AttributeValueId>,
    #[schema(value_type = Option<String>, example = "01JP8KHZP3DZKGNXRP83Q6WTQ5")]
    component_id: Option<ComponentId>,
    #[schema(value_type = Option<String>, example = "NAT Gateway IP 1")]
    component_name: Option<String>,
    #[schema(value_type = Option<String>, example = "AWS::EC2::EIP")]
    schema_name: Option<String>,
    #[schema(value_type = Option<String>, example = "01JQCHZZY99G3R0C1FA3W4AFR6")]
    action_id: Option<ActionId>,
    #[schema(value_type = Option<String>, example = "01JPNHEE9Z3DFW48XVZ1FX04KA")]
    action_prototype_id: Option<ActionPrototypeId>,
    #[schema(value_type = Option<String>, example = "Destroy")]
    action_kind: Option<ActionKind>,
    #[schema(value_type = Option<String>, example = "Destroy")]
    action_display_name: Option<String>,
    #[schema(value_type = Option<String>, example = "01JQCHZZVTAHHZ7DG0ZSCB9RXB")]
    action_originating_change_set_id: Option<ChangeSetId>,
    #[schema(value_type = Option<String>, example = "2025-03-27-19:41")]
    action_originating_change_set_name: Option<String>,
    #[schema(value_type = Option<String>, example = "Success")]
    action_result_state: Option<ActionResultState>,
    #[schema(value_type = String, example = "JsAction")]
    backend_kind: FuncBackendKind,
    #[schema(value_type = String, example = "Action")]
    backend_response_type: FuncBackendResponseType,
    #[schema(value_type = String, example = "Delete Asset")]
    function_name: String,
    #[schema(value_type = Option<String>, example = "null")]
    function_display_name: Option<String>,
    #[schema(value_type = String, example = "Action")]
    function_kind: FuncKind,
    #[schema(value_type = Option<String>, example = "null")]
    function_description: Option<String>,
    #[schema(value_type = Option<String>, example = "null")]
    function_link: Option<String>,
    #[schema(example = json!({
        "properties": {
            "code": {
                "awsCloudControlCreate": {
                    "code": "{\n  \"TypeName\": \"AWS::EC2::EIP\",\n  \"DesiredState\": {\n    \"Domain\": \"vpc\"\n  }\n}",
                    "format": "json"
                },
                "awsCloudControlUpdate": {
                    "code": "{\n  \"TypeName\": \"AWS::EC2::EIP\",\n  \"DesiredState\": {}\n}",
                    "format": "json"
                }
            },
            "domain": {
                "Domain": "vpc",
                "Tags": [],
                "extra": {
                    "AwsResourceType": "AWS::EC2::EIP",
                    "Region": "us-east-1"
                }
            },
            "resource": {
                "payload": {
                    "AllocationId": "eipalloc-033720f9556a3b0c1",
                    "PublicIp": "3.213.242.163"
                }
            },
            "si": {
                "name": "NAT Gateway IP 1",
                "resourceId": "3.213.242.163|eipalloc-033720f9556a3b0c1",
                "type": "component"
            }
        }
    }))]
    function_args: serde_json::Value,
    #[schema(value_type = String, example = "YXN5bmMgZnVuY3Rpb24gbWFpbihjb21wb2...")]
    function_code_base64: String,
    #[schema(example = json!({
        "error": null,
        "executionId": "01JQCJ0AAXGX5M9QY10AVF4GK1",
        "message": null,
        "payload": null,
        "resourceId": null,
        "status": "ok"
    }))]
    result_value: Option<serde_json::Value>,
    #[schema(example = json!({
        "id": "01JQCJ0ABJSCE01GNQDWVY1ZP5",
        "createdAt": "2025-03-27T19:41:58.514416748Z",
        "updatedAt": "2025-03-27T19:41:58.514416748Z",
        "funcRunId": "01JQCJ0AAXGX5M9QY10AVF4GK1",
        "logs": [
            {
                "stream": "stdout",
                "executionId": "",
                "level": "info",
                "group": "log",
                "message": "Running CLI command: \"aws 'cloudcontrol' 'delete-resource'\"",
                "timestamp": 1743104518
            },
            {
                "stream": "output",
                "executionId": "01JQCJ0AAXGX5M9QY10AVF4GK1",
                "level": "info",
                "group": "log",
                "message": "Output: {\"protocol\":\"result\",\"status\":\"success\"}",
                "timestamp": 1743104521
            }
        ],
        "finalized": true
    }))]
    logs: Option<FuncRunLogViewV1>,
    #[schema(value_type = String, example = "2025-03-27T19:41:58.493298051Z")]
    created_at: DateTime<Utc>,
    #[schema(value_type = String, example = "2025-03-27T19:42:02.192033089Z")]
    updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct FuncRunLogViewV1 {
    #[schema(value_type = String, example = "01JQCJ0ABJSCE01GNQDWVY1ZP5")]
    id: FuncRunLogId,
    #[schema(value_type = String, example = "2025-03-27T19:41:58.514416748Z")]
    created_at: DateTime<Utc>,
    #[schema(value_type = String, example = "2025-03-27T19:41:58.514416748Z")]
    updated_at: DateTime<Utc>,
    #[schema(value_type = String, example = "01JQCJ0AAXGX5M9QY10AVF4GK1")]
    func_run_id: FuncRunId,
    #[schema(value_type = Vec<Object>, example = json!([
        {
            "stream": "stdout",
            "executionId": "",
            "level": "info",
            "group": "log",
            "message": "Running CLI command: \"aws 'cloudcontrol' 'delete-resource'\"",
            "timestamp": 1743104518
        },
        {
            "stream": "output",
            "executionId": "01JQCJ0AAXGX5M9QY10AVF4GK1",
            "level": "info",
            "group": "log",
            "message": "Output: {\"protocol\":\"result\",\"status\":\"success\"}",
            "timestamp": 1743104521
        }
    ]))]
    logs: Vec<OutputLineViewV1>,
    #[schema(value_type = bool, example = true)]
    finalized: bool,
}

impl From<&OutputLine> for OutputLineViewV1 {
    fn from(output_line: &OutputLine) -> Self {
        Self {
            stream: output_line.stream.clone(),
            execution_id: output_line.execution_id.clone(),
            level: output_line.level.clone(),
            group: output_line.group.clone(),
            message: output_line.message.clone(),
            timestamp: output_line.timestamp,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct OutputLineViewV1 {
    #[schema(example = "stdout")]
    pub stream: String,
    #[schema(example = "01JQCJ0AAXGX5M9QY10AVF4GK1")]
    pub execution_id: String,
    #[schema(example = "info")]
    pub level: String,
    #[schema(example = "log")]
    pub group: Option<String>,
    #[schema(example = "Running CLI command: \"aws 'cloudcontrol' 'delete-resource'\"")]
    pub message: String,
    #[schema(example = 1743104518)]
    pub timestamp: u64,
}

impl From<FuncRunLog> for FuncRunLogViewV1 {
    fn from(func_run_log: FuncRunLog) -> Self {
        FuncRunLogViewV1 {
            id: func_run_log.id(),
            created_at: func_run_log.created_at(),
            updated_at: func_run_log.created_at(),
            func_run_id: func_run_log.func_run_id(),
            logs: func_run_log.logs().iter().map(|l| l.into()).collect(),
            finalized: func_run_log.is_finalized(),
        }
    }
}

impl FuncRunViewV1 {
    pub async fn assemble(ctx: &DalContext, func_run: &FuncRun) -> FuncsResult<Self> {
        let arguments: Option<CasValue> = ctx
            .layer_db()
            .cas()
            .try_read_as(&func_run.function_args_cas_address())
            .await?;
        let func_args: serde_json::Value = match arguments {
            Some(func_args_cas_value) => func_args_cas_value.into(),
            None => serde_json::Value::Null,
        };

        let code: Option<CasValue> = ctx
            .layer_db()
            .cas()
            .try_read_as(&func_run.function_code_cas_address())
            .await?;
        let code_base64: String = match code {
            Some(code_base64_cas_value) => {
                let code_base64_cas_value: serde_json::Value = code_base64_cas_value.into();
                match code_base64_cas_value.as_str() {
                    Some(code_base64_str) => code_base64_str.to_string(),
                    None => "".to_string(),
                }
            }
            None => "".to_string(),
        };

        let result_value: Option<serde_json::Value> = {
            match func_run.result_value_cas_address() {
                Some(result_value_cas_address) => {
                    let result_value_cas: Option<CasValue> = ctx
                        .layer_db()
                        .cas()
                        .try_read_as(&result_value_cas_address)
                        .await?;
                    result_value_cas.map(|r| r.into())
                }
                None => None,
            }
        };

        let logs = FuncRunLogDb::get_for_func_run_id(ctx, func_run.id())
            .await?
            .map(|v| v.into());

        Ok(FuncRunViewV1 {
            id: func_run.id(),
            state: func_run.state(),
            component_id: func_run.component_id(),
            attribute_value_id: func_run.attribute_value_id(),
            component_name: func_run.component_name().map(|v| v.to_string()),
            schema_name: func_run.schema_name().map(|v| v.to_string()),
            action_id: func_run.action_id(),
            action_prototype_id: func_run.action_prototype_id(),
            action_kind: func_run.action_kind(),
            action_display_name: func_run.action_display_name().map(|v| v.to_string()),
            action_originating_change_set_id: func_run.action_originating_change_set_id(),
            action_originating_change_set_name: func_run
                .action_originating_change_set_name()
                .map(|v| v.to_string()),
            action_result_state: func_run.action_result_state(),
            backend_kind: func_run.backend_kind(),
            backend_response_type: func_run.backend_response_type(),
            function_name: func_run.function_name().to_string(),
            function_display_name: func_run.function_display_name().map(|v| v.to_string()),
            function_kind: func_run.function_kind(),
            function_description: func_run.function_description().map(|v| v.to_string()),
            function_link: func_run.function_link().map(|v| v.to_string()),
            function_args: func_args,
            function_code_base64: code_base64,
            result_value,
            logs,
            created_at: func_run.created_at(),
            updated_at: func_run.updated_at(),
        })
    }
}
