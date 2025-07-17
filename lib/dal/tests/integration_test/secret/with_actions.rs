use dal::{
    AttributeValue,
    Component,
    DalContext,
    InputSocket,
    OutputSocket,
    Prop,
    Secret,
    Workspace,
    action::{
        Action,
        prototype::{
            ActionKind,
            ActionPrototype,
        },
    },
    prop::PropPath,
    property_editor::values::PropertyEditorValues,
    qualification::QualificationSubCheckStatus,
};
use dal_test::{
    WorkspaceSignup,
    helpers::{
        ChangeSetTestHelpers,
        create_component_for_default_schema_name_in_default_view,
        encrypt_message,
    },
    test,
};
use pretty_assertions_sorted::assert_eq;

#[test]
async fn create_action_using_secret(ctx: &mut DalContext, nw: &WorkspaceSignup) {
    // Create the components we need and commit.
    let source_component =
        create_component_for_default_schema_name_in_default_view(ctx, "dummy-secret", "source")
            .await
            .expect("could not create component");
    let source_schema_variant_id = Component::schema_variant_id(ctx, source_component.id())
        .await
        .expect("could not get schema variant id for component");
    let destination_component =
        create_component_for_default_schema_name_in_default_view(ctx, "fallout", "destination")
            .await
            .expect("could not create component");
    let destination_schema_variant_id =
        Component::schema_variant_id(ctx, destination_component.id())
            .await
            .expect("could not get schema variant id for component");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    // Cache the name of the secret definition from the test exclusive schema. Afterward, cache the
    // prop we need for attribute value update.
    let secret_definition_name = "dummy";
    let reference_to_secret_prop = Prop::find_prop_by_path(
        ctx,
        source_schema_variant_id,
        &PropPath::new(["root", "secrets", secret_definition_name]),
    )
    .await
    .expect("could not find prop by path");

    // Connect the two components to propagate the secret value and commit.
    let source_output_socket = OutputSocket::find_with_name(ctx, "dummy", source_schema_variant_id)
        .await
        .expect("could not perform find with name")
        .expect("output socket not found by name");
    let destination_input_socket =
        InputSocket::find_with_name(ctx, "dummy", destination_schema_variant_id)
            .await
            .expect("could not perform find with name")
            .expect("input socket not found by name");
    Component::connect(
        ctx,
        source_component.id(),
        source_output_socket.id(),
        destination_component.id(),
        destination_input_socket.id(),
    )
    .await
    .expect("could not connect");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    // Create the secret and commit.
    let secret = Secret::new(
        ctx,
        "johnqt",
        secret_definition_name.to_string(),
        None,
        &encrypt_message(ctx, nw.key_pair.pk(), &serde_json::json![{"value": "todd"}])
            .await
            .expect("could not encrypt message"),
        nw.key_pair.pk(),
        Default::default(),
        Default::default(),
    )
    .await
    .expect("cannot create secret");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    // Use the secret in the source component and commit.
    let property_values = PropertyEditorValues::assemble(ctx, source_component.id())
        .await
        .expect("unable to list prop values");
    let reference_to_secret_attribute_value_id = property_values
        .find_by_prop_id(reference_to_secret_prop.id)
        .expect("could not find attribute value");
    Secret::attach_for_attribute_value(
        ctx,
        reference_to_secret_attribute_value_id,
        Some(secret.id()),
    )
    .await
    .expect("could not attach secret");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    // Ensure that the qualification is successful on the source component.
    let qualifications = Component::list_qualifications(ctx, source_component.id())
        .await
        .expect("could not list qualifications");
    let qualification = qualifications
        .into_iter()
        .find(|q| q.qualification_name == "test:qualificationDummySecretStringIsTodd")
        .expect("could not find qualification");
    assert_eq!(
        QualificationSubCheckStatus::Success, // expected
        qualification.result.expect("no result found").status  // actual
    );

    // Ensure we have our "create" action on the destination component.
    let mut actions = Action::find_for_component_id(ctx, destination_component.id())
        .await
        .expect("unable to list actions for component");
    assert_eq!(
        1,             // expected
        actions.len()  // actual
    );
    let create_action_id = actions.pop().expect("no actions found");
    let create_action_prototype_id = Action::prototype_id(ctx, create_action_id)
        .await
        .expect("cannot get action prototye id");
    let create_action_prototype = ActionPrototype::get_by_id(ctx, create_action_prototype_id)
        .await
        .expect("cannot get prototype");
    assert_eq!(
        ActionKind::Create,           // expected
        create_action_prototype.kind, // actual
    );

    // set workspace token as it is currently set by interacting with the auth-api
    Workspace::get_by_pk(ctx, ctx.tenancy().workspace_pk().expect("workspace"))
        .await
        .expect("could not get workspace")
        .set_token(ctx, "token".to_string())
        .await
        .expect("could not set token");

    // Ensure that the parent is head so that the "create" action will execute by default.
    // Technically, this primarily validates the test setup rather than the system itself, but it
    // serves a secondary function of ensuring no prior functions cause this assertion to fail.
    assert!(
        ctx.parent_is_head()
            .await
            .expect("could not perform parent is head")
    );
    // Apply to the base change set and commit.
    ChangeSetTestHelpers::apply_change_set_to_base(ctx)
        .await
        .expect("could not apply change set");

    // Ensure that everything looks as expected on HEAD.
    assert_eq!(
        serde_json::json![{
            "si": {
                "color": "#ffffff",
                "name": "source",
                "type": "component",
            },
            "secrets": {
                "dummy": secret.encrypted_secret_key().to_string()
            },
            "resource_value": {},
            "qualification": {
                "test:qualificationDummySecretStringIsTodd": {
                    "result": "success",
                    "message": "dummy secret string matches expected value"
                },
            },
        }], // expected
        source_component
            .view(ctx)
            .await
            .expect("could not get materialized view")
            .expect("empty materialized view") // actual
    );

    ChangeSetTestHelpers::wait_for_actions_to_run(ctx)
        .await
        .expect("deadline for actions to run exceeded");

    let last_synced_av_id = destination_component
        .attribute_values_for_prop(ctx, &["root", "resource", "last_synced"])
        .await
        .expect("should be able to find avs for last synced")
        .pop()
        .expect("should have an av for last synced");

    let last_synced_value = AttributeValue::view(ctx, last_synced_av_id)
        .await
        .expect("should be able to get value for last synced av");

    assert_eq!(
        serde_json::json![{
            "si": {
                "color": "#ffffff",
                "name": "destination",
                "type": "component",
            },
            "domain": {
                "name": "destination",
                "active": true
            },
            "secrets": {
                "dummy": secret.encrypted_secret_key().to_string()
            },
            "resource": {
                "status": "ok",
                "payload": { "poop" :true },
                "last_synced": last_synced_value.unwrap_or(serde_json::Value::Null),
            },
            "resource_value": {}
        }], // expected
        destination_component
            .view(ctx)
            .await
            .expect("could not get materialized view")
            .expect("empty materialized view") // actual
    );
}
