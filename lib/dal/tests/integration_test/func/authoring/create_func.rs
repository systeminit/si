use dal::{
    AttributeValue,
    DalContext,
    Func,
    OutputSocket,
    Prop,
    Schema,
    SchemaVariant,
    action::prototype::ActionKind,
    diagram::Diagram,
    func::{
        FuncKind,
        authoring::{
            FuncAuthoringClient,
            FuncAuthoringError,
        },
        binding::{
            AttributeFuncDestination,
            EventualParent,
        },
        leaf::{
            LeafInputLocation,
            LeafKind,
        },
    },
    prop::PropPath,
    schema::variant::authoring::VariantAuthoringClient,
};
use dal_test::{
    helpers::{
        ChangeSetTestHelpers,
        create_component_for_default_schema_name_in_default_view,
        create_unlocked_variant_copy_for_schema_name,
    },
    test,
};

#[test(enable_veritech)]
async fn create_qualification_with_schema_variant(ctx: &mut DalContext) {
    let swifty_schema = Schema::get_by_name(ctx, "swifty")
        .await
        .expect("unable to get schema");

    let sv_id = Schema::default_variant_id(ctx, swifty_schema.id())
        .await
        .expect("unable to get schema variant");

    let func_name = "Paul's Test Func 2".to_string();
    assert!(
        FuncAuthoringClient::create_new_leaf_func(
            ctx,
            Some(func_name.clone()),
            LeafKind::Qualification,
            EventualParent::SchemaVariant(sv_id),
            &[],
        )
        .await
        .is_err()
    );

    Func::find_id_by_name(ctx, func_name.clone())
        .await
        .expect("has func");

    let new_sv = VariantAuthoringClient::create_unlocked_variant_copy(ctx, sv_id)
        .await
        .expect("can unlock sv")
        .id();
    let func = FuncAuthoringClient::create_new_leaf_func(
        ctx,
        Some(func_name.clone()),
        LeafKind::Qualification,
        EventualParent::SchemaVariant(new_sv),
        &[],
    )
    .await
    .expect("can create func");

    let schema_funcs = SchemaVariant::all_funcs(ctx, new_sv)
        .await
        .expect("Unable to get all schema variant funcs");

    assert_eq!(FuncKind::Qualification, func.kind);
    assert_eq!(func_name, func.name);
    assert_eq!(Some("main".to_string()), func.handler);
    assert_eq!(Some("async function main(component: Input): Promise<Output> {\n  return {\n    result: 'success',\n    message: 'Component qualified'\n  };\n}\n".to_string()),  func.code_plaintext().expect("has code"));

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

    let head_func = Func::find_id_by_name(ctx, func_name.clone())
        .await
        .expect("Unable to get a func");
    assert!(head_func.is_none());
}

#[test(enable_veritech)]
async fn create_codegen_with_schema_variant(ctx: &mut DalContext) {
    let swifty_schema = Schema::get_by_name(ctx, "swifty")
        .await
        .expect("unable to get schema");

    let sv_id = Schema::default_variant_id(ctx, swifty_schema.id())
        .await
        .expect("unable to get schema variant");

    // create unlocked copy
    let sv_id = VariantAuthoringClient::create_unlocked_variant_copy(ctx, sv_id)
        .await
        .expect("could create unlocked copy")
        .id();
    let func_name = "Paul's Test Func".to_string();

    let func = FuncAuthoringClient::create_new_leaf_func(
        ctx,
        Some(func_name.clone()),
        LeafKind::CodeGeneration,
        EventualParent::SchemaVariant(sv_id),
        &[],
    )
    .await
    .expect("unable to create func");

    let schema_funcs = SchemaVariant::all_funcs(ctx, sv_id)
        .await
        .expect("Unable to get all schema variant funcs");

    assert_eq!(FuncKind::CodeGeneration, func.kind);
    assert_eq!(func_name, func.name);
    assert_eq!(Some("main".to_string()), func.handler);
    assert_eq!(Some("async function main(component: Input): Promise<Output> {\n  return {\n    format: \"json\",\n    code: JSON.stringify(component),\n  };\n}\n".to_string()),  func.code_plaintext().expect("has code"));

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

    let head_func = Func::find_id_by_name(ctx, func_name.clone())
        .await
        .expect("Unable to get a func");
    assert!(head_func.is_none());
}

#[test(enable_veritech)]
async fn create_attribute_override_dynamic_func_for_prop(ctx: &mut DalContext) {
    let schema = Schema::get_by_name(ctx, "swifty")
        .await
        .expect("schema not found");
    let schema_variant_id = Schema::default_variant_id(ctx, schema.id())
        .await
        .expect("unable to get default schema variant id");
    // create unlocked copy
    let schema_variant_id =
        VariantAuthoringClient::create_unlocked_variant_copy(ctx, schema_variant_id)
            .await
            .expect("could create unlocked copy")
            .id();
    let prop_id = Prop::find_prop_id_by_path(
        ctx,
        schema_variant_id,
        &PropPath::new(["root", "domain", "name"]),
    )
    .await
    .expect("unable to get prop");

    // Create the func and commit.
    let func_name = "Paul's Test Func";
    let func = FuncAuthoringClient::create_new_attribute_func(
        ctx,
        Some(func_name.to_string()),
        None,
        AttributeFuncDestination::Prop(prop_id),
        Vec::new(),
    )
    .await
    .expect("could not create func");

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
        func.code_plaintext().expect("has code")
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
    ctx.update_visibility_and_snapshot_to_visibility(head_change_set)
        .await
        .expect("Unable to go back to HEAD");
    let head_func = Func::find_id_by_name(ctx, func_name)
        .await
        .expect("Unable to get a func");
    assert!(head_func.is_none());
}

#[test(enable_veritech)]
async fn create_attribute_override_dynamic_func_for_output_socket(ctx: &mut DalContext) {
    let schema = Schema::get_by_name(ctx, "swifty")
        .await
        .expect("schema not found");
    let schema_variant_id = Schema::default_variant_id(ctx, schema.id())
        .await
        .expect("unable to get default schema variant id");
    // create unlocked copy
    let schema_variant_id =
        VariantAuthoringClient::create_unlocked_variant_copy(ctx, schema_variant_id)
            .await
            .expect("can create unlocked copy")
            .id();
    let output_socket = OutputSocket::find_with_name(ctx, "anything", schema_variant_id)
        .await
        .expect("could not perform find output socket")
        .expect("output socket not found");

    // Create the func and commit.
    let func_name = "Paul's Test Func";

    let func = FuncAuthoringClient::create_new_attribute_func(
        ctx,
        Some(func_name.to_string()),
        None,
        AttributeFuncDestination::OutputSocket(output_socket.id()),
        vec![],
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
        func.code_plaintext().expect("has code")
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
    ctx.update_visibility_and_snapshot_to_visibility(head_change_set)
        .await
        .expect("Unable to go back to HEAD");
    let head_func = Func::find_id_by_name(ctx, func_name)
        .await
        .expect("Unable to get a func");
    assert!(head_func.is_none());
}

#[test(enable_veritech)]
async fn create_action_with_schema_variant(ctx: &mut DalContext) {
    let swifty_schema = Schema::get_by_name(ctx, "small even lego")
        .await
        .expect("unable to get schema");

    let sv_id = Schema::default_variant_id(ctx, swifty_schema.id())
        .await
        .expect("unable to get schema variant");
    // create unlocked copy
    let sv_id = VariantAuthoringClient::create_unlocked_variant_copy(ctx, sv_id)
        .await
        .expect("can create unlocked copy")
        .id();

    let func_name = "Paul's Test Action Func".to_string();
    let func = FuncAuthoringClient::create_new_action_func(
        ctx,
        Some(func_name.clone()),
        ActionKind::Update,
        sv_id,
    )
    .await
    .expect("could not create action func");

    let schema_funcs = SchemaVariant::all_funcs(ctx, sv_id)
        .await
        .expect("Unable to get all schema variant funcs");

    assert_eq!(FuncKind::Action, func.kind);
    assert_eq!(func_name, func.name);
    assert_eq!(Some("main".to_string()), func.handler);
    assert_eq!(
        Some("async function main(component: Input): Promise<Output> {\n  throw new Error(\"unimplemented!\");\n}\n".to_string()),
        func.code_plaintext().expect("has code")
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

    let head_func = Func::find_id_by_name(ctx, func_name.clone())
        .await
        .expect("Unable to get a func");
    assert!(head_func.is_none());
}

#[test(enable_veritech)]
async fn duplicate_action_kinds_causes_error(ctx: &mut DalContext) {
    let schema_variant_id = create_unlocked_variant_copy_for_schema_name(ctx, "small even lego")
        .await
        .expect("could not create unlocked copy");
    let action_kind = ActionKind::Create;
    let func_name = "Paul's Test Action Func".to_string();
    let func = FuncAuthoringClient::create_new_action_func(
        ctx,
        Some(func_name.clone()),
        ActionKind::Create,
        schema_variant_id,
    )
    .await;

    if let Err(FuncAuthoringError::ActionKindAlreadyExists(
        err_action_kind,
        err_schema_variant_id,
    )) = func
    {
        assert_eq!(action_kind, err_action_kind);
        assert_eq!(schema_variant_id, err_schema_variant_id);
    } else {
        panic!("Test should fail if we don't get action kind already exists error");
    }
}

#[test(enable_veritech)]
async fn duplicate_func_name_in_different_schema_variants_is_ok(ctx: &mut DalContext) {
    let schema_variant_id = create_unlocked_variant_copy_for_schema_name(ctx, "katy perry")
        .await
        .expect("could not create unlocked copy");

    let schema_variant_2_id = create_unlocked_variant_copy_for_schema_name(ctx, "pirate")
        .await
        .expect("could not create unlocked copy 2");

    let func_name = "Paul's Test Func".to_string();

    FuncAuthoringClient::create_new_action_func(
        ctx,
        Some(func_name.clone()),
        ActionKind::Create,
        schema_variant_id,
    )
    .await
    .expect("unable to create func");

    FuncAuthoringClient::create_new_leaf_func(
        ctx,
        Some(func_name.clone()),
        LeafKind::CodeGeneration,
        EventualParent::SchemaVariant(schema_variant_2_id),
        &[LeafInputLocation::Domain],
    )
    .await
    .expect("unable to create second func");
}

#[test(enable_veritech)]
async fn duplicate_func_name_in_same_schema_causes_error(ctx: &mut DalContext) {
    let schema_variant_id = create_unlocked_variant_copy_for_schema_name(ctx, "katy perry")
        .await
        .expect("could not create unlocked copy");

    let func_name = "Paul's Test Func".to_string();
    FuncAuthoringClient::create_new_action_func(
        ctx,
        Some(func_name.clone()),
        ActionKind::Create,
        schema_variant_id,
    )
    .await
    .expect("unable to create func");

    let func = FuncAuthoringClient::create_new_leaf_func(
        ctx,
        Some(func_name.clone()),
        LeafKind::CodeGeneration,
        EventualParent::SchemaVariant(schema_variant_id),
        &[LeafInputLocation::Domain],
    )
    .await;

    if let Err(FuncAuthoringError::FuncNameExistsOnVariant(errored_func_name, _)) = func {
        assert_eq!(func_name, errored_func_name)
    } else {
        panic!("Test should fail if we don't get this func exists in change set error")
    }
}

#[test(enable_veritech)]
async fn create_qualification_and_code_gen_with_existing_component(ctx: &mut DalContext) {
    let asset_name = "britsTestAsset".to_string();
    let description = None;
    let link = None;
    let category = "Integration Tests".to_string();
    let color = "#00b0b0".to_string();
    let variant_zero = VariantAuthoringClient::create_schema_and_variant(
        ctx,
        asset_name.clone(),
        description.clone(),
        link.clone(),
        category.clone(),
        color.clone(),
    )
    .await
    .expect("Unable to create new asset");

    let my_asset_schema = variant_zero
        .schema(ctx)
        .await
        .expect("Unable to get the schema for the variant");

    let default_schema_variant = Schema::default_variant_id(ctx, my_asset_schema.id())
        .await
        .expect("unable to get the default schema variant id");
    assert_eq!(default_schema_variant, variant_zero.id());

    // Now let's update the variant
    let first_code_update = "function main() {\n
     const myProp = new PropBuilder().setName(\"testProp\").setKind(\"string\").build()
     const myProp2 = new PropBuilder().setName(\"testPropWillRemove\").setKind(\"string\").build()
     const arrayProp = new PropBuilder().setName(\"arrayProp\").setKind(\"array\").setEntry(\n
        new PropBuilder().setName(\"arrayElem\").setKind(\"string\").build()\n
    ).build();\n
     return new AssetBuilder().addProp(myProp).addProp(arrayProp).build()\n}"
        .to_string();

    VariantAuthoringClient::save_variant_content(
        ctx,
        variant_zero.id(),
        my_asset_schema.name.clone(),
        variant_zero.display_name(),
        variant_zero.category(),
        variant_zero.description(),
        variant_zero.link(),
        variant_zero
            .get_color(ctx)
            .await
            .expect("get color from schema variant"),
        variant_zero.component_type(),
        Some(first_code_update),
    )
    .await
    .expect("save variant contents");

    let updated_variant_id = VariantAuthoringClient::regenerate_variant(ctx, variant_zero.id())
        .await
        .expect("unable to update asset");

    // We should still see that the schema variant we updated is the same as we have no components on the graph
    assert_eq!(variant_zero.id(), updated_variant_id);

    // Add a component to the diagram
    let initial_component = create_component_for_default_schema_name_in_default_view(
        ctx,
        my_asset_schema.name.clone(),
        "demo component",
    )
    .await
    .expect("could not create component");
    let initial_diagram = Diagram::assemble_for_default_view(ctx)
        .await
        .expect("could not assemble diagram");
    assert_eq!(1, initial_diagram.components.len());

    let domain_prop_av_id = initial_component
        .domain_prop_attribute_value(ctx)
        .await
        .expect("able to get domain prop");

    // Set the domain so we get some array elements
    AttributeValue::update(
        ctx,
        domain_prop_av_id,
        Some(serde_json::json!({
            "testProp": "test",
            "testPropWillRemove": "testToBeRemoved",
            "arrayProp": [
                "first",
                "second"
            ]
        })),
    )
    .await
    .expect("update failed");

    // Let's ensure that our prop is visible in the component
    Prop::find_prop_id_by_path(
        ctx,
        updated_variant_id,
        &PropPath::new(["root", "domain", "testProp"]),
    )
    .await
    .expect("able to find testProp prop");
    // now let's create a new code gen for the new schema variant
    let func_name = "Code Gen Func".to_string();

    let func = FuncAuthoringClient::create_new_leaf_func(
        ctx,
        Some(func_name.clone()),
        LeafKind::CodeGeneration,
        EventualParent::SchemaVariant(updated_variant_id),
        &[],
    )
    .await
    .expect("could not create func");

    let schema_funcs = SchemaVariant::all_funcs(ctx, updated_variant_id)
        .await
        .expect("Unable to get all schema variant funcs");

    assert_eq!(FuncKind::CodeGeneration, func.kind);
    assert_eq!(func_name, func.name);
    assert_eq!(Some("main".to_string()), func.handler);
    assert_eq!(Some("async function main(component: Input): Promise<Output> {\n  return {\n    format: \"json\",\n    code: JSON.stringify(component),\n  };\n}\n".to_string()),
     func.code_plaintext().expect("has code"));

    let mut expected_func: Vec<Func> = schema_funcs
        .into_iter()
        .filter(|f| f.name == func_name)
        .collect();
    assert!(!expected_func.is_empty());
    assert_eq!(func_name, expected_func.pop().unwrap().name);

    // let's also create a new qualification fo the new schema variant
    let func_name = "Qualification Func".to_string();
    let func = FuncAuthoringClient::create_new_leaf_func(
        ctx,
        Some(func_name.clone()),
        LeafKind::Qualification,
        EventualParent::SchemaVariant(updated_variant_id),
        &[],
    )
    .await
    .expect("could not create func");

    let schema_funcs = SchemaVariant::all_funcs(ctx, updated_variant_id)
        .await
        .expect("Unable to get all schema variant funcs");

    assert_eq!(FuncKind::Qualification, func.kind);
    assert_eq!(func_name, func.name);
    assert_eq!(Some("main".to_string()), func.handler);
    assert_eq!(Some("async function main(component: Input): Promise<Output> {\n  return {\n    result: 'success',\n    message: 'Component qualified'\n  };\n}\n".to_string()),  func.code_plaintext().expect("has code"));

    let mut expected_func: Vec<Func> = schema_funcs
        .into_iter()
        .filter(|f| f.name == func_name)
        .collect();
    assert!(!expected_func.is_empty());
    assert_eq!(func_name, expected_func.pop().unwrap().name);

    // commit changes, so DVU kicks off and we should see the outcome of the new qualification and code gen func
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");
    let component_view = initial_component
        .view(ctx)
        .await
        .expect("get component view");

    // This test confirms the code gen and qualification ran for the existing component with expected outputs
    assert_eq!(
        Some(serde_json::json!({
            "si": {
                "name": "demo component",
                "type": "component",
                "color": "#00b0b0",
            },
            "domain": {
                "testProp": "test",
                "arrayProp": [
                    "first",
                    "second",
                ]
            },
            "resource_value": {
            },
            "code": {
                "Code Gen Func": {
                    "code": "{\"domain\":{\"testProp\":\"test\",\"arrayProp\":[\"first\",\"second\"]}}",
                    "format":
                        "json",
                },
            },
            "qualification":{
                "Qualification Func": {
                    "result":"success",
                    "message":"Component qualified",
                }
            }
        })),
        component_view
    );
}
