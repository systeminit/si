use axum::{
    Json,
    extract::{
        Host,
        OriginalUri,
        Path,
    },
};
use convert_case::{
    Case,
    Casing,
};
use dal::{
    ChangeSet,
    ChangeSetId,
    ComponentId,
    FuncId,
    SchemaVariant,
    SchemaVariantId,
    WorkspacePk,
    WsEvent,
    diagram::view::ViewId,
    func::authoring::FuncAuthoringClient,
    management::prototype::ManagementPrototype,
    schema::variant::authoring::VariantAuthoringClient,
};
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{
    Deserialize,
    Serialize,
};
use si_events::audit_log::AuditLogKind;

use super::{
    ManagementApiError,
    ManagementApiResult,
    track,
};
use crate::{
    extract::{
        HandlerContext,
        PosthogClient,
    },
    service::{
        force_change_set_response::ForceChangeSetResponse,
        v2::AccessBuilder,
    },
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateTemplateRequest {
    component_ids: Vec<ComponentId>,
    asset_name: String,
    func_name: String,
    category: String,
    color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateTemplateResponse {
    schema_variant_id: SchemaVariantId,
    func_id: FuncId,
}

pub async fn generate_template(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    Path((_workspace_pk, change_set_id, view_id)): Path<(WorkspacePk, ChangeSetId, ViewId)>,
    Json(request): Json<GenerateTemplateRequest>,
) -> ManagementApiResult<ForceChangeSetResponse<GenerateTemplateResponse>> {
    let mut ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;

    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    // The default schema code
    let schema_variant_code = r#"function main() {
    const asset = new AssetBuilder();

    // Add your template variables to this object with the .addChild function.
    const templateProps = new PropBuilder()
        .setName("Values")
        .setKind("object")
        .build();

    // Auto-generated below here
    const template = new PropBuilder()
        .setName("Template")
        .setKind("object")
        .addChild(templateProps)
        .build();
    asset.addProp(template);

    return asset.build();
}
"#
    .to_string();

    let new_variant = VariantAuthoringClient::create_schema_and_variant_from_code(
        &ctx,
        request.asset_name.to_owned(),
        None,
        None,
        request.category,
        request.color,
        schema_variant_code,
    )
    .await?;

    let schema_id = SchemaVariant::schema_id(&ctx, new_variant.id()).await?;

    let func = FuncAuthoringClient::create_new_management_func(
        &ctx,
        Some(request.func_name.clone()),
        new_variant.id(),
    )
    .await?;
    FuncAuthoringClient::update_func(
        &ctx,
        func.id,
        Some("Component Sync".to_string()),
        Some("Sync the components specified in the template with the diagram".to_string()),
    )
    .await?;

    let prototype_id = ManagementPrototype::list_ids_for_func_id(&ctx, func.id)
        .await?
        .pop()
        .ok_or(ManagementApiError::FuncMissingPrototype(func.id))?;

    let create_operations =
        dal::management::generator::generate_template(&ctx, view_id, &request.component_ids)
            .await?;

    let mut component_sync_code = r#"async function main({
    currentView,
    thisComponent,
    components
}: Input): Promise < Output > {
    const templateName = _.get(
        thisComponent,
        ["properties", "si", "name"],
        "unknown",
    );
    const vars = _.get(thisComponent, [
      "properties",
      "domain",
      "Template",
      "Values",
    ]);
    const specs: Output["ops"]["create"][string][] = [];
"#
    .to_string();

    for (name, component_def) in create_operations {
        let mut variable_name: String = name.to_case(Case::Camel);
        variable_name.push_str("Spec");
        variable_name = sanitize_js_variable(&variable_name);
        let spec_body = serde_json::to_string_pretty(&component_def)?;
        let component_code = format!(
            r#"
    const {variable_name}: Output["ops"]["create"][string] = {spec_body};
    specs.push({variable_name});
"#
        );
        component_sync_code.push_str(&component_code);
    }
    component_sync_code.push_str(
        r#"

    // Check for duplicate si names in the abscene of component idempotency keys
    const seenNames = new Set<string>();
    for (const spec of specs) {
        const name = _.get(spec, ["properties", "si", "name"]);
        if (seenNames.has(name)) {
            throw new Error(`Duplicate properties.si.name found: "${name}", please regenerate the template after fixing the duplicate names or modify the id references in the management function`);
        }
        seenNames.add(name);
    }

    return template.converge(currentView, thisComponent, components, specs);
}
"#,
    );

    FuncAuthoringClient::save_code(&ctx, func.id, component_sync_code).await?;

    let schema_variant_id = new_variant.id();
    WsEvent::schema_variant_created(&ctx, schema_id, new_variant.clone())
        .await?
        .publish_on_commit(&ctx)
        .await?;

    WsEvent::template_generated(
        &ctx,
        schema_id,
        schema_variant_id,
        func.id,
        request.asset_name.clone(),
    )
    .await?
    .publish_on_commit(&ctx)
    .await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        &host_name,
        "generate_template",
        serde_json::json!({
            "generated_schema_variant_id": new_variant.id,
            "generated_prototype_id": prototype_id,
            "generated_func_id": func.id,
        }),
    );

    ctx.write_audit_log(
        AuditLogKind::GenerateTemplate {
            schema_variant_id: new_variant.id,
            management_prototype_id: prototype_id,
            func_id: func.id,
            func_name: request.func_name,
            asset_name: request.asset_name.to_owned(),
        },
        request.asset_name,
    )
    .await?;

    ctx.commit().await?;

    Ok(ForceChangeSetResponse::new(
        force_change_set_id,
        GenerateTemplateResponse {
            schema_variant_id: new_variant.id,
            func_id: func.id,
        },
    ))
}

fn sanitize_js_variable(input: &str) -> String {
    static RE: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"[^a-zA-Z0-9_$]").expect("Regex failed to compile; bug!"));
    let mut sanitized = RE.replace_all(input, "").to_string();

    if let Some(first_char) = sanitized.chars().next() {
        if !first_char.is_ascii_alphabetic() && first_char != '_' && first_char != '$' {
            sanitized.insert(0, '_');
        }
    }

    sanitized
}
