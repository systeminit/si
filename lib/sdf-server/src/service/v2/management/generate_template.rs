use std::collections::HashMap;

use axum::{
    Json,
    extract::{
        Host,
        OriginalUri,
        Path,
    },
};
use dal::{
    ChangeSet,
    ChangeSetId,
    Component,
    ComponentId,
    FuncId,
    SchemaVariant,
    SchemaVariantId,
    Ulid,
    WorkspacePk,
    WsEvent,
    attribute::attributes::Source,
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
use si_generate_template::{
    AssetSchema,
    AttributeSource,
    RunTemplateAttribute,
    RunTemplateComponent,
    RunTemplateMgmtFunc,
    Template,
};

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
    Path((_workspace_pk, change_set_id, _view_id)): Path<(WorkspacePk, ChangeSetId, ViewId)>,
    Json(request): Json<GenerateTemplateRequest>,
) -> ManagementApiResult<ForceChangeSetResponse<GenerateTemplateResponse>> {
    let mut ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;

    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;
    let mut has_aws = false;

    let mut template_components = Vec::new();
    let mut component_id_to_variable_name: HashMap<ComponentId, String> = HashMap::new();

    for component_id in &request.component_ids {
        let schema = Component::schema_for_component_id(&ctx, *component_id).await?;
        if schema.name.starts_with("AWS") {
            has_aws = true;
        }
        let mut attributes = Vec::new();
        let sources = Component::sources(&ctx, *component_id).await?;
        let si_name = Component::name_by_id(&ctx, *component_id).await?;
        let schema_variant_id = Component::schema_variant_id(&ctx, *component_id).await?;
        let all_props = SchemaVariant::all_props(&ctx, schema_variant_id).await?;
        for prop in all_props {
            let prop_path = prop.path(&ctx).await?;
            if prop_path.as_str() == "root\u{b}domain\u{b}extra\u{b}Region" {
                attributes.push(RunTemplateAttribute::new(
                    "/domain/extra/Region",
                    AttributeSource::InputSource,
                ));
            }
            if prop_path.as_str() == "root\u{b}secrets\u{b}AWS Credential" {
                attributes.push(RunTemplateAttribute::new(
                    "/secrets/AWS Credential",
                    AttributeSource::InputSource,
                ));
            }
        }

        for (path, source) in sources {
            let dest_path = path.path();
            // We only want domain properties, and we want to deal with Region directly
            if dest_path.starts_with("/domain/") && !dest_path.starts_with("/domain/extra/Region") {
                let value = match source {
                    Source::Value(json) => AttributeSource::value(json),
                    Source::Subscription {
                        component,
                        path,
                        keep_existing_subscriptions: _keep_existing_subs,
                        func,
                    } => {
                        let component_string: String = component.into();
                        AttributeSource::subscription(
                            component_string,
                            path,
                            func.map(|f| f.into()),
                            None,
                        )
                    }
                };

                attributes.push(RunTemplateAttribute::new(dest_path, value));
            }
        }
        let mut variable_name = si_name.clone();
        variable_name.push_str("Component");
        variable_name = sanitize_js_variable(&variable_name);
        component_id_to_variable_name.insert(*component_id, variable_name.clone());
        let template_component =
            RunTemplateComponent::new(variable_name, schema.name, si_name, attributes);
        template_components.push(template_component);
    }

    // Post Process the subscriptions to replace known IDs with variable name references
    for component in template_components.iter_mut() {
        for attribute in component.attributes.iter_mut() {
            if let AttributeSource::Subscription(sub_value) = &mut attribute.value {
                let c_ulid = Ulid::from_string(&sub_value.component)?;
                let c_id: ComponentId = ComponentId::from_raw_id(c_ulid.into());
                if let Some(variable_name) = component_id_to_variable_name.get(&c_id) {
                    sub_value.variable = Some(variable_name.clone());
                }
            }
        }
    }

    let asset_schema_template = AssetSchema::new(has_aws);
    let schema_variant_code = asset_schema_template.render()?;
    let mut mgmt_func_template = RunTemplateMgmtFunc::new(template_components);
    mgmt_func_template.sort_components_by_dependencies();
    let mgmt_func_code = mgmt_func_template.render()?;

    let new_variant = VariantAuthoringClient::create_schema_and_variant_from_code(
        &ctx,
        request.asset_name.to_owned(),
        Some(String::from(r#"## Template Components
Configure this component to have the Attributes you need to make your templated infrastructure dynamic. The only requirement is that you have a 'domain/Name Prefix' attribute.

Each invocation of the template will create new infrastructure; it's expected that you will have one instance of the template component for potentially many created infrastructures.

Changes in the template are never reflected in the output - components are generated once."#)),
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
        Some("Run Template".to_string()),
        Some("Run the template to create new components".to_string()),
    )
    .await?;

    let prototype_id = ManagementPrototype::list_ids_for_func_id(&ctx, func.id)
        .await?
        .pop()
        .ok_or(ManagementApiError::FuncMissingPrototype(func.id))?;

    FuncAuthoringClient::save_code(&ctx, func.id, mgmt_func_code).await?;

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
