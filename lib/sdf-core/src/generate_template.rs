use std::collections::HashMap;

use dal::{
    Component,
    ComponentError,
    ComponentId,
    FuncId,
    SchemaVariant,
    SchemaVariantError,
    Ulid,
    attribute::attributes::Source,
    func::authoring::{
        FuncAuthoringClient,
        FuncAuthoringError,
    },
    management::prototype::{
        ManagementPrototype,
        ManagementPrototypeError,
    },
    prop::PropError,
    schema::variant::authoring::{
        VariantAuthoringClient,
        VariantAuthoringError,
    },
};
use once_cell::sync::Lazy;
use regex::Regex;
use si_generate_template::{
    AssetSchema,
    AttributeSource,
    RunTemplateAttribute,
    RunTemplateComponent,
    RunTemplateMgmtFunc,
    Template,
};

pub async fn prepare_and_generate(
    ctx: &dal::DalContext,
    component_ids: Vec<ComponentId>,
    asset_name: String,
    func_name: String,
    category: String,
    color: String,
) -> Result<
    (
        SchemaVariant,
        dal::SchemaId,
        dal::Func,
        si_id::ManagementPrototypeId,
    ),
    GenerateTemplateError,
> {
    let mut has_aws = false;
    let mut template_components = Vec::new();
    let mut component_id_to_variable_name: HashMap<ComponentId, String> = HashMap::new();
    for component_id in &component_ids {
        let schema = Component::schema_for_component_id(ctx, *component_id).await?;
        if schema.name.starts_with("AWS") {
            has_aws = true;
        }
        let mut attributes = Vec::new();
        let sources = Component::sources(ctx, *component_id).await?;
        let si_name = Component::name_by_id(ctx, *component_id).await?;
        let schema_variant_id = Component::schema_variant_id(ctx, *component_id).await?;
        let all_props = SchemaVariant::all_props(ctx, schema_variant_id).await?;
        for prop in all_props {
            let prop_path = prop.path(ctx).await?;
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
                        _keep_existing_subscriptions,
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
        ctx,
        asset_name.to_owned(),
        Some(String::from(r#"## Template Components
Configure this component to have the Attributes you need to make your templated infrastructure dynamic. The only requirement is that you have a 'domain/Name Prefix' attribute.

Each invocation of the template will create new infrastructure; it's expected that you will have one instance of the template component for potentially many created infrastructures.

Changes in the template are never reflected in the output - components are generated once."#)),
        None,
        category,
        color,
        schema_variant_code,
    )
    .await?;
    let schema_id = SchemaVariant::schema_id(ctx, new_variant.id()).await?;
    let func = FuncAuthoringClient::create_new_management_func(
        ctx,
        Some(func_name.clone()),
        new_variant.id(),
    )
    .await?;
    FuncAuthoringClient::update_func(
        ctx,
        func.id,
        Some("Run Template".to_string()),
        Some("Run the template to create new components".to_string()),
    )
    .await?;
    let prototype_id = ManagementPrototype::list_ids_for_func_id(ctx, func.id)
        .await?
        .pop()
        .ok_or(GenerateTemplateError::FuncMissingPrototype(func.id))?;
    FuncAuthoringClient::save_code(ctx, func.id, mgmt_func_code).await?;
    Ok((new_variant, schema_id, func, prototype_id))
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

#[allow(missing_docs)]
#[remain::sorted]
#[derive(Debug, thiserror::Error)]
pub enum GenerateTemplateError {
    #[error("component error: {0}")]
    Component(#[from] ComponentError),
    #[error("func authoring error: {0}")]
    FuncAuthoring(#[from] FuncAuthoringError),
    #[error("generated mgmt func {0} has no prototype")]
    FuncMissingPrototype(FuncId),
    #[error("management prototype error: {0}")]
    ManagementPrototype(#[from] ManagementPrototypeError),
    #[error("prop path connot be calculated: {0}")]
    PropError(#[from] PropError),
    #[error("schema variant error: {0}")]
    SchemaVariant(#[from] SchemaVariantError),
    #[error("template error: {0}")]
    Template(#[from] si_generate_template::Error),
    #[error("translating string to ulid: {0} is not a valid ulid")]
    UlidDecode(#[from] ulid::DecodeError),
    #[error("variant authoring error: {0}")]
    VariantAuthoring(#[from] VariantAuthoringError),
}
