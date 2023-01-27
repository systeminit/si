use dal::func::argument::{FuncArgument, FuncArgumentKind};
use dal::func::backend::js_command::CommandRunResult;

use dal::schema::variant::leaves::LeafKind;
use dal::{
    generate_name,
    schema::variant::leaves::{LeafInput, LeafInputLocation},
    ChangeSet, ChangeSetStatus, Component, ComponentView, DalContext, Func, FuncBackendKind,
    FuncBackendResponseType, SchemaVariant, StandardModel, Visibility, WorkspacePk,
};

use dal_test::test;
use dal_test::test_harness::{create_schema, create_schema_variant_with_root};
use pretty_assertions_sorted::assert_eq;
use veritech_client::ResourceStatus;

#[test]
async fn add_and_run_confirmations(mut octx: DalContext, wid: WorkspacePk) {
    let ctx = &mut octx;
    ctx.update_to_head();

    let mut schema = create_schema(ctx).await;
    let (mut schema_variant, _root_prop) = create_schema_variant_with_root(ctx, *schema.id()).await;
    let schema_variant_id = *schema_variant.id();
    schema
        .set_default_schema_variant_id(ctx, Some(schema_variant_id))
        .await
        .expect("cannot set default schema variant");

    // Setup the confirmation function.
    let mut confirmation_func = Func::new(
        ctx,
        "test:confirmation",
        FuncBackendKind::JsAttribute,
        FuncBackendResponseType::Confirmation,
    )
    .await
    .expect("could not create func");
    let confirmation_func_id = *confirmation_func.id();
    let code = "async function exists(input) {
        if (!input.resource?.value) {
            return {
                success: false,
                recommendedActions: [\"create\"]
            }
        }
        return {
            success: true,
            recommendedActions: [],
        }
    }";
    confirmation_func
        .set_code_plaintext(ctx, Some(code))
        .await
        .expect("set code");
    confirmation_func
        .set_handler(ctx, Some("exists"))
        .await
        .expect("set handler");
    let confirmation_func_argument = FuncArgument::new(
        ctx,
        "resource",
        FuncArgumentKind::String,
        None,
        confirmation_func_id,
    )
    .await
    .expect("could not create func argument");

    // Add the leaf for the confirmation.
    SchemaVariant::add_leaf(
        ctx,
        confirmation_func_id,
        schema_variant_id,
        None,
        LeafKind::Confirmation,
        vec![LeafInput {
            location: LeafInputLocation::Resource,
            func_argument_id: *confirmation_func_argument.id(),
        }],
    )
    .await
    .expect("could not add qualification");

    // Finalize the schema variant and create the component.
    schema_variant
        .finalize(ctx, None)
        .await
        .expect("unable to finalize schema variant");

    // Update to the workspace tenancy and enter a new change set.
    ctx.update_to_workspace_tenancies(wid)
        .await
        .expect("could not update to workspace tenancies");
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
            },
        )
        .await
        .expect("could not set resource");

    // Observe that the confirmation worked after "creation".
    let component_view = ComponentView::new(ctx, *component.id())
        .await
        .expect("could not generate component view");
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "component",
                "type": "component",
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
            },
        )
        .await
        .expect("could not set resource");

    // Observe that the confirmation worked after "deletion".
    let component_view = ComponentView::new(ctx, *component.id())
        .await
        .expect("could not generate component view");
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "component",
                "type": "component",
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
