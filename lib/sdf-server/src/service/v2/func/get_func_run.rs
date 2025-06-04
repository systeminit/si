use std::sync::Arc;

use axum::{
    Json,
    extract::Path,
};
use chrono::{
    DateTime,
    Utc,
};
use dal::{
    ContentHash,
    DalContext,
    WorkspacePk,
};
use serde::{
    Deserialize,
    Serialize,
};
use si_events::{
    ActionId,
    ActionKind,
    ActionPrototypeId,
    ActionResultState,
    Actor,
    AttributeValueId,
    CasValue,
    ChangeSetId,
    ComponentId,
    FuncBackendKind,
    FuncBackendResponseType,
    FuncKind,
    FuncRun,
    FuncRunId,
    FuncRunLog,
    FuncRunLogId,
    FuncRunState,
    OutputLine,
};

use crate::{
    extract::HandlerContext,
    service::v2::{
        AccessBuilder,
        func::FuncAPIResult,
    },
};

/// A one-to-one mapping of cyclone's "OutputStream" type.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct OutputLineView {
    pub stream: String,
    pub execution_id: String,
    pub level: String,
    pub group: Option<String>,
    pub message: String,
    pub timestamp: u64,
}

impl From<&OutputLine> for OutputLineView {
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FuncRunLogView {
    id: FuncRunLogId,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    func_run_id: FuncRunId,
    logs: Vec<OutputLineView>,
    finalized: bool,
}

impl From<FuncRunLog> for FuncRunLogView {
    fn from(func_run_log: FuncRunLog) -> Self {
        FuncRunLogView {
            id: func_run_log.id(),
            created_at: func_run_log.created_at(),
            updated_at: func_run_log.created_at(),
            func_run_id: func_run_log.func_run_id(),
            logs: func_run_log.logs().iter().map(|l| l.into()).collect(),
            finalized: func_run_log.is_finalized(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FuncRunView {
    id: FuncRunId,
    state: FuncRunState,
    actor: Actor,
    component_id: Option<ComponentId>,
    attribute_value_id: Option<AttributeValueId>,
    component_name: Option<String>,
    schema_name: Option<String>,
    action_id: Option<ActionId>,
    action_prototype_id: Option<ActionPrototypeId>,
    action_kind: Option<ActionKind>,
    action_display_name: Option<String>,
    action_originating_change_set_id: Option<ChangeSetId>,
    action_originating_change_set_name: Option<String>,
    action_result_state: Option<ActionResultState>,
    backend_kind: FuncBackendKind,
    backend_response_type: FuncBackendResponseType,
    function_name: String,
    function_display_name: Option<String>,
    function_kind: FuncKind,
    function_description: Option<String>,
    function_link: Option<String>,
    function_args_cas_address: ContentHash,
    function_args: serde_json::Value,
    function_code_base64: String,
    function_code_cas_address: ContentHash,
    result_value_cas_address: Option<ContentHash>,
    result_value: Option<serde_json::Value>,
    result_unprocessed_value_cas_address: Option<ContentHash>,
    logs: Option<FuncRunLogView>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    unprocessed_result_value: Option<serde_json::Value>,
}

impl FuncRunView {
    pub fn new(
        func_run: &FuncRun,
        function_args: serde_json::Value,
        function_code_base64: String,
        result_value: Option<serde_json::Value>,
        logs: Option<FuncRunLogView>,
        unprocessed_result_value: Option<serde_json::Value>,
    ) -> Self {
        FuncRunView {
            id: func_run.id(),
            state: func_run.state(),
            actor: *func_run.actor(),
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
            function_args_cas_address: func_run.function_args_cas_address(),
            function_args,
            function_code_cas_address: func_run.function_code_cas_address(),
            function_code_base64,
            result_value_cas_address: func_run.result_value_cas_address(),
            result_value,
            result_unprocessed_value_cas_address: func_run.result_unprocessed_value_cas_address(),
            logs,
            created_at: func_run.created_at(),
            updated_at: func_run.updated_at(),
            unprocessed_result_value,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Eq, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GetFuncRunResponse {
    pub func_run: Option<FuncRunView>,
}

pub async fn get_func_run_view(ctx: &DalContext, func_run: &FuncRun) -> FuncAPIResult<FuncRunView> {
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

    let logs = ctx
        .layer_db()
        .func_run_log()
        .get_for_func_run_id(func_run.id())
        .await?
        .map(Arc::<FuncRunLog>::unwrap_or_clone)
        .map(|v| v.into());

    Ok(FuncRunView::new(
        func_run,
        func_args,
        code_base64,
        result_value,
        logs,
        None, // TODO(nick): we likely need to add the unprocessed value here as well in the future
    ))
}

pub async fn get_func_run(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    Path((_workspace_pk, change_set_id, func_run_id)): Path<(
        WorkspacePk,
        dal::ChangeSetId,
        FuncRunId,
    )>,
) -> FuncAPIResult<Json<GetFuncRunResponse>> {
    let ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;

    let maybe_func_run = ctx.layer_db().func_run().read(func_run_id).await?;

    match maybe_func_run {
        Some(func_run) => {
            let func_run_view = get_func_run_view(&ctx, &func_run).await?;
            Ok(Json(GetFuncRunResponse {
                func_run: Some(func_run_view),
            }))
        }
        None => Ok(Json(GetFuncRunResponse { func_run: None })),
    }
}
