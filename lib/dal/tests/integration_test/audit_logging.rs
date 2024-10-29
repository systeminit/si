use std::collections::HashSet;

use audit_logs::AuditLogsStream;
use dal::{audit_logging, Component, DalContext, Schema};
use dal_test::{
    helpers::{update_attribute_value_for_component, ChangeSetTestHelpers},
    test,
};
use pending_events::PendingEventsStream;
use si_events::audit_log::AuditLogKind;

#[test]
async fn generation_filtering_pagination(ctx: &DalContext) {
    let audit_logs = audit_logging::generate(ctx, 200)
        .await
        .expect("could not generate audit logs");
    let (filtered_and_paginated_audit_logs, _) = audit_logging::filter_and_paginate(
        audit_logs,
        Some(2),
        Some(25),
        None,
        None,
        HashSet::new(),
        HashSet::new(),
        HashSet::new(),
    )
    .expect("could not filter and paginate");
    assert_eq!(
        25,                                      // expected
        filtered_and_paginated_audit_logs.len()  // actual
    )
}

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

    // Create a component and commit. Mimic sdf by audit logging here.
    ctx.write_audit_log(AuditLogKind::CreateComponent)
        .await
        .expect("could not write audit log");
    let component = Component::new(ctx, "nyj despair club", schema_variant_id)
        .await
        .expect("could not create component");
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
    assert_eq!(
        3,
        destination_stream
            .get_info()
            .await
            .expect("could not get destination stream info")
            .state
            .messages
    );

    // Update a property editor value and commit. Mimic sdf by audit logging here.
    ctx.write_audit_log(AuditLogKind::UpdatePropertyEditorValue)
        .await
        .expect("could not write audit log");
    update_attribute_value_for_component(
        ctx,
        component.id(),
        &["root", "domain", "name"],
        serde_json::json!["pain."],
    )
    .await
    .expect("could not update attribute value");
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
    assert_eq!(
        5,
        destination_stream
            .get_info()
            .await
            .expect("could not get destination stream info")
            .state
            .messages
    );

    // Delete a component and commit. Mimic sdf by audit logging here.
    ctx.write_audit_log(AuditLogKind::DeleteComponent)
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
    assert_eq!(
        6,
        destination_stream
            .get_info()
            .await
            .expect("could not get destination stream info")
            .state
            .messages
    );
}
