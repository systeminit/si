use base64::Engine;
use dal::{
    DalContext,
    Func,
    Schema,
    action::{
        Action,
        prototype::{
            ActionKind,
            ActionPrototype,
        },
    },
};
use dal_test::{
    Result,
    helpers::{
        ChangeSetTestHelpers,
        create_component_for_default_schema_name_in_default_view,
    },
    test,
};
use pretty_assertions_sorted::assert_eq;

/// Ensure that if a schema level action prototype is created in one change set,
/// and one for the same kind is created in another change set, a correction will
/// prevent two action prototypes for the same kind.
#[test]
async fn schema_level_action_prototype_across_change_sets(ctx: &mut DalContext) -> Result<()> {
    let first_change_set_id = ctx.change_set_id();
    let schema = Schema::get_by_name(ctx, "swifty").await?;
    let default_schema_variant_id = Schema::default_variant_id(ctx, schema.id()).await?;

    // Create a schema-level Create action prototype in the first change set
    let create_action_code_cs1 = "async function main() {
                return { payload: { \"change_set\": \"first\"}, status: \"ok\" };
            }";

    let create_func_cs1 = Func::new(
        ctx,
        "test:schemaCreateActionSwiftyCS1",
        None::<String>,
        None::<String>,
        None::<String>,
        false,
        false,
        dal::FuncBackendKind::JsAction,
        dal::FuncBackendResponseType::Action,
        "main".into(),
        Some(base64::engine::general_purpose::STANDARD_NO_PAD.encode(create_action_code_cs1)),
        false,
    )
    .await?;

    let schema_action_prototype_cs1 = ActionPrototype::new(
        ctx,
        ActionKind::Create,
        "test:schemaCreateActionSwiftyCS1".into(),
        None,
        schema.id(),
        create_func_cs1.id,
    )
    .await?;

    // Verify this is now the default Create action for the schema
    let default_create = ActionPrototype::find_by_kind_for_schema_or_variant(
        ctx,
        ActionKind::Create,
        default_schema_variant_id,
    )
    .await?
    .pop()
    .expect("should have one create action");

    assert_eq!(schema_action_prototype_cs1.id(), default_create.id());

    // Create a component to verify it uses the first prototype
    let component_cs1 =
        create_component_for_default_schema_name_in_default_view(ctx, "swifty", "component_cs1")
            .await?;

    let mut actions_cs1 = Action::find_for_component_id(ctx, component_cs1.id()).await?;
    assert_eq!(1, actions_cs1.len());

    let action_id_cs1 = actions_cs1.pop().expect("should have one action");
    let prototype_id_cs1 = Action::prototype_id(ctx, action_id_cs1).await?;
    assert_eq!(schema_action_prototype_cs1.id(), prototype_id_cs1);

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("unable to commit");

    // Fork head
    let second_change_set =
        ChangeSetTestHelpers::fork_from_head_change_set_with_name(ctx, "Second Change Set").await?;

    // Create another schema-level Create action prototype in the second change set
    let create_action_code_cs2 = "async function main() {
                return { payload: { \"change_set\": \"second\"}, status: \"ok\" };
            }";

    let create_func_cs2 = Func::new(
        ctx,
        "test:schemaCreateActionSwiftyCS2",
        None::<String>,
        None::<String>,
        None::<String>,
        false,
        false,
        dal::FuncBackendKind::JsAction,
        dal::FuncBackendResponseType::Action,
        "main".into(),
        Some(base64::engine::general_purpose::STANDARD_NO_PAD.encode(create_action_code_cs2)),
        false,
    )
    .await?;

    let schema_action_prototype_cs2 = ActionPrototype::new(
        ctx,
        ActionKind::Create,
        "test:schemaCreateActionSwiftyCS2".into(),
        None,
        schema.id(),
        create_func_cs2.id,
    )
    .await?;

    // Verify the second change set uses the new prototype
    let default_create_cs2 = ActionPrototype::find_by_kind_for_schema_or_variant(
        ctx,
        ActionKind::Create,
        default_schema_variant_id,
    )
    .await?
    .pop()
    .expect("should have one create action");

    assert_eq!(schema_action_prototype_cs2.id(), default_create_cs2.id());

    // Create a component in the second change set
    let component_cs2 =
        create_component_for_default_schema_name_in_default_view(ctx, "swifty", "component_cs2")
            .await?;

    let mut actions_cs2 = Action::find_for_component_id(ctx, component_cs2.id()).await?;
    assert_eq!(1, actions_cs2.len());

    let action_id_cs2 = actions_cs2.pop().expect("should have one action");
    let prototype_id_cs2 = Action::prototype_id(ctx, action_id_cs2).await?;
    assert_eq!(schema_action_prototype_cs2.id(), prototype_id_cs2);

    // Commit the second change set
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Switch to first chagne set and apply it to base
    ctx.update_visibility_and_snapshot_to_visibility(first_change_set_id)
        .await?;
    ChangeSetTestHelpers::apply_change_set_to_base(ctx).await?;

    // Back to second change set
    ctx.update_visibility_and_snapshot_to_visibility(second_change_set.id)
        .await?;

    // Check that there's only one Create action prototype for the schema in this change set
    let create_prototypes = ActionPrototype::find_by_kind_for_schema_or_variant(
        ctx,
        ActionKind::Create,
        default_schema_variant_id,
    )
    .await?;

    assert_eq!(
        1,
        create_prototypes.len(),
        "exactly one create action prototype"
    );

    let current_prototype = create_prototypes
        .first()
        .expect("should have one prototype");

    // Verify that components in the second change set use the correct prototype
    let mut actions_cs2_after = Action::find_for_component_id(ctx, component_cs2.id()).await?;
    assert_eq!(1, actions_cs2_after.len());

    let action_id_cs2_after = actions_cs2_after.pop().expect("should have one action");
    let prototype_id_cs2_after = Action::prototype_id(ctx, action_id_cs2_after).await?;

    // The action should use the prototype that exists in this change set
    assert_eq!(
        current_prototype.id(),
        prototype_id_cs2_after,
        "action should use the correct prototype"
    );

    // Create a new component to ensure new actions use the correct prototype
    let component_cs2_new = create_component_for_default_schema_name_in_default_view(
        ctx,
        "swifty",
        "component_cs2_new",
    )
    .await?;

    let mut actions_cs2_new = Action::find_for_component_id(ctx, component_cs2_new.id()).await?;
    assert_eq!(1, actions_cs2_new.len());

    let action_id_cs2_new = actions_cs2_new.pop().expect("should have one action");
    let prototype_id_cs2_new = Action::prototype_id(ctx, action_id_cs2_new).await?;

    assert_eq!(
        current_prototype.id(),
        prototype_id_cs2_new,
        "New component action should use the correct prototype"
    );

    Ok(())
}
