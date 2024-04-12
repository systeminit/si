use dal::workspace_snapshot::content_address::ContentAddressDiscriminants;
use dal::workspace_snapshot::edge_weight::EdgeWeightKindDiscriminants;
use dal::{AttributeValue, Component, DalContext};
use dal_test::test;
use dal_test::test_harness::{
    commit_and_update_snapshot, connect_components_with_socket_names,
    create_component_for_schema_name, PropEditorTestView,
};
use serde_json::json;

#[test]
async fn validation_format_errors(ctx: &mut DalContext) {
    let component = create_component_for_schema_name(ctx, "BadValidations", "bad").await;
    commit_and_update_snapshot(ctx).await;

    let bad_json_path = &["root", "domain", "bad_validation_json"];
    let prop_view = PropEditorTestView::for_component_id(ctx, component.id())
        .await
        .get_value(bad_json_path);
    assert_eq!(
        json!({
            "value": null,
            "validation": {
                "status": "Error",
                "message": "JoiValidationJsonParsingError: Unexpected token ' in JSON at position 0",
            }
        }),
        extract_value_and_validation(prop_view)
    );

    let bad_format_path = &["root", "domain", "bad_validation_format"];
    let prop_view = PropEditorTestView::for_component_id(ctx, component.id())
        .await
        .get_value(bad_format_path);

    assert_eq!(
        json!({
            "value": null,
            "validation": {
                "status": "Error",
                "message": "JoiValidationFormatError: validationFormat must be of type object",
            }
        }),
        extract_value_and_validation(prop_view)
    );
}

#[test]
async fn prop_editor_validation(ctx: &mut DalContext) {
    let component = create_component_for_schema_name(ctx, "pirate", "Robinson Crusoe").await;
    commit_and_update_snapshot(ctx).await;

    let prop_path = &["root", "domain", "working_eyes"];
    let av_id = component
        .attribute_values_for_prop(ctx, prop_path)
        .await
        .expect("find value ids for the prop")
        .pop()
        .expect("there should only be one value id");

    let prop_view = PropEditorTestView::for_component_id(ctx, component.id())
        .await
        .get_value(prop_path);

    assert_eq!(
        json!({
            "value": null,
            "validation": {
                "status": "Failure",
                "message": "\"value\" is required",
            }
        }),
        extract_value_and_validation(prop_view)
    );

    AttributeValue::update(ctx, av_id, Some(json!(1)))
        .await
        .expect("override attribute value");

    commit_and_update_snapshot(ctx).await;

    let prop_view = PropEditorTestView::for_component_id(ctx, component.id())
        .await
        .get_value(prop_path);

    assert_eq!(
        json!({
            "value": 1,
            "validation": {
                "status": "Success",
                "message": null,
            }
        }),
        extract_value_and_validation(prop_view)
    );

    AttributeValue::update(ctx, av_id, Some(json!(3)))
        .await
        .expect("override attribute value");

    commit_and_update_snapshot(ctx).await;

    let prop_view = PropEditorTestView::for_component_id(ctx, component.id())
        .await
        .get_value(prop_path);

    assert_eq!(
        json!({
            "value": 3,
            "validation": {
                "status": "Failure",
                "message": "\"value\" must be less than or equal to 2",
            }
        }),
        extract_value_and_validation(prop_view)
    );
}

#[test]
async fn validation_on_dependent_value(ctx: &mut DalContext) {
    let output_component = create_component_for_schema_name(ctx, "ValidatedOutput", "Output").await;
    let input_component = create_component_for_schema_name(ctx, "ValidatedInput", "Input").await;

    connect_components_with_socket_names(
        ctx,
        output_component.id(),
        "number",
        input_component.id(),
        "number",
    )
    .await;

    let prop_path = &["root", "domain", "a_number"];
    let av_id = output_component
        .attribute_values_for_prop(ctx, prop_path)
        .await
        .expect("find value ids for the prop")
        .pop()
        .expect("there should only be one value id");

    AttributeValue::update(ctx, av_id, Some(json!(1)))
        .await
        .expect("override attribute value");

    commit_and_update_snapshot(ctx).await;

    let source_prop_view = PropEditorTestView::for_component_id(ctx, output_component.id())
        .await
        .get_value(prop_path);
    let destination_prop_view = PropEditorTestView::for_component_id(ctx, input_component.id())
        .await
        .get_value(prop_path);

    // Check validations and values
    let source_result = extract_value_and_validation(source_prop_view);
    assert_eq!(
        json!({
            "value": 1,
            "validation": {
                "status": "Success",
                "message": null
            }

        }),
        source_result
    );

    let destination_result = extract_value_and_validation(destination_prop_view);
    assert_eq!(source_result, destination_result,);

    AttributeValue::update(ctx, av_id, Some(json!(3)))
        .await
        .expect("override attribute value");

    commit_and_update_snapshot(ctx).await;

    let source_prop_view = PropEditorTestView::for_component_id(ctx, input_component.id())
        .await
        .get_value(prop_path);
    let destination_prop_view = PropEditorTestView::for_component_id(ctx, output_component.id())
        .await
        .get_value(prop_path);

    let source_result = extract_value_and_validation(source_prop_view);
    assert_eq!(
        json!({
            "value": 3,
            "validation": {
                "status": "Failure",
                "message": "\"value\" must be less than or equal to 2"
            }

        }),
        source_result
    );

    let destination_result = extract_value_and_validation(destination_prop_view);
    assert_eq!(source_result, destination_result,);
}

#[test]
async fn multiple_changes_single_validation(ctx: &mut DalContext) {
    let component = create_component_for_schema_name(ctx, "pirate", "Robinson Crusoe").await;

    let prop_path = &["root", "domain", "working_eyes"];
    let av_id = component
        .attribute_values_for_prop(ctx, prop_path)
        .await
        .expect("find value ids for the prop")
        .pop()
        .expect("there should only be one value id");

    AttributeValue::update(ctx, av_id, Some(json!(1)))
        .await
        .expect("override attribute value");

    AttributeValue::update(ctx, av_id, Some(json!(3)))
        .await
        .expect("override attribute value");

    AttributeValue::update(ctx, av_id, Some(json!(1)))
        .await
        .expect("override attribute value");

    commit_and_update_snapshot(ctx).await;

    // There should be only one ValidationOutput node for attribute value
    {
        let validation_node_idxs = ctx
            .workspace_snapshot()
            .expect("get workspace snapshot")
            .outgoing_targets_for_edge_weight_kind(
                av_id,
                EdgeWeightKindDiscriminants::ValidationOutput,
            )
            .await
            .expect("get outgoing targets");

        assert_eq!(validation_node_idxs.len(), 1);

        let validation_node_idx = validation_node_idxs
            .first()
            .expect("Have a validation node id");

        ctx.workspace_snapshot()
            .expect("get workspace snapshot")
            .get_node_weight(*validation_node_idx)
            .await
            .expect("get validation node weight")
            .get_option_content_node_weight_of_kind(ContentAddressDiscriminants::ValidationOutput)
            .expect("find ValidationOutput node weight");
    }
}

#[test]
async fn validation_qualification(ctx: &mut DalContext) {
    let component = create_component_for_schema_name(ctx, "pirate", "Robinson Crusoe").await;

    commit_and_update_snapshot(ctx).await;

    // Get qualifications, should be failure (required)
    let validation_qualification = Component::list_qualifications(ctx, component.id())
        .await
        .expect("to get qualifications view")
        .into_iter()
        .find(|q| q.qualification_name == "validations")
        .expect("find validations qualification");

    assert_eq!(
        json!({
            "title": "Prop Validations",
            "output": [{
                "stream": "stdout",
                "line": "working_eyes: \"value\" is required",
                "level": "log",
            }],
            "description": null,
            "link": null,
            "result": {
                "status": "failure",
                "title": null,
                "link": null,
                "sub_checks": [{
                    "description": "Component has 1 invalid value(s).",
                    "status": "failure",
                }],
            },
            "qualificationName": "validations",
        }),
        serde_json::to_value(&validation_qualification).expect("serialise qualification")
    );

    // Update Value
    let prop_path = &["root", "domain", "working_eyes"];
    let av_id = component
        .attribute_values_for_prop(ctx, prop_path)
        .await
        .expect("find value ids for the prop")
        .pop()
        .expect("there should only be one value id");
    AttributeValue::update(ctx, av_id, Some(json!(1)))
        .await
        .expect("override attribute value");

    commit_and_update_snapshot(ctx).await;

    // Get qualifications, should be ok
    let validation_qualification = Component::list_qualifications(ctx, component.id())
        .await
        .expect("to get qualifications view")
        .into_iter()
        .find(|q| q.qualification_name == "validations")
        .expect("find validations qualification");

    assert_eq!(
        json!({
            "title": "Prop Validations",
            "output": [],
            "description": null,
            "link": null,
            "result": {
                "status": "success",
                "title": null,
                "link": null,
                "sub_checks": [{
                    "description": "Component has 0 invalid value(s).",
                    "status": "success",
                }],
            },
            "qualificationName": "validations",
        }),
        serde_json::to_value(validation_qualification).expect("serialise qualification")
    );
}

fn extract_value_and_validation(prop_editor_value: serde_json::Value) -> serde_json::Value {
    let value = prop_editor_value
        .get("value")
        .expect("get value from property editor value");
    let validation = prop_editor_value
        .get("validation")
        .expect("get validation from property editor value");

    json!({
        "value": value,
        "validation": validation,
    })
}
