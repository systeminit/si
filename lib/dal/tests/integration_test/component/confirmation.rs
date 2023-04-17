use dal::func::backend::js_command::CommandRunResult;
use dal::job::definition::{FixItem, FixesJob};

use dal::{
    component::confirmation::view::ConfirmationStatus, generate_name, ChangeSet, ChangeSetStatus,
    Component, ComponentView, ComponentViewProperties, DalContext, Fix, FixBatch,
    FixCompletionStatus, Schema, StandardModel, Visibility,
};
use dal_test::test;
use pretty_assertions_sorted::assert_eq;
use veritech_client::ResourceStatus;

/// Recommendation: run this test with the following environment variable:
/// ```shell
/// SI_TEST_BUILTIN_SCHEMAS=test
/// ```
#[test]
async fn add_and_run_confirmations(mut octx: DalContext) {
    let ctx = &mut octx;
    ctx.update_to_head();

    let schema_variant_id = *Schema::find_by_name(ctx, "starfield")
        .await
        .expect("could not find schema")
        .default_schema_variant_id()
        .expect("could not get default variant id");

    let new_change_set = ChangeSet::new(ctx, generate_name(), None)
        .await
        .expect("could not create new change set");
    ctx.update_visibility(Visibility::new(new_change_set.pk, None));

    // Create a component and immediately apply the change set.
    let (component, _) = Component::new(ctx, "component", schema_variant_id)
        .await
        .expect("cannot create component");

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");
    assert_eq!(new_change_set.pk, ctx.visibility().change_set_pk);
    let mut change_set = ChangeSet::get_by_pk(ctx, &ctx.visibility().change_set_pk)
        .await
        .expect("could not fetch change set by pk")
        .expect("no change set found for pk");
    change_set
        .apply(ctx)
        .await
        .expect("cannot apply change set");

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    assert_eq!(&change_set.status, &ChangeSetStatus::Applied);
    ctx.update_visibility(Visibility::new_head(false));

    // Observe that the confirmation failed.
    let component_view = ComponentView::new(ctx, *component.id())
        .await
        .expect("could not generate component view");
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "component",
                "type": "component",
                "color": "#ffffff",
                "protected": false
            },
            "domain": {},
            "confirmation": {
                "test:confirmation": {
                    "success": false,
                    "recommendedActions": ["create"]
                }
            }
        }], // expected
        component_view.properties // actual
    );

    // "Create" the resource.
    component
        .set_resource(
            ctx,
            CommandRunResult {
                status: ResourceStatus::Ok,
                value: Some(serde_json::json!["poop"]),
                message: None,
                logs: vec![],
                last_synced: Default::default(),
            },
            true,
        )
        .await
        .expect("could not set resource");

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    // Observe that the confirmation worked after "creation".
    let component_view = ComponentView::new(ctx, *component.id())
        .await
        .expect("could not generate component view");
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "component",
                "type": "component",
                "color": "#ffffff",
                "protected": false
            },
            "domain": {},
            "resource": {
                "logs": [],
                "value": "poop",
                "status": "ok"
            },
            "confirmation": {
                "test:confirmation": {
                    "success": true,
                    "recommendedActions": []
                }
            }
        }], // expected
        component_view.properties // actual
    );

    // "Delete" the resource.
    component
        .set_resource(
            ctx,
            CommandRunResult {
                status: ResourceStatus::Ok,
                value: None,
                message: None,
                logs: vec![],
                last_synced: Default::default(),
            },
            true,
        )
        .await
        .expect("could not set resource");

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    // Observe that the confirmation worked after "deletion".
    let component_view = ComponentView::new(ctx, *component.id())
        .await
        .expect("could not generate component view");
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "component",
                "type": "component",
                "color": "#ffffff",
                "protected": false
            },
            "domain": {},
            "resource": {
                "logs": [],
                "status": "ok"
            },
            "confirmation": {
                "test:confirmation": {
                    "success": false,
                    "recommendedActions": ["create"]
                }
            }
        }], // expected
        component_view.properties // actual
    );
}

/// Recommendation: run this test with the following environment variable:
/// ```shell
/// SI_TEST_BUILTIN_SCHEMAS=test
/// ```
#[test]
async fn list_confirmations(mut octx: DalContext) {
    let ctx = &mut octx;
    ctx.update_to_head();

    let schema_variant_id = *Schema::find_by_name(ctx, "starfield")
        .await
        .expect("could not find schema")
        .default_schema_variant_id()
        .expect("could not get default variant id");

    let new_change_set = ChangeSet::new(ctx, generate_name(), None)
        .await
        .expect("could not create new change set");
    ctx.update_visibility(Visibility::new(new_change_set.pk, None));

    // Create a component and immediately apply the change set.
    let (component, _) = Component::new(ctx, "component", schema_variant_id)
        .await
        .expect("cannot create component");

    assert_eq!(new_change_set.pk, ctx.visibility().change_set_pk);
    let mut change_set = ChangeSet::get_by_pk(ctx, &ctx.visibility().change_set_pk)
        .await
        .expect("could not fetch change set by pk")
        .expect("no change set found for pk");
    change_set
        .apply(ctx)
        .await
        .expect("cannot apply change set");

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    assert_eq!(&change_set.status, &ChangeSetStatus::Applied);
    ctx.update_visibility(Visibility::new_head(false));

    // List confirmations.
    let (confirmations, mut recommendations) = Component::list_confirmations(ctx)
        .await
        .expect("could not list confirmations");
    assert_eq!(
        1,                   // expected
        confirmations.len()  // actual
    );
    let recommendation = recommendations.pop().expect("recommendations are empty");

    // Check that there is only one recommendation and that it looks as expected.
    assert!(recommendations.is_empty());
    assert_eq!(
        "create",                           // expected
        &recommendation.recommended_action  // actual
    );
    assert_eq!(
        false,                          // expected
        recommendation.has_running_fix  // actual
    );
    assert_eq!(
        None,                    // expected
        recommendation.last_fix  // actual
    );

    // Observe that the confirmation "failed" (i.e. did not fail execution, but returned
    // with an unsuccessful result).
    let component_view = ComponentView::new(ctx, *component.id())
        .await
        .expect("could not generate component view");
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "component",
                "type": "component",
                "color": "#ffffff",
                "protected": false
            },
            "domain": {},
            "confirmation": {
                "test:confirmation": {
                    "success": false,
                    "recommendedActions": ["create"]
                }
            }
        }], // expected
        component_view.properties // actual
    );

    // Run the fix from our recommendation.
    let batch = FixBatch::new(ctx, "toddhoward@systeminit.com")
        .await
        .expect("could not create fix batch");
    let fix = Fix::new(
        ctx,
        *batch.id(),
        recommendation.confirmation_attribute_value_id,
        recommendation.component_id,
        &recommendation.recommended_action,
    )
    .await
    .expect("could not create fix");
    let fixes = vec![FixItem {
        id: *fix.id(),
        attribute_value_id: recommendation.confirmation_attribute_value_id,
        component_id: recommendation.component_id,
        action: recommendation.recommended_action,
    }];
    ctx.enqueue_job(FixesJob::new(ctx, fixes, *batch.id()))
        .await
        .expect("failed to enqueue job");

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    // Ensure that our confirmations views look as intended. We should have exactly zero
    // recommendations!
    let (mut confirmations, recommendations) = Component::list_confirmations(ctx)
        .await
        .expect("could not list confirmations");
    let confirmation = confirmations.pop().expect("views are empty");
    assert!(confirmations.is_empty());
    assert_eq!(confirmation.status, ConfirmationStatus::Success);
    assert!(recommendations.is_empty());

    // Observe that the confirmation worked after "creation".
    let component_view = ComponentView::new(ctx, *component.id())
        .await
        .expect("could not generate component view");

    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "component",
                "type": "component",
                "color": "#ffffff",
                "protected": false
            },
            "domain": {},
            "resource": {
                "logs": [],
                "value": "poop",
                "status": "ok"
            },
            "confirmation": {
                "test:confirmation": {
                    "success": true,
                    "recommendedActions": []
                }
            }
        }], // expected
        ComponentViewProperties::try_from(component_view)
            .expect("could not create component view properties")
            .drop_resource_last_synced()
            .to_value()
            .expect("could not convert to value") // actual
    );

    // "Delete" the resource. This is similar to deleting the resource in the "real world" and
    // waiting for a resource sync, but the model has remained the same.
    component
        .set_resource(
            ctx,
            CommandRunResult {
                status: ResourceStatus::Ok,
                value: None,
                message: None,
                logs: vec![],
                last_synced: Default::default(),
            },
            true,
        )
        .await
        .expect("could not set resource");

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    // List confirmations.
    let (confirmations, mut recommendations) = Component::list_confirmations(ctx)
        .await
        .expect("could not list confirmations");
    assert_eq!(
        1,                   // expected
        confirmations.len()  // actual
    );
    let recommendation = recommendations.pop().expect("recommendations are empty");

    // Check that there is only one recommendation and that it looks as expected.
    assert!(recommendations.is_empty());
    assert_eq!(
        "create",                           // expected
        &recommendation.recommended_action  // actual
    );
    assert_eq!(
        false,                          // expected
        recommendation.has_running_fix  // actual
    );
    assert_eq!(
        FixCompletionStatus::Success, // expected
        recommendation
            .last_fix
            .expect("last fix not found")
            .status()  // actual
    );

    // Observe that the confirmation worked after "deletion". The component should be re-creatable.
    // The "success" field should be "false" because we have at least one recommended action (in
    // this case, it should be exactly one).
    let component_view = ComponentView::new(ctx, *component.id())
        .await
        .expect("could not generate component view");
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "component",
                "type": "component",
                "color": "#ffffff",
                "protected": false
            },
            "domain": {},
            "resource": {
                "logs": [],
                "status": "ok"
            },
            "confirmation": {
                "test:confirmation": {
                    "success": false,
                    "recommendedActions": ["create"]
                }
            }
        }], // expected
        component_view.properties // actual
    );
}
