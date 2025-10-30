use dal::{
    DalContext,
    Func,
    InputSocket,
    Prop,
    SchemaVariant,
    func::{
        argument::FuncArgument,
        authoring::FuncAuthoringClient,
        binding::{
            AttributeFuncArgumentSource,
            AttributeFuncDestination,
            EventualParent,
            FuncBinding,
        },
        leaf::{
            LeafInputLocation,
            LeafKind,
        },
    },
};
use dal_test::{
    helpers::ChangeSetTestHelpers,
    test,
};
use pretty_assertions_sorted::assert_eq;

mod binding;
mod create_func;
mod func_argument;
mod save_and_exec;
mod save_func;
mod test_execute;

#[test]
async fn create_unlocked_copy_code_gen_func(ctx: &mut DalContext) {
    // grab an existing attribute func
    let string_codegen_fn_name = "test:generateStringCode";
    let func_id = Func::find_id_by_name(ctx, string_codegen_fn_name)
        .await
        .expect("found func")
        .expect("is some");

    // get the existing bindings
    let mut existing_bindings = FuncBinding::get_code_gen_bindings_for_func_id(ctx, func_id)
        .await
        .expect("got existing bindings");
    // make sure the bindings are correct
    assert_eq!(
        1,                       // expected
        existing_bindings.len()  // actual
    );

    let code_gen_binding = existing_bindings.pop().expect("has one binding");

    assert_eq!(
        LeafKind::CodeGeneration,   // expected
        code_gen_binding.leaf_kind  // actual
    );
    assert_eq!(
        LeafInputLocation::Domain,                                   // expected
        *code_gen_binding.inputs.first().expect("has a leaf input")  // actual
    );

    let new_display_name = Some("woo hoo".to_string());

    // try and change something, this fails because the function is locked on import!
    let res = FuncAuthoringClient::update_func(ctx, func_id, new_display_name, None).await;

    assert!(res.is_err());

    // create an unlocked copy
    let new_func = FuncAuthoringClient::create_unlocked_func_copy(ctx, func_id, None)
        .await
        .expect("could create unlocked copy");

    // new func has one binding
    // get the existing bindings
    let mut new_bindings = FuncBinding::get_code_gen_bindings_for_func_id(ctx, new_func.id)
        .await
        .expect("got new bindings");
    // make sure the bindings are correct
    assert_eq!(
        1,                  // expected
        new_bindings.len()  // actual
    );

    let code_gen_binding = new_bindings.pop().expect("has one binding");

    assert_eq!(
        LeafKind::CodeGeneration,   // expected
        code_gen_binding.leaf_kind  // actual
    );
    assert_eq!(
        LeafInputLocation::Domain,                                   // expected
        *code_gen_binding.inputs.first().expect("has a leaf input")  // actual
    );
}

#[test]
async fn create_unlocked_copy_attribute_func(ctx: &mut DalContext) {
    // grab an existing attribute func

    let fn_name = "test:falloutEntriesToGalaxies";
    let func_id = Func::find_id_by_name(ctx, fn_name)
        .await
        .expect("found func")
        .expect("is some");

    // there is one existing binding
    let mut existing_bindings = FuncBinding::get_attribute_bindings_for_func_id(ctx, func_id)
        .await
        .expect("got existing bindings");

    assert_eq!(
        1,                       // expected
        existing_bindings.len()  // actual
    );
    let attribute_binding = existing_bindings.pop().expect("has one attribute binding");

    let output_location = attribute_binding.output_location;
    // output location is a prop with name "galaxies"
    let AttributeFuncDestination::Prop(prop_output) = output_location else {
        panic!("output location is wrong");
    };
    let prop_name = Prop::get_by_id(ctx, prop_output)
        .await
        .expect("prop exists");
    assert_eq!(
        "galaxies".to_owned(), // expected
        prop_name.name         // actual
    );
    assert_eq!(
        1,                                         // expected
        attribute_binding.argument_bindings.len()  // actual
    );
    // one argument, input source is an input socket name "fallout"
    let arg_binding = attribute_binding
        .argument_bindings
        .first()
        .expect("has an argument");
    let AttributeFuncArgumentSource::InputSocket(input) = arg_binding.attribute_func_input_location
    else {
        panic!("input source is wrong");
    };

    let input_socket = InputSocket::get_by_id(ctx, input)
        .await
        .expect("found input socket");

    assert_eq!(
        "fallout",           // expected
        input_socket.name()  // actual
    );

    // argument name is "entries"
    let func_arg_name = FuncArgument::get_by_id(ctx, arg_binding.func_argument_id)
        .await
        .expect("found func arg");

    assert_eq!(
        "entries",          // expected
        func_arg_name.name  // actual
    );

    // attribute is a the schema variant level, and it's currently locked
    let EventualParent::SchemaVariant(sv_id) = attribute_binding.eventual_parent else {
        panic!("expect parent to be for schema variant");
    };

    let sv = SchemaVariant::get_by_id(ctx, sv_id).await.expect("has sv");
    assert!(sv.is_locked());

    // let's try to edit the func, this will fail because it's currently locked

    let new_display_name = Some("woo hoo".to_string());

    let res = FuncAuthoringClient::update_func(ctx, func_id, new_display_name, None).await;

    assert!(res.is_err());

    // now let's unlock the func
    let new_func = FuncAuthoringClient::create_unlocked_func_copy(ctx, func_id, None)
        .await
        .expect("could create copy");

    let new_func_id = new_func.id;

    // let's make sure it's all good

    // let all_new = FuncBinding::get_attribute_bindings_for_func_id(ctx, new_func_id)
    //     .await
    //     .expect("got unlocked latest");
    // dbg!(&all_new);

    // let all_old = FuncBinding::get_attribute_bindings_for_func_id(ctx, func_id)
    //     .await
    //     .expect("got all bindings");
    // dbg!(&all_old);

    // check all bindings and make sure everything checks out!
    // get the new bindings
    let mut new_bindings = FuncBinding::get_attribute_bindings_for_func_id(ctx, new_func_id)
        .await
        .expect("found new bindings");

    // new func has one binding for the newly unlocked variant!
    assert_eq!(
        1,                  // expected
        new_bindings.len()  // actual
    );

    let attribute_binding = new_bindings.pop().expect("has one attribute binding");

    let output_location = attribute_binding.output_location;
    // output location is a prop
    let AttributeFuncDestination::Prop(prop_output) = output_location else {
        panic!("output location is wrong");
    };
    // prop name is galaxies
    let prop_name = Prop::get_by_id(ctx, prop_output)
        .await
        .expect("prop exists");
    assert_eq!(
        "galaxies".to_owned(), // expected
        prop_name.name         // actual
    );
    // one arg binding to an input socket named "fallout"
    assert_eq!(
        1,                                         // expected
        attribute_binding.argument_bindings.len()  // actual
    );
    let arg_binding = attribute_binding
        .argument_bindings
        .first()
        .expect("has an argument");
    let AttributeFuncArgumentSource::InputSocket(input) = arg_binding.attribute_func_input_location
    else {
        panic!("input source is wrong");
    };

    let input_socket = InputSocket::get_by_id(ctx, input)
        .await
        .expect("found input socket");

    assert_eq!(
        "fallout",           // expected
        input_socket.name()  // actual
    );
    // func arg name is "entries"
    let func_arg_name = FuncArgument::get_by_id(ctx, arg_binding.func_argument_id)
        .await
        .expect("found func arg");

    assert_eq!(
        "entries",          // expected
        func_arg_name.name  // actual
    );
}

#[test]
async fn create_unlocked_copy_auth_func(ctx: &mut DalContext) {
    let fn_name = "test:setDummySecretString";
    let func_id = Func::find_id_by_name(ctx, fn_name)
        .await
        .expect("found auth func")
        .expect("has a func");
    // get the existing bindings
    let mut existing_bindings = FuncBinding::get_auth_bindings_for_func_id(ctx, func_id)
        .await
        .expect("got existing bindings");
    // make sure the bindings are correct
    assert_eq!(
        1,                       // expected
        existing_bindings.len()  // actual
    );

    existing_bindings.pop().expect("has one binding");

    let new_display_name = Some("woo hoo".to_string());

    // try and change something, this fails because the function is locked on import!
    let res = FuncAuthoringClient::update_func(ctx, func_id, new_display_name, None).await;

    assert!(res.is_err());
    // create an unlocked copy
    let new_func = FuncAuthoringClient::create_unlocked_func_copy(ctx, func_id, None)
        .await
        .expect("could create unlocked copy");

    // new func has one binding
    // get the existing bindings
    let new_bindings = FuncBinding::get_auth_bindings_for_func_id(ctx, new_func.id)
        .await
        .expect("got new bindings");
    // make sure the bindings are correct
    assert_eq!(
        1,                  // expected
        new_bindings.len()  // actual
    );
}

#[test]
async fn create_unlocked_func_and_check_locked_on_apply(ctx: &mut DalContext) {
    let fn_name = "test:setDummySecretString";
    let func_id = Func::find_id_by_name(ctx, fn_name)
        .await
        .expect("found auth func")
        .expect("has a func");

    // create an unlocked copy
    let new_func = FuncAuthoringClient::create_unlocked_func_copy(ctx, func_id, None)
        .await
        .expect("could create unlocked copy");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("unable to commit");

    let func_id = new_func.id;
    let new_code = "async function auth(secret: Input): Promise<Output> { requestStorage.setItem('dummySecretString', secret.value); requestStorage.setItem('workspaceTokens', secret.WorkspaceToken);}";
    let res = FuncAuthoringClient::save_code(ctx, new_func.id, new_code.to_string()).await;
    assert!(res.is_ok());

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("unable to commit");

    new_func.lock(ctx).await.expect("unable to lock the func");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("unable to commit");

    // Apply to HEAD
    ChangeSetTestHelpers::apply_change_set_to_base(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    let func = Func::get_by_id(ctx, func_id)
        .await
        .expect("can't find the new func");

    assert!(func.is_locked);
}
