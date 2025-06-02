use axum::{
    Json,
    extract::Path,
};
use dal::{
    Func,
    func::FuncKind,
};
use serde::Serialize;
use serde_json::json;
use utoipa::ToSchema;

use super::{
    FuncV1RequestPath,
    FuncsError,
    FuncsResult,
};
use crate::extract::{
    PosthogEventTracker,
    change_set::ChangeSetDalContext,
};

#[utoipa::path(
    get,
    path = "/v1/w/{workspace_id}/change-sets/{change_set_id}/funcs/{func_id}",
    params(
        ("workspace_id" = String, Path, description = "Workspace identifier"),
        ("change_set_id" = String, Path, description = "Change Set identifier"),
        ("func_id" = String, Path, description = "Func identifier"),
    ),
    tag = "funcs",
    summary = "Get function details",
    responses(
        (status = 200, description = "Func retrieved successfully", body = GetFuncV1Response,
          example = json!([{
            "funcId": "01JP8A3S8VDQ1KRQWQRHB1ZEB2",
            "code": "async function main(input: Input): Promise < Output > {\n    if (!input.domain?.region) {\n        return {\n            result: \"failure\",\n            message: \"No Region Name to validate\",\n        };\n    }\n\n    const child = await siExec.waitUntilEnd(\"aws\", [\n        \"ec2\",\n        \"describe-regions\",\n        \"--region-names\",\n        input.domain?.region!,\n        \"--region\",\n        \"us-east-1\",\n    ]);\n\n    if (child.exitCode !== 0) {\n        console.error(child.stderr);\n        return {\n            result: \"failure\",\n            message: \"Error from API\"\n        }\n    }\n\n    const regionDetails = JSON.parse(child.stdout).Regions;\n    if (regionDetails.length === 0 || regionDetails.length > 1) {\n        return {\n            result: \"failure\",\n            message: \"Unable to find Region\"\n        }\n    }\n\n    if (regionDetails[0].OptInStatus === \"not-opted-in\") {\n        return {\n            result: \"failure\",\n            message: \"Region not-opted-in for use\"\n        }\n    }\n\n    return {\n        result: \"success\",\n        message: \"Region is available to use\",\n    };\n}",
            "name": "AWS Region Validator",
            "description": "Validates if an AWS region exists and is available for use",
            "displayName": "Validate Region",
            "kind": "Qualification",
            "isLocked": false,
            "link": "https://docs.aws.amazon.com/AWSEC2/latest/APIReference/API_DescribeRegions.html"
          }])
        ),
        (status = 401, description = "Unauthorized - Invalid or missing token"),
        (status = 404, description = "Func not found"),
        (status = 500, description = "Internal server error", body = crate::service::v1::common::ApiError)
    )
)]
pub async fn get_func(
    ChangeSetDalContext(ref ctx): ChangeSetDalContext,
    tracker: PosthogEventTracker,
    Path(FuncV1RequestPath { func_id }): Path<FuncV1RequestPath>,
) -> FuncsResult<Json<GetFuncV1Response>> {
    let func = Func::get_by_id_opt(ctx, func_id)
        .await?
        .ok_or(FuncsError::FuncNotFound(func_id))?;

    tracker.track(
        ctx,
        "api_get_func",
        json!({
            "func_id": func_id,
            "func_name": func.clone().name,
        }),
    );

    Ok(Json(GetFuncV1Response {
        name: func.clone().name,
        description: func.clone().description,
        display_name: func.clone().display_name,
        link: func.clone().link,
        is_locked: func.is_locked,
        kind: func.kind,
        code: func.code_plaintext()?.unwrap_or("".to_string()),
    }))
}

#[derive(Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct GetFuncV1Response {
    #[schema(value_type = String, example = "AWS Region Validator")]
    pub name: String,
    #[schema(value_type = String, example = "Validates if an AWS region exists and is available for use")]
    pub description: Option<String>,
    #[schema(value_type = String, example = "Validate Region")]
    pub display_name: Option<String>,
    #[schema(value_type = String, example = "Qualification")]
    pub kind: FuncKind,
    #[schema(value_type = bool, example = false)]
    pub is_locked: bool,
    #[schema(
        example = "async function main(input: Input): Promise < Output > {\n    if (!input.domain?.region) {\n        return {\n            result: \"failure\",\n            message: \"No Region Name to validate\",\n        };\n    }\n\n    const child = await siExec.waitUntilEnd(\"aws\", [\n        \"ec2\",\n        \"describe-regions\",\n        \"--region-names\",\n        input.domain?.region!,\n        \"--region\",\n        \"us-east-1\",\n    ]);\n\n    if (child.exitCode !== 0) {\n        console.error(child.stderr);\n        return {\n            result: \"failure\",\n            message: \"Error from API\"\n        }\n    }\n\n    const regionDetails = JSON.parse(child.stdout).Regions;\n    if (regionDetails.length === 0 || regionDetails.length > 1) {\n        return {\n            result: \"failure\",\n            message: \"Unable to find Region\"\n        }\n    }\n\n    if (regionDetails[0].OptInStatus === \"not-opted-in\") {\n        return {\n            result: \"failure\",\n            message: \"Region not-opted-in for use\"\n        }\n    }\n\n    return {\n        result: \"success\",\n        message: \"Region is available to use\",\n    };\n}"
    )]
    pub code: String,
    #[schema(value_type = String, example = "https://docs.aws.amazon.com/AWSEC2/latest/APIReference/API_DescribeRegions.html")]
    pub link: Option<String>,
}
