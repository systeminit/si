use axum::{
    extract::Path,
    response::Json,
};
use dal::{
    AttributePrototype,
    AttributePrototypeId,
    ComponentId,
    FuncId,
    PropId,
    SchemaVariant,
    cached_module::CachedModule,
    func::{
        argument::{
            FuncArgument,
            FuncArgumentKind,
        },
        authoring::FuncAuthoringClient,
        binding::{
            AttributeArgumentBinding,
            AttributeFuncArgumentSource,
            AttributeFuncDestination,
            EventualParent,
        },
    },
};
use sdf_extract::{
    PosthogEventTracker,
    change_set::ChangeSetDalContext,
};
use serde::{
    Deserialize,
    Serialize,
};
use si_events::audit_log::AuditLogKind;
use utoipa::{
    self,
    ToSchema,
};

use super::{
    SchemaError,
    SchemaResult,
    SchemaVariantV1RequestPath,
};

#[utoipa::path(
    post,
    path = "/v1/w/{workspace_id}/change-sets/{change_set_id}/schemas/{schema_id}/variant/{schema_variant_id}/funcs/attribute",
    params(
        ("workspace_id" = String, Path, description = "Workspace identifier"),
        ("change_set_id" = String, Path, description = "Change Set identifier"),
        ("schema_id" = String, Path, description = "Schema identifier"),
        ("schema_variant_id" = String, Path, description = "Schema variant identifier"),
    ),
    summary = "Create an attribute function and attach to a schema variant",
    tag = "schemas",
    request_body = CreateVariantAttributeFuncV1Request,
    responses(
        (status = 200, description = "Attribute function successfully created and attached to the variant", body = CreateVariantAttributeFuncV1Response),
        (status = 401, description = "Unauthorized - Invalid or missing token"),
        (status = 404, description = "Schema variant not found"),
        (status = 412, description = "Schema variant not found for schema"),
        (status = 500, description = "Internal server error", body = crate::service::v1::common::ApiError)
    )
)]
pub async fn create_variant_attribute(
    ChangeSetDalContext(ref ctx): ChangeSetDalContext,
    tracker: PosthogEventTracker,
    Path(SchemaVariantV1RequestPath {
        schema_id,
        schema_variant_id,
    }): Path<SchemaVariantV1RequestPath>,
    payload: Result<
        Json<CreateVariantAttributeFuncV1Request>,
        axum::extract::rejection::JsonRejection,
    >,
) -> SchemaResult<Json<CreateVariantAttributeFuncV1Response>> {
    let Json(payload) = payload?;

    if ctx.change_set_id() == ctx.get_workspace_default_change_set_id().await? {
        return Err(SchemaError::NotPermittedOnHead);
    }

    let schema_id_for_variant = SchemaVariant::schema_id(ctx, schema_variant_id).await?;
    if schema_id != schema_id_for_variant {
        return Err(SchemaError::SchemaVariantNotMemberOfSchema(
            schema_id,
            schema_variant_id,
        ));
    }

    let is_builtin = CachedModule::find_latest_for_schema_id(ctx, schema_id)
        .await?
        .is_some();

    if !is_builtin {
        let schema_variant = SchemaVariant::get_by_id(ctx, schema_variant_id).await?;
        if schema_variant.is_locked() {
            return Err(SchemaError::LockedVariant(schema_variant_id));
        }
    }

    // Set output location to the prop
    let output_location = AttributeFuncDestination::Prop(payload.prop_id);

    let eventual_parent = if let Some(component_id) = payload.component_id {
        Some(EventualParent::Component(component_id))
    } else if is_builtin {
        Some(EventualParent::Schemas(vec![schema_id]))
    } else {
        Some(EventualParent::SchemaVariant(schema_variant_id))
    };

    // Create the func first (without argument bindings)
    let func = FuncAuthoringClient::create_new_attribute_func(
        ctx,
        Some(payload.name.clone()),
        eventual_parent.clone(),
        output_location,
        vec![], // Empty argument bindings for now
    )
    .await?;

    // Get the attribute prototype ID
    let attribute_prototype_ids = AttributePrototype::list_ids_for_func_id(ctx, func.id).await?;
    let attribute_prototype_id = attribute_prototype_ids
        .first()
        .copied()
        .ok_or_else(|| SchemaError::Validation("No attribute prototype created".to_string()))?;

    // Create func arguments and assemble bindings
    let argument_bindings =
        assemble_argument_bindings(ctx, func.id, payload.argument_bindings).await?;

    // Update the attribute binding with the new arguments
    if !argument_bindings.is_empty() {
        dal::func::binding::attribute::AttributeBinding::update_attribute_binding_arguments(
            ctx,
            attribute_prototype_id,
            argument_bindings,
        )
        .await?;
    }

    FuncAuthoringClient::update_func(
        ctx,
        func.id,
        payload.display_name.clone(),
        payload.description.clone(),
    )
    .await?;

    FuncAuthoringClient::save_code(ctx, func.id, payload.code).await?;

    ctx.write_audit_log(
        AuditLogKind::CreateFunc {
            func_display_name: func.display_name.clone(),
            func_kind: func.kind.into(),
        },
        func.name.clone(),
    )
    .await?;

    let subject_name = if let Some(component_id) = payload.component_id {
        dal::Component::get_by_id(ctx, component_id)
            .await?
            .name(ctx)
            .await?
            .to_owned()
    } else {
        SchemaVariant::get_by_id(ctx, schema_variant_id)
            .await?
            .display_name()
            .to_owned()
    };

    // Get destination prop name
    let destination_name = dal::Prop::get_by_id(ctx, payload.prop_id).await?.name;

    ctx.write_audit_log(
        AuditLogKind::AttachAttributeFunc {
            func_id: func.id,
            func_display_name: func.display_name.clone(),
            schema_variant_id: Some(schema_variant_id),
            component_id: payload.component_id,
            subject_name,
            prop_id: Some(payload.prop_id),
            output_socket_id: None,
            destination_name,
        },
        func.name.clone(),
    )
    .await?;

    tracker.track(
        ctx,
        "api_create_attribute_func",
        serde_json::json!({
            "func_id": func.id,
            "func_name": func.name.to_owned(),
            "schema_variant_id": schema_variant_id,
        }),
    );

    ctx.commit().await?;

    Ok(Json(CreateVariantAttributeFuncV1Response {
        func_id: func.id,
        attribute_prototype_id,
    }))
}

#[allow(clippy::result_large_err)]
async fn assemble_argument_bindings(
    ctx: &dal::DalContext,
    func_id: FuncId,
    arg_requests: Vec<AttributeArgumentBindingRequest>,
) -> SchemaResult<Vec<AttributeArgumentBinding>> {
    let mut argument_bindings = vec![];

    for arg_req in arg_requests {
        // Create the FuncArgument first
        let func_argument = FuncArgument::new(
            ctx,
            arg_req.name,
            arg_req.kind,
            arg_req.element_kind,
            func_id,
        )
        .await?;

        // Determine the input location
        let input_location = if let Some(prop_id) = arg_req.prop_id {
            AttributeFuncArgumentSource::Prop(prop_id)
        } else if let Some(static_value) = arg_req.static_value {
            AttributeFuncArgumentSource::StaticArgument(static_value)
        } else {
            return Err(SchemaError::MissingInputLocationForAttributeFunc);
        };

        // Create the binding with the newly created func_argument_id
        argument_bindings.push(AttributeArgumentBinding {
            func_argument_id: func_argument.id,
            attribute_prototype_argument_id: None,
            attribute_func_input_location: input_location,
        });
    }

    Ok(argument_bindings)
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateVariantAttributeFuncV1Request {
    /// Unique name for the function (e.g., "awsEC2SetInstanceType")
    #[schema(value_type = String, example = "awsEC2SetInstanceType")]
    pub name: String,

    /// Human-readable display name
    #[schema(value_type = Option<String>, example = "Set Instance Type")]
    pub display_name: Option<String>,

    /// Description of what the function does
    #[schema(value_type = Option<String>, example = "Sets the instance type for an EC2 Instance")]
    pub description: Option<String>,

    /// TypeScript code for the function. Should export a main function that takes arguments and returns a value.
    #[schema(
        value_type = String,
        example = "async function main(instanceType: Input): Promise<Output> { return instanceType; }"
    )]
    pub code: String,

    /// Prop ID where the function output will be written (required)
    #[schema(value_type = String, example = "01H9ZQD35JPMBGHH69BT0Q79VZ")]
    pub prop_id: PropId,

    /// Optional component ID for component-level bindings. If not provided, creates a schema variant-level binding.
    #[schema(value_type = Option<String>, example = "01H9ZQD35JPMBGHH69BT0Q79VZ")]
    pub component_id: Option<ComponentId>,

    /// Function arguments with their bindings (input sources). Each argument defines its type and where its value comes from.
    #[schema(value_type = Vec<AttributeArgumentBindingRequest>)]
    pub argument_bindings: Vec<AttributeArgumentBindingRequest>,
}

#[derive(Deserialize, Serialize, Debug, ToSchema, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AttributeArgumentBindingRequest {
    /// Name of the function argument (e.g., "instanceType", "region", "tags")
    #[schema(value_type = String, example = "instanceType")]
    pub name: String,

    /// Type of the argument. Valid values: "Any", "Array", "Boolean", "Float", "Integer", "Json", "Map", "Object", "String". Use 'Array' with element_kind for typed arrays.
    #[schema(value_type = String, example = "String")]
    pub kind: FuncArgumentKind,

    /// Element type for Array arguments. Required when kind is 'Array'. Specifies the type of array elements.
    #[schema(value_type = Option<String>, example = "String")]
    pub element_kind: Option<FuncArgumentKind>,

    /// Prop ID to bind this argument to. Either prop_id or static_value must be provided.
    #[schema(value_type = Option<String>, example = "01H9ZQD35JPMBGHH69BT0Q79VZ")]
    pub prop_id: Option<PropId>,

    /// Static value for this argument. Can be string, number, boolean, array, or object. Either prop_id or static_value must be provided.
    #[schema(value_type = Option<serde_json::Value>, example = json!("t2.micro"))]
    pub static_value: Option<serde_json::Value>,
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateVariantAttributeFuncV1Response {
    #[schema(value_type = String, example = "01H9ZQD35JPMBGHH69BT0Q79VZ")]
    pub func_id: FuncId,
    #[schema(value_type = String, example = "01H9ZQD35JPMBGHH69BT0Q79VZ")]
    pub attribute_prototype_id: AttributePrototypeId,
}
