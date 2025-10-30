use base64::{
    Engine,
    engine::general_purpose,
};
use si_id::SchemaId;
use telemetry::prelude::*;

use super::{
    FuncAuthoringError,
    FuncAuthoringResult,
};
use crate::{
    Component,
    DalContext,
    Func,
    FuncBackendKind,
    FuncBackendResponseType,
    Schema,
    SchemaVariant,
    SchemaVariantId,
    action::prototype::{
        ActionKind,
        ActionPrototype,
    },
    func::{
        binding::{
            AttributeArgumentBinding,
            AttributeFuncDestination,
            EventualParent,
            action::ActionBinding,
            attribute::AttributeBinding,
            authentication::AuthBinding,
            leaf::LeafBinding,
            management::ManagementBinding,
        },
        leaf::{
            LeafInputLocation,
            LeafKind,
        },
    },
    generate_name,
    management::prototype::ManagementPrototypeParent,
};

static DEFAULT_CODE_HANDLER: &str = "main";
static DEFAULT_ATTRIBUTE_CODE: &str = include_str!("data/defaults/attribute.ts");
static DEFAULT_CODE_GENERATION_CODE: &str = include_str!("data/defaults/code_generation.ts");
static DEFAULT_QUALIFICATION_CODE: &str = include_str!("data/defaults/qualification.ts");
static DEFAULT_ACTION_CODE: &str = include_str!("data/defaults/action.ts");
static DEFAULT_AUTHENTICATION_CODE: &str = include_str!("data/defaults/authentication.ts");
static DEFAULT_MGMT_CODE: &str = include_str!("data/defaults/management.ts");

#[allow(dead_code)]
static DEFAULT_VALIDATION_CODE: &str = include_str!("data/defaults/validation.ts");

#[instrument(
    name = "func.authoring.create_func.create.management",
    level = "debug",
    skip(ctx)
)]
pub(crate) async fn create_management_func(
    ctx: &DalContext,
    name: Option<String>,
    parent: ManagementPrototypeParent,
) -> FuncAuthoringResult<Func> {
    let name = name.unwrap_or(generate_name());

    match &parent {
        // An overlay can have any name
        ManagementPrototypeParent::Schemas(schema_ids) => {
            for schema_id in schema_ids {
                fail_if_func_name_already_used_on_schema(ctx, &name, *schema_id).await?;
            }
        }
        ManagementPrototypeParent::SchemaVariant(variant_id) => {
            fail_if_func_name_already_used_on_variant(ctx, &name, *variant_id).await?
        }
    }

    let func = create_func_stub(
        ctx,
        name.clone(),
        FuncBackendKind::Management,
        FuncBackendResponseType::Management,
        DEFAULT_MGMT_CODE,
        DEFAULT_CODE_HANDLER,
        false,
    )
    .await?;

    match parent {
        ManagementPrototypeParent::Schemas(schema_ids) => {
            ManagementBinding::create_management_binding(ctx, func.id, Some(schema_ids), None)
                .await?;
        }
        ManagementPrototypeParent::SchemaVariant(schema_variant_id) => {
            ManagementBinding::create_management_binding(
                ctx,
                func.id,
                None,
                Some(schema_variant_id),
            )
            .await?;
        }
    }

    Ok(func)
}

#[instrument(
    name = "func.authoring.create_func.create.action",
    level = "debug",
    skip(ctx)
)]
pub(crate) async fn create_action_func_overlay(
    ctx: &DalContext,
    name: Option<String>,
    action_kind: ActionKind,
    schema_id: SchemaId,
) -> FuncAuthoringResult<Func> {
    let name = name.unwrap_or(generate_name());
    fail_if_func_name_already_used_on_schema(ctx, &name, schema_id).await?;
    // need to see if there's already an action func of this particular kind for the schema variant
    // before doing anything else

    let func = create_func_stub(
        ctx,
        name,
        FuncBackendKind::JsAction,
        FuncBackendResponseType::Action,
        DEFAULT_ACTION_CODE,
        DEFAULT_CODE_HANDLER,
        false,
    )
    .await?;

    if let Err(err) =
        ActionBinding::create_action_binding_overlay(ctx, func.id, action_kind, schema_id).await
    {
        if let crate::func::binding::FuncBindingError::ActionKindAlreadyExistsForSchema(_, _) = &err
        {
            Func::delete_by_id(ctx, func.id).await?;
        }
        return Err(err)?;
    }

    Ok(func)
}

#[instrument(
    name = "func.authoring.create_func.create.action",
    level = "debug",
    skip(ctx)
)]
pub(crate) async fn create_action_func(
    ctx: &DalContext,
    name: Option<String>,
    action_kind: ActionKind,
    schema_variant_id: SchemaVariantId,
) -> FuncAuthoringResult<Func> {
    let name = name.unwrap_or(generate_name());
    fail_if_func_name_already_used_on_variant(ctx, &name, schema_variant_id).await?;

    // need to see if there's already an action func of this particular kind for the schema variant
    // before doing anything else

    let existing_actions_for_variant = ActionPrototype::for_variant(ctx, schema_variant_id).await?;
    for action in existing_actions_for_variant {
        if action.kind == action_kind && action_kind != ActionKind::Manual {
            return Err(FuncAuthoringError::ActionKindAlreadyExists(
                action_kind,
                schema_variant_id,
            ));
        }
    }

    let func = create_func_stub(
        ctx,
        name,
        FuncBackendKind::JsAction,
        FuncBackendResponseType::Action,
        DEFAULT_ACTION_CODE,
        DEFAULT_CODE_HANDLER,
        false,
    )
    .await?;

    ActionBinding::create_action_binding(ctx, func.id, action_kind, schema_variant_id).await?;

    Ok(func)
}

#[instrument(
    name = "func.authoring.create_func.create.leaf",
    level = "debug",
    skip(ctx)
)]
pub(crate) async fn create_leaf_func(
    ctx: &DalContext,
    name: Option<String>,
    leaf_kind: LeafKind,
    eventual_parent: EventualParent,
    inputs: &[LeafInputLocation],
) -> FuncAuthoringResult<Func> {
    let name = name.unwrap_or(generate_name());
    fail_if_name_already_used_on_eventual_parent(ctx, &name, &eventual_parent).await?;

    let (code, handler, backend_kind, backend_response_type) = match leaf_kind {
        LeafKind::CodeGeneration => (
            DEFAULT_CODE_GENERATION_CODE,
            DEFAULT_CODE_HANDLER,
            FuncBackendKind::JsAttribute,
            FuncBackendResponseType::CodeGeneration,
        ),
        LeafKind::Qualification => (
            DEFAULT_QUALIFICATION_CODE,
            DEFAULT_CODE_HANDLER,
            FuncBackendKind::JsAttribute,
            FuncBackendResponseType::Qualification,
        ),
    };
    let func = create_func_stub(
        ctx,
        name,
        backend_kind,
        backend_response_type,
        code,
        handler,
        false,
    )
    .await?;
    LeafBinding::create_leaf_func_binding(ctx, func.id, eventual_parent, leaf_kind, inputs).await?;
    Ok(func)
}

#[instrument(
    name = "func.authoring.create_func.create.attribute",
    level = "debug",
    skip(ctx)
)]
pub(crate) async fn create_attribute_func(
    ctx: &DalContext,
    name: Option<String>,
    eventual_parent: Option<EventualParent>,
    output_location: Option<AttributeFuncDestination>,
    argument_bindings: Vec<AttributeArgumentBinding>,
    is_transformation: bool,
) -> FuncAuthoringResult<Func> {
    let name = name.unwrap_or(generate_name());

    if let Some(eventual_parent) = &eventual_parent {
        fail_if_name_already_used_on_eventual_parent(ctx, &name, eventual_parent).await?;
    }

    let (code, handler, backend_kind, backend_response_type) = (
        DEFAULT_ATTRIBUTE_CODE,
        DEFAULT_CODE_HANDLER,
        FuncBackendKind::JsAttribute,
        FuncBackendResponseType::Unset,
    );

    let func = create_func_stub(
        ctx,
        name,
        backend_kind,
        backend_response_type,
        code,
        handler,
        is_transformation,
    )
    .await?;

    if let Some(output_location) = output_location {
        AttributeBinding::upsert_attribute_binding(
            ctx,
            func.id,
            eventual_parent,
            output_location,
            argument_bindings,
        )
        .await?;
    }

    Ok(func)
}

#[instrument(
    name = "func.authoring.create_func.create.authentication",
    level = "debug",
    skip(ctx)
)]
pub(crate) async fn create_authentication_func(
    ctx: &DalContext,
    name: Option<String>,
    schema_variant_id: SchemaVariantId,
) -> FuncAuthoringResult<Func> {
    let name = name.unwrap_or(generate_name());
    fail_if_func_name_already_used_on_variant(ctx, &name, schema_variant_id).await?;

    let func = create_func_stub(
        ctx,
        name,
        FuncBackendKind::JsAuthentication,
        FuncBackendResponseType::Void,
        DEFAULT_AUTHENTICATION_CODE,
        DEFAULT_CODE_HANDLER,
        false,
    )
    .await?;

    AuthBinding::create_auth_binding(ctx, func.id, schema_variant_id).await?;
    Ok(func)
}

async fn create_func_stub(
    ctx: &DalContext,
    name: String,
    backend_kind: FuncBackendKind,
    backend_response_type: FuncBackendResponseType,
    code: &str,
    handler: &str,
    is_transformation: bool,
) -> FuncAuthoringResult<Func> {
    let code_base64 = general_purpose::STANDARD_NO_PAD.encode(code);

    let func = Func::new(
        ctx,
        name,
        None::<String>,
        None::<String>,
        None::<String>,
        false,
        false,
        backend_kind,
        backend_response_type,
        Some(handler),
        Some(code_base64),
        is_transformation,
    )
    .await?;

    Ok(func)
}

async fn fail_if_func_name_already_used_on_variant(
    ctx: &DalContext,
    name: impl AsRef<str>,
    schema_variant_id: SchemaVariantId,
) -> FuncAuthoringResult<()> {
    if SchemaVariant::all_funcs_without_intrinsics(ctx, schema_variant_id)
        .await?
        .iter()
        .any(|f| f.name == name.as_ref())
    {
        return Err(FuncAuthoringError::FuncNameExistsOnVariant(
            name.as_ref().to_string(),
            schema_variant_id,
        ));
    }

    Ok(())
}

async fn fail_if_func_name_already_used_on_schema(
    ctx: &DalContext,
    name: impl AsRef<str>,
    schema_id: SchemaId,
) -> FuncAuthoringResult<()> {
    let func_ids = Schema::all_overlay_func_ids(ctx, schema_id).await?;

    for func_id in func_ids {
        let func = Func::get_by_id(ctx, func_id).await?;

        if func.name == name.as_ref() {
            return Err(FuncAuthoringError::FuncNameExistsOnSchema(
                name.as_ref().to_string(),
                schema_id,
            ));
        }
    }

    Ok(())
}

async fn fail_if_name_already_used_on_eventual_parent(
    ctx: &DalContext,
    name: impl AsRef<str>,
    eventual_parent: &EventualParent,
) -> FuncAuthoringResult<()> {
    // Check if name is already used on the same variant, if applicable
    match eventual_parent {
        EventualParent::SchemaVariant(variant_id) => {
            fail_if_func_name_already_used_on_variant(ctx, &name, *variant_id).await?
        }
        EventualParent::Component(component_id) => {
            let variant_id = Component::schema_variant_id(ctx, *component_id).await?;
            fail_if_func_name_already_used_on_variant(ctx, &name, variant_id).await?;
        }

        EventualParent::Schemas(schema_ids) => {
            for schema_id in schema_ids {
                fail_if_func_name_already_used_on_schema(ctx, &name, *schema_id).await?;
            }
        }
    }

    Ok(())
}
