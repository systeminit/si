use axum::{
    extract::Query,
    response::Json,
};
use dal::{
    AttributeValue,
    Component,
    ComponentId,
    DalContext,
};
use dal_summary_generator::ComponentSummaryGenerator;
use serde::Serialize;
use serde_json::{
    Value,
    json,
};
use utoipa::{
    self,
    ToSchema,
};
use si_db::{
    FuncRunDb,
    FuncRunLogDb,
};
use super::ComponentsError;
use crate::{
    extract::{
        PosthogEventTracker,
        change_set::ChangeSetDalContext,
    },
    service::v1::common::QueryStringPaginationParams,
};

#[derive(serde::Deserialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ListComponentsParams {
    #[serde(flatten)]
    pub pagination: QueryStringPaginationParams,

    // Existing option
    pub include_codegen: Option<bool>,

    // Graph summary options
    pub include_all: Option<bool>,
    pub include_functions: Option<bool>,
    pub include_subscriptions: Option<bool>,
    pub include_manages: Option<bool>,
    pub include_action_functions: Option<bool>,
    pub include_management_functions: Option<bool>,
    pub include_qualification_functions: Option<bool>,
    pub include_resource_info: Option<bool>,
    pub include_diff_status: Option<bool>,
    pub include_execution_history: Option<bool>,
}

#[derive(Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ListComponentsV1Response {
    #[schema(
        value_type = Vec<ComponentDetailsV1>,
        example = json!([
            {
                "component_id": "01H9ZQD35JPMBGHH69BT0Q79AA",
                "name": "my-vpc",
                "schema_name": "AWS::EC2::VPC"
            },
            {
                "component_id": "01H9ZQD35JPMBGHH69BT0Q79BB",
                "name": "Public 1",
                "schema_name": "AWS::EC2::Subnet"
            }
        ])
    )]
    pub component_details: Vec<ComponentDetailsV1>,
    pub next_cursor: Option<String>,
}

#[derive(Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ComponentDetailsV1 {
    #[schema(value_type = String)]
    pub component_id: ComponentId,
    pub name: String,
    pub schema_name: String,
    pub codegen: Option<Value>,

    // Optional graph summary fields (included when graph summary parameters are used)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_resource: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_diff: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diff_status: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub subscriptions: Vec<SubscriptionRelationshipV1>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub manages: Vec<ManagementRelationshipV1>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub action_functions: Vec<FunctionRelationshipV1>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub management_functions: Vec<FunctionRelationshipV1>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub qualification_functions: Vec<FunctionRelationshipV1>,
}

#[utoipa::path(
    get,
    path = "/v1/w/{workspace_id}/change-sets/{change_set_id}/components",
    params(
        ("workspace_id" = String, Path, description = "Workspace identifier"),
        ("change_set_id" = String, Path, description = "Change Set identifier"),
        ("limit" = Option<String>, Query, description = "Maximum number of results to return (default: 50, max: 300)"),
        ("cursor" = Option<String>, Query, description = "Cursor for pagination (ComponentId of the last item from previous page)"),
        ("includeCodegen" = Option<bool>, Query, description = "Allow returning the codegen for the cloudformation template for the component (if it exists)"),
        ("includeAll" = Option<bool>, Query, description = "Include all graph summary data (equivalent to enabling all include options)"),
        ("includeFunctions" = Option<bool>, Query, description = "Include all function types (action, management, qualification)"),
        ("includeSubscriptions" = Option<bool>, Query, description = "Include subscription relationships"),
        ("includeManages" = Option<bool>, Query, description = "Include management relationships"),
        ("includeActionFunctions" = Option<bool>, Query, description = "Include action function relationships"),
        ("includeManagementFunctions" = Option<bool>, Query, description = "Include management function relationships"),
        ("includeQualificationFunctions" = Option<bool>, Query, description = "Include qualification function relationships"),
        ("includeResourceInfo" = Option<bool>, Query, description = "Include resource information (resource ID and status)"),
        ("includeDiffStatus" = Option<bool>, Query, description = "Include component diff status vs HEAD (Added/Modified/None)"),
        ("includeExecutionHistory" = Option<bool>, Query, description = "Include last 10 execution history entries for each function"),
    ),
    summary = "List all components",
    tag = "components",
    responses(
        (status = 200, description = "Components retrieved successfully", body = ListComponentsV1Response, example = json!({
                    "componentDetails": [
                        {
                            "component_id": "01H9ZQD35JPMBGHH69BT0Q79AA",
                            "name": "my-vpc",
                            "schema_name": "AWS::EC2::VPC"
                        },
                        {
                            "component_id": "01H9ZQD35JPMBGHH69BT0Q79BB",
                            "name": "Public 1",
                            "schema_name": "AWS::EC2::Subnet"
                        }
                    ],
                    "nextCursor": null
                })),
        (status = 401, description = "Unauthorized - Invalid or missing token"),
        (status = 500, description = "Internal server error", body = crate::service::v1::common::ApiError)
    )
)]
#[allow(deprecated)]
pub async fn list_components(
    ChangeSetDalContext(ref ctx): ChangeSetDalContext,
    Query(params): Query<ListComponentsParams>,
    tracker: PosthogEventTracker,
) -> Result<Json<ListComponentsV1Response>, ComponentsError> {
    let maybe_summary_generator_config = SummaryGeneratorConfig::from_params(&params);

    // Set default limit and enforce a max limit
    let limit = params
        .pagination
        .limit
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(50)
        .min(300) as usize;
    let include_codegen = params.include_codegen;
    let cursor = params.pagination.cursor;

    let (component_details, next_cursor) = list_components_inner(
        ctx,
        limit,
        cursor,
        include_codegen,
        maybe_summary_generator_config,
    )
    .await?;

    tracker.track(ctx, "api_list_components", json!({}));

    Ok(Json(ListComponentsV1Response {
        component_details,
        next_cursor,
    }))
}

// NOTE(nick): we can instrument this if we need to
async fn list_components_inner(
    ctx: &DalContext,
    limit: usize,
    cursor: Option<String>,
    include_codegen: Option<bool>,
    maybe_summary_generator_config: Option<SummaryGeneratorConfig>,
) -> Result<(Vec<ComponentDetailsV1>, Option<String>), ComponentsError> {
    let mut component_details_vec = Vec::with_capacity(limit);

    // Get all component IDs. This ensures that we do not access CAS. Then, sort for consistent
    // pagination.
    let mut all_component_ids = Component::list_ids(ctx).await?;
    all_component_ids.sort();

    // Find the start index by matching the ID.
    let start_index = if let Some(ref cursor_str) = cursor {
        match all_component_ids
            .iter()
            .position(|id| id.to_string() == *cursor_str)
        {
            Some(index) => index + 1, // Start after the cursor
            None => 0,
        }
    } else {
        0 // Start from the beginning
    };

    // Compute the end index and extract the paginated slice
    let end_index = (start_index + limit).min(all_component_ids.len());
    let paginated_component_ids: Vec<ComponentId> =
        all_component_ids[start_index..end_index].to_vec();

    // Generate the next cursor from the last item's ID
    let next_cursor = if end_index < all_component_ids.len() && !paginated_component_ids.is_empty()
    {
        paginated_component_ids.last().map(|id| id.to_string())
    } else {
        None
    };

    for paginated_component_id in paginated_component_ids.iter().copied() {
        component_details_vec.push(ComponentDetailsV1 {
            component_id: paginated_component_id,
            name: Component::name_by_id(ctx, paginated_component_id).await?,
            schema_name: Component::schema_for_component_id(ctx, paginated_component_id)
                .await?
                .name,

            // Existed before graph summary fields
            codegen: match include_codegen {
                Some(codegen) if codegen => {
                    let code_map_av_id =
                        Component::find_code_map_attribute_value_id(ctx, paginated_component_id)
                            .await?;
                    AttributeValue::view(ctx, code_map_av_id)
                        .await?
                        .and_then(|view| view.get("awsCloudFormationLint").cloned())
                }
                Some(_) | None => None,
            },

            // Initialize graph summary fields
            has_resource: None,
            resource_id: None,
            resource_status: None,
            has_diff: None,
            diff_status: None,
            subscriptions: Vec::new(),
            manages: Vec::new(),
            action_functions: Vec::new(),
            management_functions: Vec::new(),
            qualification_functions: Vec::new(),
        });
    }

    // If any graph summary options are enabled (i.e. a "config" was provided to this function),
    // populate the graph data.
    if let Some(config) = maybe_summary_generator_config {
        let generator = ComponentSummaryGenerator::new(
            config.include_action_functions,
            config.include_diff_status,
            config.include_execution_history,
            config.include_management_functions,
            config.include_manages,
            config.include_qualification_functions,
            config.include_resource_info,
            config.include_subscriptions,
        );

        // Build graph summary for the components. Drop the HEAD context from scope as soon as we
        // no longer need it.
        let graph_summaries = {
            let head_ctx = ctx.clone_with_head().await?;
            generator
                .generate(ctx, &head_ctx, paginated_component_ids.as_slice())
                .await?
        };

        // Merge graph summary data into the component details collection.
        for component_details in &mut component_details_vec {
            if let Some(graph_data) =
                graph_summaries.get(&component_details.component_id.to_string())
            {
                component_details.has_resource = graph_data.has_resource;
                component_details.resource_id = graph_data.resource_id.clone();
                component_details.resource_status = graph_data.resource_status.clone();
                component_details.has_diff = graph_data.has_diff;
                component_details.diff_status = graph_data.diff_status.clone();
                component_details.subscriptions = graph_data
                    .subscriptions
                    .iter()
                    .cloned()
                    .map(Into::into)
                    .collect();
                component_details.manages =
                    graph_data.manages.iter().cloned().map(Into::into).collect();
                component_details.action_functions = graph_data
                    .action_functions
                    .iter()
                    .cloned()
                    .map(Into::into)
                    .collect();
                component_details.management_functions = graph_data
                    .management_functions
                    .iter()
                    .cloned()
                    .map(Into::into)
                    .collect();
                component_details.qualification_functions = graph_data
                    .qualification_functions
                    .iter()
                    .cloned()
                    .map(Into::into)
                    .collect();
            }
        }
    }

    Ok((component_details_vec, next_cursor))
}

// Graph summary data structures
#[derive(Serialize, Debug, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SubscriptionRelationshipV1 {
    #[schema(value_type = String)]
    pub to_component_id: ComponentId,
    pub to_component_name: String,
    pub from_path: String,
    pub to_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_value: Option<serde_json::Value>,
}

#[derive(Serialize, Debug, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ManagementRelationshipV1 {
    #[schema(value_type = String)]
    pub to_component_id: ComponentId,
    pub to_component_name: String,
}

#[derive(Serialize, Debug, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct FunctionRelationshipV1 {
    pub function_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execution_status: Option<FunctionExecutionStatusV1>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[schema(value_type = Vec<String>)]
    pub depends_on: Vec<si_id::ActionId>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub execution_history: Vec<ExecutionHistoryEntry>,
}

#[derive(Serialize, Debug, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct FunctionExecutionStatusV1 {
    pub state: String,
    pub has_active_run: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(value_type = Option<String>)]
    pub func_run_id: Option<si_id::FuncRunId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(value_type = Option<String>)]
    pub action_id: Option<si_id::ActionId>,
}

#[derive(Serialize, Debug, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionHistoryEntry {
    #[schema(value_type = String)]
    pub func_run_id: si_id::FuncRunId,
    pub state: String,
    #[schema(value_type = String, format = DateTime)]
    pub started_at: chrono::DateTime<chrono::Utc>,
}

impl From<dal_summary_generator::SubscriptionRelationship> for SubscriptionRelationshipV1 {
    fn from(value: dal_summary_generator::SubscriptionRelationship) -> Self {
        Self {
            to_component_id: value.to_component_id,
            to_component_name: value.to_component_name,
            from_path: value.from_path,
            to_path: value.to_path,
            current_value: value.current_value,
        }
    }
}

impl From<dal_summary_generator::ManagementRelationship> for ManagementRelationshipV1 {
    fn from(value: dal_summary_generator::ManagementRelationship) -> Self {
        Self {
            to_component_id: value.to_component_id,
            to_component_name: value.to_component_name,
        }
    }
}

impl From<dal_summary_generator::FunctionRelationship> for FunctionRelationshipV1 {
    fn from(value: dal_summary_generator::FunctionRelationship) -> Self {
        Self {
            function_name: value.function_name,
            execution_status: value.execution_status.map(Into::into),
            depends_on: value.depends_on,
            execution_history: value
                .execution_history
                .into_iter()
                .map(Into::into)
                .collect(),
        }
    }
}

impl From<dal_summary_generator::FunctionExecutionStatus> for FunctionExecutionStatusV1 {
    fn from(value: dal_summary_generator::FunctionExecutionStatus) -> Self {
        Self {
            state: value.state,
            has_active_run: value.has_active_run,
            func_run_id: value.func_run_id,
            action_id: value.action_id,
        }
    }
}

impl From<dal_summary_generator::ExecutionHistoryEntry> for ExecutionHistoryEntry {
    fn from(value: dal_summary_generator::ExecutionHistoryEntry) -> Self {
        Self {
            func_run_id: value.func_run_id,
            state: value.state,
            started_at: value.started_at,
        }
    }
}

#[derive(Debug)]
struct SummaryGeneratorConfig {
    include_action_functions: bool,
    include_diff_status: bool,
    include_execution_history: bool,
    include_management_functions: bool,
    include_manages: bool,
    include_qualification_functions: bool,
    include_resource_info: bool,
    include_subscriptions: bool,
}

impl SummaryGeneratorConfig {
    pub fn from_params(params: &ListComponentsParams) -> Option<Self> {
        // Evaluate the parameters that cannot be overriden.
        let include_all = params.include_all.unwrap_or(false);
        let include_functions = params.include_functions.unwrap_or(false);

        // Evaluate the parameters that may be overriden.
        let mut include_action_functions = params.include_action_functions.unwrap_or(false);
        let mut include_diff_status = params.include_diff_status.unwrap_or(false);
        let mut include_execution_history = params.include_execution_history.unwrap_or(false);
        let mut include_management_functions = params.include_management_functions.unwrap_or(false);
        let mut include_manages = params.include_manages.unwrap_or(false);
        let mut include_qualification_functions =
            params.include_qualification_functions.unwrap_or(false);
        let mut include_resource_info = params.include_resource_info.unwrap_or(false);
        let mut include_subscriptions = params.include_subscriptions.unwrap_or(false);

        // Check if we are including any graph data.
        let include_any_graph_data = include_action_functions
            || include_all
            || include_diff_status
            || include_execution_history
            || include_functions
            || include_management_functions
            || include_manages
            || include_qualification_functions
            || include_resource_info
            || include_subscriptions;

        // If we are including any graph data, we need to check if we need to override
        // configuration options.
        if include_any_graph_data {
            // Check for function-related overrides.
            include_action_functions = include_all || include_functions || include_action_functions;
            include_management_functions =
                include_all || include_functions || include_management_functions;
            include_qualification_functions =
                include_all || include_functions || include_qualification_functions;

            // Check for all other overrides.
            include_diff_status = include_all || include_diff_status;
            include_execution_history = include_all || include_execution_history;
            include_manages = include_all || include_manages;
            include_resource_info = include_all || include_resource_info;
            include_subscriptions = include_all || include_subscriptions;

            Some(Self {
                include_action_functions,
                include_diff_status,
                include_execution_history,
                include_management_functions,
                include_manages,
                include_qualification_functions,
                include_resource_info,
                include_subscriptions,
            })
        } else {
            None
        }
    }
}
