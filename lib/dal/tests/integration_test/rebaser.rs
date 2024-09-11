use base64::{engine::general_purpose, Engine};
use dal::func::argument::{FuncArgument, FuncArgumentKind};
use dal::{AttributeValue, Component, DalContext, Func, FuncBackendKind, FuncBackendResponseType};
use dal_test::helpers::{create_component_for_default_schema_name, ChangeSetTestHelpers};
use dal_test::test;
use pretty_assertions_sorted::assert_eq;

#[test]
async fn modify_func_node(ctx: &mut DalContext) {
    let code_base64 = general_purpose::STANDARD_NO_PAD.encode("this is code");

    let func = Func::new(
        ctx,
        "test",
        None::<String>,
        None::<String>,
        None::<String>,
        false,
        false,
        FuncBackendKind::JsAttribute,
        FuncBackendResponseType::Boolean,
        None::<String>,
        Some(code_base64.clone()),
    )
    .await
    .expect("able to make a func");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    Func::get_by_id_or_error(ctx, func.id)
        .await
        .expect("able to get func by id");

    assert_eq!(Some(code_base64), func.code_base64);

    let new_code_base64 = general_purpose::STANDARD_NO_PAD.encode("this is new code");

    let func = func
        .modify(ctx, |f| {
            f.name = "i changed this".into();

            Ok(())
        })
        .await
        .expect("able to modify func");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit");

    let refetched_func = Func::get_by_id_or_error(ctx, func.id)
        .await
        .expect("able to fetch func");

    assert_eq!("i changed this", refetched_func.name.as_str());

    let func = func
        .modify(ctx, |f| {
            f.code_base64 = Some(new_code_base64.clone());

            Ok(())
        })
        .await
        .expect("able to modify func");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    let modified_func = Func::get_by_id_or_error(ctx, func.id)
        .await
        .expect("able to get func by id again");

    assert_eq!(
        Some(new_code_base64.as_str()),
        modified_func.code_base64.as_deref()
    );

    let funcs = Func::list_for_default_and_editing(ctx)
        .await
        .expect("able to list funcs");
    let modified_func_again = funcs
        .iter()
        .find(|f| f.id == modified_func.id)
        .expect("func should be in list");

    assert_eq!(Some(new_code_base64), modified_func_again.code_base64);
}

#[test]
async fn func_node_with_arguments(ctx: &mut DalContext) {
    let code_base64 = general_purpose::STANDARD_NO_PAD.encode("this is code");

    let func = Func::new(
        ctx,
        "test",
        None::<String>,
        None::<String>,
        None::<String>,
        false,
        false,
        FuncBackendKind::JsAttribute,
        FuncBackendResponseType::Boolean,
        None::<String>,
        Some(code_base64),
    )
    .await
    .expect("able to make a func");

    Func::get_by_id_or_error(ctx, func.id)
        .await
        .expect("able to get func by id before commit");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    Func::get_by_id_or_error(ctx, func.id)
        .await
        .expect("able to get func by id after rebase");

    let new_code_base64 = general_purpose::STANDARD_NO_PAD.encode("this is new code");

    // modify func
    let func = func
        .modify(ctx, |f| {
            f.name = "test:modified".into();
            f.code_base64 = Some(new_code_base64.clone());

            Ok(())
        })
        .await
        .expect("able to modify func");

    // create func arguments
    let arg_1 = FuncArgument::new(ctx, "argle bargle", FuncArgumentKind::Object, None, func.id)
        .await
        .expect("able to create func argument");
    FuncArgument::new(ctx, "argy bargy", FuncArgumentKind::Object, None, func.id)
        .await
        .expect("able to create func argument 2");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    let modified_func = Func::get_by_id_or_error(ctx, func.id)
        .await
        .expect("able to get func by id again");

    assert_eq!(
        Some(new_code_base64).as_deref(),
        modified_func.code_base64.as_deref()
    );
    assert_eq!("test:modified", modified_func.name.as_str());

    let args = FuncArgument::list_for_func(ctx, modified_func.id)
        .await
        .expect("able to list args");

    assert_eq!(2, args.len());

    // Modify func argument
    FuncArgument::modify_by_id(ctx, arg_1.id, |arg| {
        arg.name = "bargle argle".into();

        Ok(())
    })
    .await
    .expect("able to modify func");

    let func_arg_refetch = FuncArgument::get_by_id_or_error(ctx, arg_1.id)
        .await
        .expect("get func arg");

    assert_eq!(
        "bargle argle",
        func_arg_refetch.name.as_str(),
        "refetch should have updated func arg name"
    );

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");
    let args = FuncArgument::list_for_func(ctx, func.id)
        .await
        .expect("able to list args again");

    assert_eq!(2, args.len());

    let modified_arg = args
        .iter()
        .find(|a| a.id == arg_1.id)
        .expect("able to get modified func arg");

    assert_eq!(
        "bargle argle",
        modified_arg.name.as_str(),
        "modified func arg should have new name after rebase"
    );
}

#[test]
async fn delete_func_node(ctx: &mut DalContext) {
    let code_base64 = general_purpose::STANDARD_NO_PAD.encode("this is code");

    let func = Func::new(
        ctx,
        "test",
        None::<String>,
        None::<String>,
        None::<String>,
        false,
        false,
        FuncBackendKind::JsAttribute,
        FuncBackendResponseType::Boolean,
        None::<String>,
        Some(code_base64),
    )
    .await
    .expect("able to make a func");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    let snapshot_id_before_deletion = ctx.workspace_snapshot().expect("get snap").id().await;

    Func::get_by_id_or_error(ctx, func.id)
        .await
        .expect("able to get func by id");

    Func::delete_by_id(ctx, func.id)
        .await
        .expect("able to remove func");

    assert!(Func::get_by_id_or_error(ctx, func.id).await.is_err());

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    let snapshot_id_after_deletion = ctx.workspace_snapshot().expect("get snap").id().await;

    // A sanity check
    assert_ne!(snapshot_id_before_deletion, snapshot_id_after_deletion);

    let result = Func::get_by_id_or_error(ctx, func.id).await;
    assert!(result.is_err());
}

#[test]
async fn correctly_detect_unrelated_unmodified_data(ctx: &mut DalContext) {
    let shared_component_id =
        create_component_for_default_schema_name(ctx, "swifty", "Shared component")
            .await
            .expect("could not create component")
            .id();
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("Unable to commit_and_update_snapshot_to_visibility");
    // Swifty is in HEAD
    ChangeSetTestHelpers::apply_change_set_to_base(ctx)
        .await
        .expect("Unable to merge to base change set");

    // Both change sets have Swifty in HEAD
    let change_set_a =
        ChangeSetTestHelpers::fork_from_head_change_set_with_name(ctx, "Change set A")
            .await
            .expect("Unable to create change set A");
    let change_set_b =
        ChangeSetTestHelpers::fork_from_head_change_set_with_name(ctx, "Change set B")
            .await
            .expect("Unable to create change set B");

    // Modify the shared component in change set A.
    ctx.update_visibility_and_snapshot_to_visibility(change_set_a.id)
        .await
        .expect("Unable to switch to change set A");
    let cs_a_name_av_id = {
        let component = Component::get_by_id(ctx, shared_component_id)
            .await
            .expect("Unable to get shared component in change set A");
        component
            .attribute_values_for_prop(ctx, &["root", "si", "name"])
            .await
            .expect("Unable to get attribute values for si.name")
            .first()
            .copied()
            .expect("si.name attribute value not found")
    };
    // Swifty in Change Set A has a name of 'Modified in change set A'
    AttributeValue::update(
        ctx,
        cs_a_name_av_id,
        Some(serde_json::json!("Modified in change set A")),
    )
    .await
    .expect("Unable to update shared component name in change set A");
    // Change set A is committed to the rebaser
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("Unable to commit_and_update_snapshot_to_visibility for change set A");

    // Create a new component in change set B.
    ctx.update_visibility_and_snapshot_to_visibility(change_set_b.id)
        .await
        .expect("Unable to switch to change set B");
    let _change_set_b_component_id =
        create_component_for_default_schema_name(ctx, "swifty", "Change Set B Component")
            .await
            .expect("could not creat component")
            .id();
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("Unable to commit_and_update_snapshot_to_visibility for change set B");

    // Merge change set A to head.
    ctx.update_visibility_and_snapshot_to_visibility(change_set_a.id)
        .await
        .expect("Unable to switch to change set A");
    ChangeSetTestHelpers::apply_change_set_to_base(ctx)
        .await
        .expect("Unable to merge change set A");

    // Make sure the shared component looks as we'd expect.
    let expected_qualification_item_count = 1;
    let shared_component = Component::get_by_id(ctx, shared_component_id)
        .await
        .expect("Unable to get shared component in change set A");
    let av_ids = shared_component
        .attribute_values_for_prop(ctx, &["root", "si", "name"])
        .await
        .expect("Unable to get attribute values for si.name");
    assert_eq!(av_ids.len(), 1, "Found more than one AV for si.name");
    let av_ids = shared_component
        .attribute_values_for_prop(ctx, &["root", "domain", "name"])
        .await
        .expect("Unable to get attribute values for domain.name");
    assert_eq!(av_ids.len(), 1, "Found more than one AV for domain.name");
    let av_ids = shared_component
        .attribute_values_for_prop(ctx, &["root", "qualification", "qualificationItem"])
        .await
        .expect("Unable to get attribute values for qualification.qualificationItem");
    assert_eq!(
        av_ids.len(),
        expected_qualification_item_count,
        "Found more than one AV for qualification.qualificationItem"
    );

    // Merge change set B to head.
    ctx.update_visibility_and_snapshot_to_visibility(change_set_b.id)
        .await
        .expect("Unable to switch to change set B");
    ChangeSetTestHelpers::apply_change_set_to_base(ctx)
        .await
        .expect("Unable to merge change set B");

    // Make sure merging change set B didn't affect anything it didn't touch.
    let shared_component = Component::get_by_id(ctx, shared_component_id)
        .await
        .expect("Unable to get shared component in change set A");
    let av_ids = shared_component
        .attribute_values_for_prop(ctx, &["root", "si", "name"])
        .await
        .expect("Unable to get attribute values for si.name");
    assert_eq!(av_ids.len(), 1, "Found more than one AV for si.name");
    let av_ids = shared_component
        .attribute_values_for_prop(ctx, &["root", "domain", "name"])
        .await
        .expect("Unable to get attribute values for domain.name");
    assert_eq!(av_ids.len(), 1, "Found more than one AV for domain.name");
    let av_ids = shared_component
        .attribute_values_for_prop(ctx, &["root", "qualification", "qualificationItem"])
        .await
        .expect("Unable to get attribute values for qualification.qualificationItem");
    assert_eq!(
        av_ids.len(),
        expected_qualification_item_count,
        "Found more than one AV for qualification.qualificationItem"
    );
}
