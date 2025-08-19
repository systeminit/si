use std::time::Duration;

use dal::{
    AttributeValue,
    ChangeSet,
    Component,
    DalContext,
    workspace_snapshot::{
        content_address::ContentAddressDiscriminants,
        edge_weight::EdgeWeightKindDiscriminants,
    },
};
use dal_test::{
    Result,
    expected::ExpectSchemaVariant,
    helpers::{
        ChangeSetTestHelpers,
        PropEditorTestView,
        component::find_management_prototype,
        connect_components_with_socket_names,
        create_component_for_default_schema_name_in_default_view,
        extract_value_and_validation,
    },
    test,
};
use pretty_assertions_sorted::assert_eq;
use serde_json::json;

#[test]
async fn validation_format_errors(ctx: &mut DalContext) -> Result<()> {
    let component =
        create_component_for_default_schema_name_in_default_view(ctx, "BadValidations", "bad")
            .await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    let bad_json_path = &["root", "domain", "bad_validation_json"];
    let prop_view = PropEditorTestView::for_component_id(ctx, component.id())
        .await?
        .get_value(bad_json_path)?;
    assert_eq!(
        json!({
            "value": null,
            "validation": {
                "status": "Error",
                "message": "UserCodeException: Invalid JSON format",
            }
        }),
        extract_value_and_validation(prop_view)?
    );

    let bad_format_path = &["root", "domain", "bad_validation_format"];
    let prop_view = PropEditorTestView::for_component_id(ctx, component.id())
        .await?
        .get_value(bad_format_path)?;

    assert_eq!(
        json!({
            "value": null,
            "validation": {
                "status": "Error",
                "message": "UserCodeException: validationFormat 5 is wrong: ValidationError: \"value\" must be of type object",
            }
        }),
        extract_value_and_validation(prop_view)?
    );

    Ok(())
}

#[test]
async fn prop_editor_validation(ctx: &mut DalContext) -> Result<()> {
    let component =
        create_component_for_default_schema_name_in_default_view(ctx, "pirate", "Robinson Crusoe")
            .await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    let prop_path = &["root", "domain", "working_eyes"];
    let av_id = component
        .attribute_values_for_prop(ctx, prop_path)
        .await?
        .pop()
        .expect("there should only be one value id");

    let prop_view = PropEditorTestView::for_component_id(ctx, component.id())
        .await?
        .get_value(prop_path)?;

    assert_eq!(
        json!({
            "value": null,
            "validation": {
                "status": "Failure",
                "message": "\"value\" is required",
            }
        }),
        extract_value_and_validation(prop_view)?
    );

    AttributeValue::update(ctx, av_id, Some(json!(1))).await?;

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    let prop_view = PropEditorTestView::for_component_id(ctx, component.id())
        .await?
        .get_value(prop_path)?;

    assert_eq!(
        json!({
            "value": 1,
            "validation": {
                "status": "Success",
                "message": null,
            }
        }),
        extract_value_and_validation(prop_view)?
    );

    AttributeValue::update(ctx, av_id, Some(json!(3))).await?;

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    let prop_view = PropEditorTestView::for_component_id(ctx, component.id())
        .await?
        .get_value(prop_path)?;

    assert_eq!(
        json!({
            "value": 3,
            "validation": {
                "status": "Failure",
                "message": "\"value\" must be less than or equal to 2",
            }
        }),
        extract_value_and_validation(prop_view)?
    );

    Ok(())
}

#[test]
async fn validation_pre_post_mgmt_func(ctx: &mut DalContext) -> Result<()> {
    let component =
        create_component_for_default_schema_name_in_default_view(ctx, "ValidatedOutput", "Output")
            .await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // check default post create - should be one failure because a_number is required
    let component_in_list =
        dal_materialized_views::component::assemble(ctx.clone(), component.id()).await?;
    assert!(component_in_list.qualification_totals.failed == 1);

    // now let's run the mgmt func that should fix this one
    let management_prototype =
        find_management_prototype(ctx, component.id(), "Good import validated output").await?;

    ChangeSetTestHelpers::enqueue_management_func_job(
        ctx,
        management_prototype.id(),
        component.id(),
        None,
    )
    .await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Wait for dvu
    ChangeSet::wait_for_dvu(ctx, false).await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Check that the validation is now passing as the mgmt func wrote a valid value
    // loop to wait for the validation job to finish
    let seconds = 10;
    let mut did_pass = false;
    for _ in 0..(seconds * 10) {
        ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

        let component_in_list =
            dal_materialized_views::component::assemble(ctx.clone(), component.id()).await?;

        if component_in_list.qualification_totals.failed == 0 {
            did_pass = true;
            break;
        }

        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    if !did_pass {
        panic!(
            "Validation job should have finished and set the value correctly, but it did not. Must investigate!"
        );
    }

    // now let's run the bad import mgmt func (that sets a non-valid value)
    let bad_management_prototype =
        find_management_prototype(ctx, component.id(), "Bad import validated output").await?;

    ChangeSetTestHelpers::enqueue_management_func_job(
        ctx,
        bad_management_prototype.id(),
        component.id(),
        None,
    )
    .await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;
    // Wait for dvu
    ChangeSet::wait_for_dvu(ctx, false).await?;

    // now check that the validation is failing again
    // loop to wait for the validation job to finish
    let seconds = 10;
    let mut did_pass = false;
    for _ in 0..(seconds * 10) {
        ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

        let component_in_list =
            dal_materialized_views::component::assemble(ctx.clone(), component.id()).await?;

        if component_in_list.qualification_totals.failed == 1 {
            did_pass = true;
            break;
        }

        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    if !did_pass {
        panic!(
            "Validation job should have finished and set the value correctly, but it did not. Must investigate!"
        );
    }

    Ok(())
}

#[ignore]
#[test]
async fn validation_on_dependent_value(ctx: &mut DalContext) -> Result<()> {
    let output_component =
        create_component_for_default_schema_name_in_default_view(ctx, "ValidatedOutput", "Output")
            .await?;
    let input_component =
        create_component_for_default_schema_name_in_default_view(ctx, "ValidatedInput", "Input")
            .await?;

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    connect_components_with_socket_names(
        ctx,
        output_component.id(),
        "number",
        input_component.id(),
        "number",
    )
    .await?;

    let prop_path = &["root", "domain", "a_number"];
    let av_id = output_component
        .attribute_values_for_prop(ctx, prop_path)
        .await?
        .pop()
        .expect("there should only be one value id");

    AttributeValue::update(ctx, av_id, Some(json!(1))).await?;

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    let source_prop_view = PropEditorTestView::for_component_id(ctx, output_component.id())
        .await?
        .get_value(prop_path)?;
    let destination_prop_view = PropEditorTestView::for_component_id(ctx, input_component.id())
        .await?
        .get_value(prop_path)?;

    // Check validations and values
    let source_result = extract_value_and_validation(source_prop_view)?;
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

    let destination_result = extract_value_and_validation(destination_prop_view)?;
    assert_eq!(source_result, destination_result,);

    AttributeValue::update(ctx, av_id, Some(json!(3))).await?;

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    let source_prop_view = PropEditorTestView::for_component_id(ctx, input_component.id())
        .await?
        .get_value(prop_path)?;
    let destination_prop_view = PropEditorTestView::for_component_id(ctx, output_component.id())
        .await?
        .get_value(prop_path)?;

    let source_result = extract_value_and_validation(source_prop_view)?;
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

    let destination_result = extract_value_and_validation(destination_prop_view)?;
    assert_eq!(source_result, destination_result,);

    Ok(())
}

#[test]
async fn multiple_changes_single_validation(ctx: &mut DalContext) -> Result<()> {
    let component =
        create_component_for_default_schema_name_in_default_view(ctx, "pirate", "Robinson Crusoe")
            .await?;

    let prop_path = &["root", "domain", "working_eyes"];
    let av_id = component
        .attribute_values_for_prop(ctx, prop_path)
        .await?
        .pop()
        .expect("there should only be one value id");

    AttributeValue::update(ctx, av_id, Some(json!(1))).await?;

    AttributeValue::update(ctx, av_id, Some(json!(3))).await?;

    AttributeValue::update(ctx, av_id, Some(json!(1))).await?;

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // There should be only one ValidationOutput node for attribute value
    {
        let validation_node_idxs = ctx
            .workspace_snapshot()?
            .outgoing_targets_for_edge_weight_kind(
                av_id,
                EdgeWeightKindDiscriminants::ValidationOutput,
            )
            .await?;

        assert_eq!(validation_node_idxs.len(), 1);

        let validation_node_idx = validation_node_idxs
            .first()
            .expect("Have a validation node id");

        ctx.workspace_snapshot()?
            .get_node_weight(*validation_node_idx)
            .await?
            .get_option_content_node_weight_of_kind(ContentAddressDiscriminants::ValidationOutput)
            .expect("find ValidationOutput node weight");
    }

    Ok(())
}

#[test]
async fn validation_qualification(ctx: &mut DalContext) -> Result<()> {
    let component =
        create_component_for_default_schema_name_in_default_view(ctx, "pirate", "Robinson Crusoe")
            .await?;

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Get qualifications, should be failure (required)
    let validation_qualification = Component::list_qualifications(ctx, component.id())
        .await?
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
            "finalized": true,
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
        serde_json::to_value(&validation_qualification)?
    );

    // Update Value
    let prop_path = &["root", "domain", "working_eyes"];
    let av_id = component
        .attribute_values_for_prop(ctx, prop_path)
        .await?
        .pop()
        .expect("there should only be one value id");
    AttributeValue::update(ctx, av_id, Some(json!(1))).await?;

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Get qualifications, should be ok
    let validation_qualification = Component::list_qualifications(ctx, component.id())
        .await?
        .into_iter()
        .find(|q| q.qualification_name == "validations")
        .expect("find validations qualification");

    assert_eq!(
        json!({
            "title": "Prop Validations",
            "output": [],
            "finalized": true,
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
        serde_json::to_value(validation_qualification)?
    );

    Ok(())
}

#[test]
async fn required_unset_value(ctx: &mut DalContext) -> Result<()> {
    ExpectSchemaVariant::create_named(
        ctx,
        "required_value",
        r#"
            function main() {
                return new AssetBuilder()
                    .addProp(new PropBuilder()
                        .setName("Value")
                        .setKind("string")
                        .setValidationFormat(Joi.string().required())
                        .build()
                    )
                    .build();
            }
        "#,
    )
    .await;

    let component = create_component_for_default_schema_name_in_default_view(
        ctx,
        "required_value",
        "required_value",
    )
    .await?;

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Get qualifications, should be failure (required)
    let validation_qualification = Component::list_qualifications(ctx, component.id())
        .await?
        .into_iter()
        .find(|q| q.qualification_name == "validations")
        .expect("find validations qualification");

    assert_eq!(
        json!({
            "title": "Prop Validations",
            "output": [{
                "stream": "stdout",
                "line": "Value: \"value\" is required",
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
            "finalized": true,
        }),
        serde_json::to_value(&validation_qualification)?
    );
    Ok(())
}

#[test]
async fn required_default_value(ctx: &mut DalContext) -> Result<()> {
    ExpectSchemaVariant::create_named(
        ctx,
        "required_default",
        r#"
            function main() {
                return new AssetBuilder()
                    .addProp(new PropBuilder()
                        .setName("Value")
                        .setKind("string")
                        .setDefaultValue("ok")
                        .setValidationFormat(Joi.string().required())
                        .build()
                    )
                    .build();
            }
        "#,
    )
    .await;

    let component = create_component_for_default_schema_name_in_default_view(
        ctx,
        "required_default",
        "required_default",
    )
    .await?;

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Get qualifications, should be success (not required)
    let validation_qualification = Component::list_qualifications(ctx, component.id())
        .await?
        .into_iter()
        .find(|q| q.qualification_name == "validations")
        .expect("find validations qualification");

    assert_eq!(
        json!({
            "title": "Prop Validations",
            "output": [],
            "finalized": true,
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
        serde_json::to_value(&validation_qualification)?
    );
    Ok(())
}
