use axum::{
    extract::Path,
    response::Json,
};
use dal::{
    Schema,
    SchemaVariant,
    schema::variant::authoring::VariantAuthoringClient,
    slow_rt,
};
use sdf_extract::{
    PosthogEventTracker,
    change_set::ChangeSetDalContext,
};
use serde::{
    Deserialize,
    Serialize,
};
use serde_json::json;
use si_events::audit_log::AuditLogKind;
use utoipa::{
    self,
    ToSchema,
};

use super::{
    GetSchemaVariantV1Response,
    SchemaError,
    SchemaResult,
    SchemaVariantFunc,
    SchemaVariantV1RequestPath,
};

#[utoipa::path(
    put,
    path = "/v1/w/{workspace_id}/change-sets/{change_set_id}/schemas/{schema_id}/variant/{schema_variant_id}",
    params(
        ("workspace_id" = String, Path, description = "Workspace identifier"),
        ("change_set_id" = String, Path, description = "Change Set identifier"),
        ("schema_id" = String, Path, description = "Schema identifier"),
        ("schema_variant_id" = String, Path, description = "Schema variant identifier"),
    ),
    summary = "Update the schema variant and regenerate",
    tag = "schemas",
    request_body = UpdateSchemaVariantV1Request,
    responses(
        (status = 200, description = "Schema variant successfully updated", body = GetSchemaVariantV1Response),
        (status = 401, description = "Unauthorized - Invalid or missing token"),
        (status = 404, description = "Schema variant not found"),
        (status = 412, description = "Schema variant not found for schema"),
        (status = 500, description = "Internal server error", body = crate::service::v1::common::ApiError)
    )
)]
#[allow(deprecated)]
pub async fn update_schema_variant(
    ChangeSetDalContext(ref ctx): ChangeSetDalContext,
    tracker: PosthogEventTracker,
    Path(SchemaVariantV1RequestPath {
        schema_id,
        schema_variant_id,
    }): Path<SchemaVariantV1RequestPath>,
    payload: Result<Json<UpdateSchemaVariantV1Request>, axum::extract::rejection::JsonRejection>,
) -> SchemaResult<Json<GetSchemaVariantV1Response>> {
    let Json(payload) = payload?;

    if ctx.change_set_id() == ctx.get_workspace_default_change_set_id().await? {
        return Err(SchemaError::NotPermittedOnHead);
    }

    let schema_variants = Schema::list_schema_variant_ids(ctx, schema_id).await?;
    if !schema_variants.contains(&schema_variant_id) {
        return Err(SchemaError::SchemaVariantNotMemberOfSchema(
            schema_id,
            schema_variant_id,
        ));
    }

    let variant = SchemaVariant::get_by_id_opt(ctx, schema_variant_id)
        .await?
        .ok_or(SchemaError::SchemaVariantNotFound(schema_variant_id))?;

    if variant.is_locked() {
        return Err(SchemaError::LockedVariant(schema_variant_id));
    }

    let schema = Schema::get_by_id(ctx, schema_id).await?;

    let color = payload.color.unwrap_or_else(|| variant.color().to_string());

    VariantAuthoringClient::save_variant_content(
        ctx,
        schema_variant_id,
        schema.name(),
        payload.name,
        payload.category,
        payload.description,
        payload.link,
        color,
        variant.component_type(),
        Some(payload.code),
    )
    .await?;

    let updated_schema_variant_id =
        VariantAuthoringClient::regenerate_variant(ctx, schema_variant_id).await?;

    // Final check that we updated the correct variant!!
    if updated_schema_variant_id != schema_variant_id {
        return Err(SchemaError::LockedVariant(schema_variant_id));
    }

    tracker.track(
        ctx,
        "api_update_variant",
        json!({
            "schema_variant_id": schema_variant_id,
        }),
    );

    ctx.write_audit_log(
        AuditLogKind::RegenerateSchemaVariant { schema_variant_id },
        variant.display_name().to_string(),
    )
    .await?;

    ctx.commit().await?;

    let luminork_variant = slow_rt::spawn(
        dal_materialized_views::luminork::schema::variant::assemble(ctx.clone(), variant.id()),
    )?
    .await??;

    let variant_funcs: Vec<SchemaVariantFunc> = luminork_variant
        .variant_funcs
        .into_iter()
        .map(SchemaVariantFunc::from)
        .collect();

    let response = GetSchemaVariantV1Response {
        variant_id: luminork_variant.variant_id,
        display_name: luminork_variant.display_name,
        category: luminork_variant.category,
        color: luminork_variant.color,
        is_locked: luminork_variant.is_locked,
        description: luminork_variant.description,
        link: luminork_variant.link,
        asset_func_id: luminork_variant.asset_func_id,
        variant_func_ids: luminork_variant.variant_func_ids,
        variant_funcs,
        is_default_variant: luminork_variant.is_default_variant,
        domain_props: luminork_variant.domain_props.map(Into::into),
    };

    Ok(Json(response))
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSchemaVariantV1Request {
    #[schema(value_type = String, example = "AWS Region Validator")]
    pub name: String,
    #[schema(value_type = Option<String>, example = "Validates if an AWS region exists and is available for use")]
    pub description: Option<String>,
    #[schema(value_type = Option<String>, example = "https://docs.aws.amazon.com/AWSEC2/latest/APIReference/API_DescribeRegions.html")]
    pub link: Option<String>,
    #[schema(value_type = String, example = "AWS::EC2")]
    pub category: String,
    #[schema(value_type = Option<String>, example = "#FF5733")]
    pub color: Option<String>,
    #[schema(
        example = "async function main(input: Input): Promise < Output > {\n    if (!input.domain?.region) {\n        return {\n            result: \"failure\",\n            message: \"No Region Name to validate\",\n        };\n    }\n\n    const child = await siExec.waitUntilEnd(\"aws\", [\n        \"ec2\",\n        \"describe-regions\",\n        \"--region-names\",\n        input.domain?.region!,\n        \"--region\",\n        \"us-east-1\",\n    ]);\n\n    if (child.exitCode !== 0) {\n        console.error(child.stderr);\n        return {\n            result: \"failure\",\n            message: \"Error from API\"\n        }\n    }\n\n    const regionDetails = JSON.parse(child.stdout).Regions;\n    if (regionDetails.length === 0 || regionDetails.length > 1) {\n        return {\n            result: \"failure\",\n            message: \"Unable to find Region\"\n        }\n    }\n\n    if (regionDetails[0].OptInStatus === \"not-opted-in\") {\n        return {\n            result: \"failure\",\n            message: \"Region not-opted-in for use\"\n        }\n    }\n\n    return {\n        result: \"success\",\n        message: \"Region is available to use\",\n    };\n}"
    )]
    pub code: String,
}
