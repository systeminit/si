use base64::{
    Engine,
    engine::general_purpose,
};
use dal::{
    DalContext,
    Func,
    Schema,
    action::prototype::ActionKind,
    func::{
        authoring::FuncAuthoringClient,
        binding::FuncBinding,
    },
    management::prototype::ManagementPrototype,
};
use dal_test::{
    Result,
    helpers::schema::{
        create_overlay_action_func,
        create_overlay_management_func,
    },
    test,
};
use pretty_assertions_sorted::assert_eq;

/// Test that has_variant_bindings returns false for action overlays
#[test]
async fn has_variant_bindings_action_overlay(ctx: &mut DalContext) -> Result<()> {
    let schema = Schema::get_by_name(ctx, "swifty").await?;

    let create_action_code = r#"async function main() {
        return { payload: { "status": "created" }, status: "ok" };
    }"#;

    // Create an action overlay
    let func_id = create_overlay_action_func(
        ctx,
        schema.id(),
        "test:overlayAction",
        create_action_code,
        ActionKind::Create,
    )
    .await?;

    // Check that has_variant_bindings returns false (it's schema-level, not variant-level)
    let has_variant_bindings = FuncBinding::has_variant_bindings(ctx, func_id).await?;
    assert!(
        !has_variant_bindings,
        "Action overlay function should not have variant bindings"
    );

    Ok(())
}

/// Test that has_variant_bindings returns false for management overlays
#[test]
async fn has_variant_bindings_management_overlay(ctx: &mut DalContext) -> Result<()> {
    let schema = Schema::get_by_name(ctx, "swifty").await?;

    let mgmt_code = r#"async function main({ thisComponent, components }) {
        return {
            status: "ok",
            ops: { update: { self: { attributes: { "/domain/name": "updated" } } } }
        };
    }"#;

    // Create a management overlay
    let func_id =
        create_overlay_management_func(ctx, schema.id(), "test:overlayMgmt", mgmt_code).await?;

    // Check that has_variant_bindings returns false (it's schema-level, not variant-level)
    let has_variant_bindings = FuncBinding::has_variant_bindings(ctx, func_id).await?;
    assert!(
        !has_variant_bindings,
        "Management overlay function should not have variant bindings"
    );

    Ok(())
}

/// Test that has_variant_bindings returns true for variant-level functions
#[test]
async fn has_variant_bindings_variant_level(ctx: &mut DalContext) -> Result<()> {
    // Find a variant-level function
    let func_id = Func::find_id_by_name(ctx, "test:createActionStarfield")
        .await?
        .expect("could not find test:createActionStarfield");

    // Check that has_variant_bindings returns true for variant-level func
    let has_variant_bindings = FuncBinding::has_variant_bindings(ctx, func_id).await?;
    assert!(
        has_variant_bindings,
        "Variant-level function should have variant bindings"
    );

    Ok(())
}

/// Test that locked overlay functions can be updated via save_code
#[test]
async fn update_locked_overlay_action_save_code(ctx: &mut DalContext) -> Result<()> {
    let schema = Schema::get_by_name(ctx, "swifty").await?;

    let original_code = r#"async function main() {
        return { payload: { "status": "original" }, status: "ok" };
    }"#;

    let updated_code = r#"async function main() {
        return { payload: { "status": "updated" }, status: "ok" };
    }"#;

    // Create an action overlay
    let func_id = create_overlay_action_func(
        ctx,
        schema.id(),
        "test:lockedOverlayAction",
        original_code,
        ActionKind::Create,
    )
    .await?;

    // Lock the function to test updating locked overlays
    let func = Func::get_by_id(ctx, func_id).await?;
    func.lock(ctx).await?;

    // Verify the function is now locked
    let func = Func::get_by_id(ctx, func_id).await?;
    assert!(func.is_locked, "Overlay function should be locked");

    // Verify it does NOT have variant bindings (only schema-level)
    let has_variant_bindings = FuncBinding::has_variant_bindings(ctx, func_id).await?;
    assert!(
        !has_variant_bindings,
        "Function should not have variant bindings"
    );

    // Update the code - this should succeed even though the func is locked
    let result = FuncAuthoringClient::save_code(ctx, func_id, updated_code).await;
    if let Err(e) = &result {
        eprintln!("Error from save_code: {e:?}");
        eprintln!("has_variant_bindings returned: {has_variant_bindings}");
    }
    assert!(
        result.is_ok(),
        "save_code should succeed for locked overlay function: {:?}",
        result.err()
    );

    // Verify the code was actually updated
    let func = Func::get_by_id(ctx, func_id).await?;
    let code_base64 = func.code_base64.expect("should have code");
    let decoded_bytes = general_purpose::STANDARD_NO_PAD
        .decode(&code_base64)
        .expect("should decode base64");
    let decoded_code = String::from_utf8(decoded_bytes).expect("should be valid UTF-8");
    assert!(
        decoded_code.contains("updated"),
        "Code should be updated to new version"
    );

    Ok(())
}

/// Test that locked overlay functions can be updated via update_func
#[test]
async fn update_locked_overlay_management_metadata(ctx: &mut DalContext) -> Result<()> {
    let schema = Schema::get_by_name(ctx, "swifty").await?;

    let mgmt_code = r#"async function main({ thisComponent, components }) {
        return {
            status: "ok",
            ops: { update: { self: { attributes: { "/domain/name": "managed" } } } }
        };
    }"#;

    // Create a management overlay
    let func_id =
        create_overlay_management_func(ctx, schema.id(), "test:lockedOverlayMgmt", mgmt_code)
            .await?;

    // Lock the function to test updating locked overlays
    let func = Func::get_by_id(ctx, func_id).await?;
    func.lock(ctx).await?;

    // Verify the function is now locked
    let func = Func::get_by_id(ctx, func_id).await?;
    assert!(func.is_locked, "Overlay function should be locked");

    // Verify it does NOT have variant bindings (only schema-level)
    let has_variant_bindings = FuncBinding::has_variant_bindings(ctx, func_id).await?;
    assert!(
        !has_variant_bindings,
        "Function should not have variant bindings"
    );

    // Update the metadata - this should succeed even though the func is locked
    let result = FuncAuthoringClient::update_func(
        ctx,
        func_id,
        Some("Updated Display Name".to_string()),
        Some("Updated description".to_string()),
    )
    .await;
    if let Err(e) = &result {
        eprintln!("Error from update_func: {e:?}");
        eprintln!("has_variant_bindings returned: {has_variant_bindings}");
    }
    assert!(
        result.is_ok(),
        "update_func should succeed for locked overlay function: {:?}",
        result.err()
    );

    // Verify the metadata was actually updated
    let updated_func = result.unwrap();
    assert_eq!(
        Some("Updated Display Name".to_string()),
        updated_func.display_name,
        "Display name should be updated"
    );
    assert_eq!(
        Some("Updated description".to_string()),
        updated_func.description,
        "Description should be updated"
    );

    Ok(())
}

/// Test that locked variant-level functions still cannot be updated
#[test]
async fn update_locked_variant_func_fails(ctx: &mut DalContext) -> Result<()> {
    // Find a variant-level function
    let func_id = Func::find_id_by_name(ctx, "test:createActionStarfield")
        .await?
        .expect("could not find test:createActionStarfield");

    let func = Func::get_by_id(ctx, func_id).await?;

    // Verify it's locked and HAS variant bindings
    assert!(func.is_locked, "Test function should be locked");
    let has_variant_bindings = FuncBinding::has_variant_bindings(ctx, func_id).await?;
    assert!(
        has_variant_bindings,
        "Test function should have variant bindings"
    );

    // Try to update the code - this should fail
    let result = FuncAuthoringClient::save_code(ctx, func_id, "new code").await;
    assert!(
        result.is_err(),
        "save_code should fail for locked variant-level function"
    );

    // Try to update metadata - this should also fail
    let result =
        FuncAuthoringClient::update_func(ctx, func_id, Some("New Name".to_string()), None).await;
    assert!(
        result.is_err(),
        "update_func should fail for locked variant-level function"
    );

    Ok(())
}

/// Test that overlay functions remain accessible after removal and recreation
#[test]
async fn overlay_function_removal_and_recreation(ctx: &mut DalContext) -> Result<()> {
    let schema = Schema::get_by_name(ctx, "swifty").await?;

    let mgmt_code = r#"async function main({ thisComponent, components }) {
        return { status: "ok", ops: {} };
    }"#;

    // Create an overlay
    let func_id =
        create_overlay_management_func(ctx, schema.id(), "test:removableOverlay", mgmt_code)
            .await?;

    // Verify it does NOT have variant bindings (only schema-level)
    let has_variant_bindings = FuncBinding::has_variant_bindings(ctx, func_id).await?;
    assert!(
        !has_variant_bindings,
        "Should not have variant bindings initially (only schema-level)"
    );

    // Get the management prototype for the overlay
    let protos = ManagementPrototype::list_for_schema_id(ctx, schema.id()).await?;
    let mut proto = None;
    for p in &protos {
        let p_func_id = ManagementPrototype::func_id(ctx, p.id()).await?;
        if p_func_id == func_id {
            proto = Some(p);
            break;
        }
    }
    let proto = proto.expect("should find prototype");

    // Remove the prototype (simulating detach)
    ManagementPrototype::remove(ctx, proto.id()).await?;

    // Verify still no variant bindings after removal (and no bindings at all)
    let has_variant_bindings_after = FuncBinding::has_variant_bindings(ctx, func_id).await?;
    assert!(
        !has_variant_bindings_after,
        "Should not have variant bindings after removal"
    );

    Ok(())
}
