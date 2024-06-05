use dal::action::prototype::ActionKind;
use dal::func::authoring::{
    AttributeOutputLocation, CreateFuncOptions, FuncAuthoringClient, FuncAuthoringError,
};
use dal::func::FuncKind;
use dal::prop::PropPath;
use dal::{ChangeSet, DalContext, Func, OutputSocket, Prop, Schema, SchemaVariant};
use dal_test::helpers::ChangeSetTestHelpers;
use dal_test::test;

#[test]
async fn create_qualification_no_options(ctx: &mut DalContext) {
    let new_change_set = ChangeSet::fork_head(ctx, "new change set")
        .await
        .expect("could not create new change set");
    ctx.update_visibility_and_snapshot_to_visibility(new_change_set.id)
        .await
        .expect("could not update visibility");

    let func_name = "Paul's Test Func".to_string();
    let func = FuncAuthoringClient::create_func(
        ctx,
        FuncKind::Qualification,
        Some(func_name.clone()),
        None,
    )
    .await
    .expect("unable to create func");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    assert_eq!(FuncKind::Qualification, func.kind);
    assert_eq!(func_name, func.name);
    assert_eq!(Some("main".to_string()), func.handler);
    assert_eq!(Some("async function main(component: Input): Promise<Output> {\n  return {\n    result: 'success',\n    message: 'Component qualified'\n  };\n}\n".to_string()),  func.code);

    let head_change_set = ctx
        .get_workspace_default_change_set_id()
        .await
        .expect("Unable to find HEAD changeset id");

    ctx.update_visibility_and_snapshot_to_visibility(head_change_set)
        .await
        .expect("Unable to go back to HEAD");

    let head_func = Func::find_by_name(ctx, func_name.clone())
        .await
        .expect("Unable to get a func");
    assert!(head_func.is_none());
}

#[test]
async fn create_qualification_with_schema_variant(ctx: &mut DalContext) {
    let maybe_swifty_schema = Schema::find_by_name(ctx, "swifty")
        .await
        .expect("unable to get schema");
    assert!(maybe_swifty_schema.is_some());

    let swifty_schema = maybe_swifty_schema.unwrap();
    let maybe_sv_id = swifty_schema
        .get_default_schema_variant_id(ctx)
        .await
        .expect("unable to get schema variant");
    assert!(maybe_sv_id.is_some());
    let sv_id = maybe_sv_id.unwrap();

    let func_name = "Paul's Test Func".to_string();
    let func = FuncAuthoringClient::create_func(
        ctx,
        FuncKind::Qualification,
        Some(func_name.clone()),
        Some(CreateFuncOptions::QualificationOptions {
            schema_variant_id: sv_id,
        }),
    )
    .await
    .expect("unable to create func");

    let schema_funcs = SchemaVariant::all_funcs(ctx, sv_id)
        .await
        .expect("Unable to get all schema variant funcs");

    assert_eq!(FuncKind::Qualification, func.kind);
    assert_eq!(func_name, func.name);
    assert_eq!(Some("main".to_string()), func.handler);
    assert_eq!(Some("async function main(component: Input): Promise<Output> {\n  return {\n    result: 'success',\n    message: 'Component qualified'\n  };\n}\n".to_string()),  func.code);

    let mut expected_func: Vec<Func> = schema_funcs
        .into_iter()
        .filter(|f| f.name == func_name)
        .collect();
    assert!(!expected_func.is_empty());
    assert_eq!(func_name, expected_func.pop().unwrap().name);

    let head_change_set = ctx
        .get_workspace_default_change_set_id()
        .await
        .expect("Unable to find HEAD changeset id");

    ctx.update_visibility_and_snapshot_to_visibility(head_change_set)
        .await
        .expect("Unable to go back to HEAD");

    let head_func = Func::find_by_name(ctx, func_name.clone())
        .await
        .expect("Unable to get a func");
    assert!(head_func.is_none());
}

#[test]
async fn create_codegen_no_options(ctx: &mut DalContext) {
    let func_name = "Paul's Test Func".to_string();
    let func = FuncAuthoringClient::create_func(
        ctx,
        FuncKind::CodeGeneration,
        Some(func_name.clone()),
        None,
    )
    .await
    .expect("unable to create func");

    assert_eq!(FuncKind::CodeGeneration, func.kind);
    assert_eq!(func_name, func.name);
    assert_eq!(Some("main".to_string()), func.handler);
    assert_eq!(Some("async function main(component: Input): Promise<Output> {\n  return {\n    format: \"json\",\n    code: JSON.stringify(component),\n  };\n}\n".to_string()),  func.code);

    let head_change_set = ctx
        .get_workspace_default_change_set_id()
        .await
        .expect("Unable to find HEAD changeset id");

    ctx.update_visibility_and_snapshot_to_visibility(head_change_set)
        .await
        .expect("Unable to go back to HEAD");

    let head_func = Func::find_by_name(ctx, func_name.clone())
        .await
        .expect("Unable to get a func");
    assert!(head_func.is_none());
}

#[test]
async fn create_codegen_with_schema_variant(ctx: &mut DalContext) {
    let maybe_swifty_schema = Schema::find_by_name(ctx, "swifty")
        .await
        .expect("unable to get schema");
    assert!(maybe_swifty_schema.is_some());

    let swifty_schema = maybe_swifty_schema.unwrap();
    let maybe_sv_id = swifty_schema
        .get_default_schema_variant_id(ctx)
        .await
        .expect("unable to get schema variant");
    assert!(maybe_sv_id.is_some());
    let sv_id = maybe_sv_id.unwrap();

    let func_name = "Paul's Test Func".to_string();
    let func = FuncAuthoringClient::create_func(
        ctx,
        FuncKind::CodeGeneration,
        Some(func_name.clone()),
        Some(CreateFuncOptions::CodeGenerationOptions {
            schema_variant_id: sv_id,
        }),
    )
    .await
    .expect("unable to create func");

    let schema_funcs = SchemaVariant::all_funcs(ctx, sv_id)
        .await
        .expect("Unable to get all schema variant funcs");

    assert_eq!(FuncKind::CodeGeneration, func.kind);
    assert_eq!(func_name, func.name);
    assert_eq!(Some("main".to_string()), func.handler);
    assert_eq!(Some("async function main(component: Input): Promise<Output> {\n  return {\n    format: \"json\",\n    code: JSON.stringify(component),\n  };\n}\n".to_string()),  func.code);

    let mut expected_func: Vec<Func> = schema_funcs
        .into_iter()
        .filter(|f| f.name == func_name)
        .collect();
    assert!(!expected_func.is_empty());
    assert_eq!(func_name, expected_func.pop().unwrap().name);

    let head_change_set = ctx
        .get_workspace_default_change_set_id()
        .await
        .expect("Unable to find HEAD changeset id");

    ctx.update_visibility_and_snapshot_to_visibility(head_change_set)
        .await
        .expect("Unable to go back to HEAD");

    let head_func = Func::find_by_name(ctx, func_name.clone())
        .await
        .expect("Unable to get a func");
    assert!(head_func.is_none());
}

#[test]
async fn create_attribute_no_options(ctx: &mut DalContext) {
    let func_name = "Paul's Test Func".to_string();
    let func =
        FuncAuthoringClient::create_func(ctx, FuncKind::Attribute, Some(func_name.clone()), None)
            .await
            .expect("unable to create func");

    assert_eq!(FuncKind::Attribute, func.kind);
    assert_eq!(func_name, func.name);
    assert_eq!(Some("main".to_string()), func.handler);
    assert_eq!(
        Some(
            "async function main(input: Input): Promise<Output> {\n  return null;\n}\n".to_string()
        ),
        func.code
    );

    let head_change_set = ctx
        .get_workspace_default_change_set_id()
        .await
        .expect("Unable to find HEAD changeset id");

    ctx.update_visibility_and_snapshot_to_visibility(head_change_set)
        .await
        .expect("Unable to go back to HEAD");

    let head_func = Func::find_by_name(ctx, func_name.clone())
        .await
        .expect("Unable to get a func");
    assert!(head_func.is_none());
}

#[test]
async fn create_attribute_override_dynamic_func_for_prop(ctx: &mut DalContext) {
    let schema = Schema::find_by_name(ctx, "swifty")
        .await
        .expect("unable to find schema by name")
        .expect("schema not found");
    let schema_variant_id = schema
        .get_default_schema_variant_id(ctx)
        .await
        .expect("unable to get default schema variant id")
        .expect("default schema variant id not found");
    let prop_id = Prop::find_prop_id_by_path(
        ctx,
        schema_variant_id,
        &PropPath::new(["root", "domain", "name"]),
    )
    .await
    .expect("unable to get prop");

    // Create the func and commit.
    let func_name = "Paul's Test Func";
    let func = FuncAuthoringClient::create_func(
        ctx,
        FuncKind::Attribute,
        Some(func_name.to_string()),
        Some(CreateFuncOptions::AttributeOptions {
            output_location: AttributeOutputLocation::Prop { prop_id },
        }),
    )
    .await
    .expect("unable to create func");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    // Ensure that the created func looks as expected.
    assert_eq!(FuncKind::Attribute, func.kind);
    assert_eq!(func_name, func.name);
    assert_eq!(Some("main".to_string()), func.handler);
    assert_eq!(
        Some(
            "async function main(input: Input): Promise<Output> {\n  return null;\n}\n".to_string()
        ),
        func.code
    );

    // Ensure the func is created and associated with the schema variant.
    let funcs = SchemaVariant::all_funcs(ctx, schema_variant_id)
        .await
        .expect("Unable to get all schema variant funcs");
    let func = funcs
        .iter()
        .find(|f| f.name == func_name)
        .expect("func not found")
        .to_owned();
    assert_eq!(func_name, func.name);

    // Ensure that the func does not exist on head.
    let head_change_set = ctx
        .get_workspace_default_change_set_id()
        .await
        .expect("Unable to find HEAD changeset id");
    ctx.update_visibility_and_snapshot_to_visibility_no_editing_change_set(head_change_set)
        .await
        .expect("Unable to go back to HEAD");
    let head_func = Func::find_by_name(ctx, func_name)
        .await
        .expect("Unable to get a func");
    assert!(head_func.is_none());
}

#[test]
async fn create_attribute_override_dynamic_func_for_output_socket(ctx: &mut DalContext) {
    let schema = Schema::find_by_name(ctx, "swifty")
        .await
        .expect("unable to find schema by name")
        .expect("schema not found");
    let schema_variant_id = schema
        .get_default_schema_variant_id(ctx)
        .await
        .expect("unable to get default schema variant id")
        .expect("default schema variant id not found");
    let output_socket = OutputSocket::find_with_name(ctx, "anything", schema_variant_id)
        .await
        .expect("could not perform find output socket")
        .expect("output socket not found");

    // Create the func and commit.
    let func_name = "Paul's Test Func";
    let func = FuncAuthoringClient::create_func(
        ctx,
        FuncKind::Attribute,
        Some(func_name.to_string()),
        Some(CreateFuncOptions::AttributeOptions {
            output_location: AttributeOutputLocation::OutputSocket {
                output_socket_id: output_socket.id(),
            },
        }),
    )
    .await
    .expect("unable to create func");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    // Ensure that the created func looks as expected.
    assert_eq!(FuncKind::Attribute, func.kind);
    assert_eq!(func_name, func.name);
    assert_eq!(Some("main".to_string()), func.handler);
    assert_eq!(
        Some(
            "async function main(input: Input): Promise<Output> {\n  return null;\n}\n".to_string()
        ),
        func.code
    );

    // Ensure the func is created and associated with the schema variant.
    let funcs = SchemaVariant::all_funcs(ctx, schema_variant_id)
        .await
        .expect("Unable to get all schema variant funcs");
    let func = funcs
        .iter()
        .find(|f| f.name == func_name)
        .expect("func not found")
        .to_owned();
    assert_eq!(func_name, func.name);

    // Ensure that the func does not exist on head.
    let head_change_set = ctx
        .get_workspace_default_change_set_id()
        .await
        .expect("Unable to find HEAD changeset id");
    ctx.update_visibility_and_snapshot_to_visibility_no_editing_change_set(head_change_set)
        .await
        .expect("Unable to go back to HEAD");
    let head_func = Func::find_by_name(ctx, func_name)
        .await
        .expect("Unable to get a func");
    assert!(head_func.is_none());
}

#[test]
async fn create_action_no_options(ctx: &mut DalContext) {
    let func_name = "Paul's Test Action Func".to_string();
    let func =
        FuncAuthoringClient::create_func(ctx, FuncKind::Action, Some(func_name.clone()), None)
            .await
            .expect("unable to create func");

    assert_eq!(FuncKind::Action, func.kind);
    assert_eq!(func_name, func.name);
    assert_eq!(Some("main".to_string()), func.handler);
    assert_eq!(
        Some("async function main(component: Input): Promise<Output> {\n  throw new Error(\"unimplemented!\");\n}\n".to_string()),
        func.code
    );

    let head_change_set = ctx
        .get_workspace_default_change_set_id()
        .await
        .expect("Unable to find HEAD changeset id");

    ctx.update_visibility_and_snapshot_to_visibility(head_change_set)
        .await
        .expect("Unable to go back to HEAD");

    let head_func = Func::find_by_name(ctx, func_name.clone())
        .await
        .expect("Unable to get a func");
    assert!(head_func.is_none());
}

#[test]
async fn create_action_with_schema_variant(ctx: &mut DalContext) {
    let maybe_swifty_schema = Schema::find_by_name(ctx, "small even lego")
        .await
        .expect("unable to get schema");
    assert!(maybe_swifty_schema.is_some());

    let swifty_schema = maybe_swifty_schema.unwrap();
    let maybe_sv_id = swifty_schema
        .get_default_schema_variant_id(ctx)
        .await
        .expect("unable to get schema variant");
    assert!(maybe_sv_id.is_some());
    let sv_id = maybe_sv_id.unwrap();

    let func_name = "Paul's Test Action Func".to_string();
    let func = FuncAuthoringClient::create_func(
        ctx,
        FuncKind::Action,
        Some(func_name.clone()),
        Some(CreateFuncOptions::ActionOptions {
            schema_variant_id: sv_id,
            action_kind: ActionKind::Update,
        }),
    )
    .await
    .expect("unable to create func");

    let schema_funcs = SchemaVariant::all_funcs(ctx, sv_id)
        .await
        .expect("Unable to get all schema variant funcs");

    assert_eq!(FuncKind::Action, func.kind);
    assert_eq!(func_name, func.name);
    assert_eq!(Some("main".to_string()), func.handler);
    assert_eq!(
        Some("async function main(component: Input): Promise<Output> {\n  throw new Error(\"unimplemented!\");\n}\n".to_string()),
        func.code
    );

    let mut expected_func: Vec<Func> = schema_funcs
        .into_iter()
        .filter(|f| f.name == func_name)
        .collect();
    assert!(!expected_func.is_empty());

    let action_func = expected_func.pop().unwrap();
    assert_eq!(func_name, action_func.name);

    let head_change_set = ctx
        .get_workspace_default_change_set_id()
        .await
        .expect("Unable to find HEAD changeset id");

    ctx.update_visibility_and_snapshot_to_visibility(head_change_set)
        .await
        .expect("Unable to go back to HEAD");

    let head_func = Func::find_by_name(ctx, func_name.clone())
        .await
        .expect("Unable to get a func");
    assert!(head_func.is_none());
}

#[test]
async fn duplicate_action_kinds_causes_error(ctx: &mut DalContext) {
    let maybe_swifty_schema = Schema::find_by_name(ctx, "small even lego")
        .await
        .expect("unable to get schema");
    assert!(maybe_swifty_schema.is_some());

    let swifty_schema = maybe_swifty_schema.unwrap();
    let maybe_sv_id = swifty_schema
        .get_default_schema_variant_id(ctx)
        .await
        .expect("unable to get schema variant");
    assert!(maybe_sv_id.is_some());
    let sv_id = maybe_sv_id.unwrap();

    let func_name = "Paul's Test Action Func".to_string();
    let action_kind = ActionKind::Create;
    let func = FuncAuthoringClient::create_func(
        ctx,
        FuncKind::Action,
        Some(func_name.clone()),
        Some(CreateFuncOptions::ActionOptions {
            schema_variant_id: sv_id,
            action_kind,
        }),
    )
    .await;

    if let Err(FuncAuthoringError::ActionKindAlreadyExists(
        err_action_kind,
        err_schema_variant_id,
    )) = func
    {
        assert_eq!(action_kind, err_action_kind);
        assert_eq!(sv_id, err_schema_variant_id);
    } else {
        panic!("Test should fail if we don't get action kind already exists error");
    }
}

#[test]
async fn duplicate_func_name_causes_error(ctx: &mut DalContext) {
    let func_name = "Paul's Test Func".to_string();
    FuncAuthoringClient::create_func(ctx, FuncKind::Action, Some(func_name.clone()), None)
        .await
        .expect("unable to create func");

    let func = FuncAuthoringClient::create_func(
        ctx,
        FuncKind::CodeGeneration,
        Some(func_name.clone()),
        None,
    )
    .await;

    if let Err(FuncAuthoringError::FuncNameExists(errored_func_name)) = func {
        assert_eq!(func_name, errored_func_name)
    } else {
        panic!("Test should fail if we don't get this func exists in change set error")
    }
}
