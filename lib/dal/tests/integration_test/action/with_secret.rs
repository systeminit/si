use dal::prop::PropPath;
use dal::property_editor::values::PropertyEditorValues;
use dal::qualification::QualificationSubCheckStatus;
use dal::{
    ActionKind, AttributeValue, ChangeSet, Component, DalContext, DeprecatedAction, InputSocket,
    OutputSocket, Prop, Secret,
};
use dal_test::test_harness::{create_component_for_schema_name, encrypt_message};
use dal_test::{test, WorkspaceSignup};
use pretty_assertions_sorted::assert_eq;

#[test]
async fn create_action_using_secret(ctx: &mut DalContext, nw: &WorkspaceSignup) {
    let source_component = create_component_for_schema_name(ctx, "dummy-secret", "source").await;
    let source_schema_variant_id = Component::schema_variant_id(ctx, source_component.id())
        .await
        .expect("could not get schema variant id for component");

    let destination_component =
        create_component_for_schema_name(ctx, "fallout", "destination").await;
    let destination_schema_variant_id =
        Component::schema_variant_id(ctx, destination_component.id())
            .await
            .expect("could not get schema variant id for component");

    // This is the name of the secret definition from the test exclusive schema.
    let secret_definition_name = "dummy";

    // Cache the prop we need for attribute value update.
    let reference_to_secret_prop = Prop::find_prop_by_path(
        ctx,
        source_schema_variant_id,
        &PropPath::new(["root", "secrets", secret_definition_name]),
    )
    .await
    .expect("could not find prop by path");

    // Connect the two components to propagate the secret value.
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

    // Create the secret and commit.
    let secret = Secret::new(
        ctx,
        "johnqt",
        secret_definition_name.to_string(),
        None,
        &encrypt_message(ctx, nw.key_pair.pk(), &serde_json::json![{"value": "todd"}]).await,
        nw.key_pair.pk(),
        Default::default(),
        Default::default(),
    )
    .await
    .expect("cannot create secret");
    let conflicts = ctx.blocking_commit().await.expect("unable to commit");
    assert!(conflicts.is_none());
    ctx.update_snapshot_to_visibility()
        .await
        .expect("unable to update snapshot to visibility");

    // Use the secret in the source component and commit.
    let property_values = PropertyEditorValues::assemble(ctx, source_component.id())
        .await
        .expect("unable to list prop values");
    let reference_to_secret_attribute_value_id = property_values
        .find_by_prop_id(reference_to_secret_prop.id)
        .expect("could not find attribute value");
    AttributeValue::update_for_secret(
        ctx,
        reference_to_secret_attribute_value_id,
        Some(secret.id()),
    )
    .await
    .expect("unable to perform attribute value update");
    let conflicts = ctx.blocking_commit().await.expect("unable to commit");
    assert!(conflicts.is_none());
    ctx.update_snapshot_to_visibility()
        .await
        .expect("unable to update snapshot to visibility");

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

    // Ensure we have our create action on the destination component.
    let mut actions = DeprecatedAction::for_component(ctx, destination_component.id())
        .await
        .expect("unable to list actions for component");
    assert_eq!(
        1,             // expected
        actions.len()  // actual
    );
    let create_action = actions.pop().expect("no actions found");
    let create_action_prototype = create_action
        .prototype(ctx)
        .await
        .expect("could not get action prototype for action");
    assert_eq!(
        ActionKind::Create,           // expected
        create_action_prototype.kind, // actual
    );

    // Ensure that the parent is head so that the "create" action will execute by default.
    // Technically, this primarily validates the test setup rather than the system itself, but it
    // serves a secondary function of ensuring no prior functions cause this assertion to fail.
    assert!(ctx
        .parent_is_head()
        .await
        .expect("could not perform parent is head"));

    // Apply to the base change set and commit.
    let applied_change_set = ChangeSet::apply_to_base_change_set(ctx, true)
        .await
        .expect("could apply to base change set");
    let conflicts = ctx.blocking_commit().await.expect("unable to commit");
    assert!(conflicts.is_none());

    // Observe that the "create" action on the destination component succeeded. We'll use the base
    // change set id to do so.
    ctx.update_visibility_and_snapshot_to_visibility_no_editing_change_set(
        applied_change_set
            .base_change_set_id
            .expect("base change set not found"),
    )
    .await
    .expect("could not update visibility and snapshot to visibility");

    assert_eq!(
        serde_json::json![{
            "qualification": {
                "test:qualificationDummySecretStringIsTodd": {
                    "message": "dummy secret string matches expected value",
                    "result": "success",
                },
            },
            "resource": {},
            "resource_value": {},
            "secret_definition": {},
            "secrets": {
                "dummy": secret.id().to_string(),
            },
            "si": {
                "color": "#ffffff",
                "name": "source",
                "type": "component",
            },
        }], // expected
        source_component
            .materialized_view(ctx)
            .await
            .expect("could not get materialized view")
            .expect("empty materialized view") // actual
    );
    let last_synced_av_id = destination_component
        .attribute_values_for_prop(ctx, &["root", "resource", "last_synced"])
        .await
        .expect("should be able to find avs for last synced")
        .pop()
        .expect("should have an av for last synced");

    let last_synced_value = AttributeValue::get_by_id(ctx, last_synced_av_id)
        .await
        .expect("should be able to get last synced av")
        .materialized_view(ctx)
        .await
        .expect("should be able to get value for last synced av");

    assert_eq!(
        serde_json::json![{
            "domain": {
                "active": true,
                "name": "destination",
            },
            "resource": {
                "last_synced": last_synced_value.unwrap_or(serde_json::Value::Null),
                "logs": [
                    "Setting dummySecretString to requestStorage",
                     "Output: {\n  \"protocol\": \"result\",\n  \"status\": \"success\",\n  \"executionId\": \"ayrtonsennajscommand\",\n  \"payload\": {\n    \"poop\": true\n  },\n  \"health\": \"ok\"\n}",
                ],
                "payload": "{\"poop\":true}",
                "status": "ok",
            },
            "resource_value": {},
            "secrets": {
                "dummy": secret.id().to_string(),
            },
            "si": {
                "color": "#ffffff",
                "name": "destination",
                "type": "component",
            },
        }], // expected
        destination_component
            .materialized_view(ctx)
            .await
            .expect("could not get materialized view")
            .expect("empty materialized view") // actual
    );
}
