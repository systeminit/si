use audit_logs::AuditLogsStream;
use dal::{audit_logging, prop::PropPath, AttributeValue, DalContext, Prop, Schema, SchemaVariant};
use dal_test::helpers::create_named_component_for_schema_variant_on_default_view;
use dal_test::{helpers::ChangeSetTestHelpers, test};
use pending_events::PendingEventsStream;
use pretty_assertions_sorted::assert_eq;
use si_events::audit_log::AuditLogKind;

#[ignore]
#[test]
async fn round_trip(ctx: &mut DalContext) {
    let schema = Schema::find_by_name(ctx, "swifty")
        .await
        .expect("could not perform find by name")
        .expect("schema not found by name");
    let schema_variant_id = schema
        .get_default_schema_variant_id(ctx)
        .await
        .expect("could not get default schema variant id")
        .expect("no default schema variant id found");
    let schema_variant = SchemaVariant::get_by_id_or_error(ctx, schema_variant_id)
        .await
        .expect("could not get schema variant");

    // Create a component and commit. Mimic sdf by audit logging here.
    let component_name = "nyj despair_club";

    let component = create_named_component_for_schema_variant_on_default_view(
        ctx,
        component_name,
        schema_variant_id,
    )
    .await
    .expect("could not create component");
    ctx.write_audit_log(
        AuditLogKind::CreateComponent {
            name: component_name.to_string(),
            component_id: component.id().into(),
            schema_variant_id: schema_variant_id.into(),
            schema_variant_name: schema_variant.display_name().to_string(),
        },
        component_name.to_string(),
    )
    .await
    .expect("could not write audit log");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    // Collect the streams needed throughout the test.
    let (source_stream, destination_stream) = {
        let source_stream_wrapper = PendingEventsStream::get_or_create(ctx.jetstream_context())
            .await
            .expect("could not get or create pending events stream");
        let destination_stream_wrapper = AuditLogsStream::get_or_create(ctx.jetstream_context())
            .await
            .expect("could not get or create audit logs stream");
        let source_stream = source_stream_wrapper
            .stream()
            .await
            .expect("could not get inner stream");
        let destination_stream = destination_stream_wrapper
            .stream()
            .await
            .expect("could not get inner destination stream");
        (source_stream, destination_stream)
    };

    // Check that the streams look as we expect.
    assert_eq!(
        0,
        source_stream
            .get_info()
            .await
            .expect("could not get source stream info")
            .state
            .messages
    );
    let first_destination_stream_message_count = destination_stream
        .get_info()
        .await
        .expect("could not get destination stream info")
        .state
        .messages;
    assert!(first_destination_stream_message_count > 0);

    // List all audit logs twice to ensure we don't consume/ack them. After that, check that they
    // look as we expect.
    let first_run_audit_logs = audit_logging::list(ctx)
        .await
        .expect("could not list audit logs");
    let second_run_audit_logs = audit_logging::list(ctx)
        .await
        .expect("could not list audit logs");
    assert_eq!(first_run_audit_logs, second_run_audit_logs);
    assert_eq!(
        first_destination_stream_message_count as usize, // expected
        first_run_audit_logs.len()                       // actual
    );

    // Update a property editor value and commit. Mimic sdf by audit logging here.
    let prop_path_raw = ["root", "domain", "name"];
    let prop = Prop::find_prop_by_path(ctx, schema_variant_id, &PropPath::new(prop_path_raw))
        .await
        .expect("could not find prop by path");
    let mut attribute_value_ids = component
        .attribute_values_for_prop(ctx, &prop_path_raw)
        .await
        .expect("could not get attribute values for prop");
    let attribute_value_id = attribute_value_ids
        .pop()
        .expect("no attribute values found");
    assert!(attribute_value_ids.is_empty());
    let before_value = AttributeValue::get_by_id(ctx, attribute_value_id)
        .await
        .expect("could not get attribute value by id")
        .value(ctx)
        .await
        .expect("could not get value for attribute value");
    let after_value = Some(serde_json::json!("pain."));
    AttributeValue::update(ctx, attribute_value_id, after_value.to_owned())
        .await
        .expect("could not update attribute value");
    ctx.write_audit_log(
        AuditLogKind::UpdatePropertyEditorValue {
            component_id: component.id().into(),
            component_name: component_name.to_string(),
            schema_variant_id: schema_variant_id.into(),
            schema_variant_display_name: schema_variant.display_name().to_string(),
            prop_id: prop.id.into(),
            prop_name: prop.name.to_owned(),
            attribute_value_id: attribute_value_id.into(),
            before_value,
            after_value,
        },
        component_name.to_string(),
    )
    .await
    .expect("could not write audit log");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    // Check that the streams look as we expect.
    assert_eq!(
        0,
        source_stream
            .get_info()
            .await
            .expect("could not get source stream info")
            .state
            .messages
    );
    let second_destination_stream_message_count = destination_stream
        .get_info()
        .await
        .expect("could not get destination stream info")
        .state
        .messages;
    assert!(second_destination_stream_message_count > first_destination_stream_message_count);

    // List all audit logs twice to ensure we don't consume/ack them. After that, check that they
    // look as we expect.
    let first_run_audit_logs = audit_logging::list(ctx)
        .await
        .expect("could not list audit logs");
    let second_run_audit_logs = audit_logging::list(ctx)
        .await
        .expect("could not list audit logs");
    assert_eq!(first_run_audit_logs, second_run_audit_logs);
    assert_eq!(
        second_destination_stream_message_count as usize, // expected
        first_run_audit_logs.len()                        // actual
    );

    // Delete a component and commit. Mimic sdf by audit logging here.
    ctx.write_audit_log(
        AuditLogKind::DeleteComponent {
            name: component_name.to_string(),
            component_id: component.id().into(),
            schema_variant_id: schema_variant_id.into(),
            schema_variant_name: schema_variant.display_name().to_string(),
        },
        component_name.to_string(),
    )
    .await
    .expect("could not write audit log");
    assert!(component
        .delete(ctx)
        .await
        .expect("unable to delete component")
        .is_none());
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    // Check that the streams look as we expect.
    assert_eq!(
        0,
        source_stream
            .get_info()
            .await
            .expect("could not get source stream info")
            .state
            .messages
    );
    let third_destination_stream_message_count = destination_stream
        .get_info()
        .await
        .expect("could not get destination stream info")
        .state
        .messages;
    assert!(third_destination_stream_message_count > second_destination_stream_message_count);

    // List all audit logs twice to ensure we don't consume/ack them. After that, check that they
    // look as we expect.
    let first_run_audit_logs = audit_logging::list(ctx)
        .await
        .expect("could not list audit logs");
    let second_run_audit_logs = audit_logging::list(ctx)
        .await
        .expect("could not list audit logs");
    assert_eq!(first_run_audit_logs, second_run_audit_logs);
    assert_eq!(
        third_destination_stream_message_count as usize, // expected
        first_run_audit_logs.len()                       // actual
    );
}
