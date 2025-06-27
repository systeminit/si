use dal::{
    DalContext,
    action::{
        Action,
        prototype::ActionPrototype,
    },
    attribute::attributes,
};
use dal_test::{
    Result,
    helpers::{
        attribute::value::{
            self,
        },
        component::{
            self,
        },
        create_component_for_default_schema_name_in_default_view,
        schema::variant,
    },
    prelude::ChangeSetTestHelpers,
    test,
};
use pretty_assertions_sorted::assert_eq;
use serde_json::json;
use si_events::{
    ActionKind,
    ActionState,
};

// Test that updating attributes sets them (and their parents) correctly, but leaves default
// values and other values alone.
#[test]
async fn update_attributes(ctx: &DalContext) -> Result<()> {
    variant::create(
        ctx,
        "test",
        r#"
            function main() {
                return {
                    props: [
                        { name: "Parent", kind: "object", children: [
                            { name: "Updated", kind: "string" },
                            { name: "New", kind: "string" },
                            { name: "Unchanged", kind: "string" },
                            { name: "Missing", kind: "string" },
                            { name: "Default", kind: "string", defaultValue: "default" },
                        ]},
                        { name: "Missing", kind: "string" },
                        { name: "Default", kind: "string", defaultValue: "default" },
                    ]
                };
            }
        "#,
    )
    .await?;

    // Set some initial values and make sure they are set correctly without messing with other
    // values or defaults.
    let component_id = component::create(ctx, "test", "test").await?;
    value::set(ctx, ("test", "/domain/Parent/Updated"), "old").await?;
    value::set(ctx, ("test", "/domain/Parent/Unchanged"), "old").await?;
    assert_eq!(
        json!({
            "Parent": {
                "Updated": "old",
                "Unchanged": "old",
                "Default": "default",
            },
            "Default": "default",
        }),
        component::domain(ctx, "test").await?
    );
    assert!(value::is_set(ctx, ("test", "/domain/Parent/Updated")).await?);
    assert!(!value::is_set(ctx, ("test", "/domain/Parent/New")).await?);
    assert!(value::is_set(ctx, ("test", "/domain/Parent/Unchanged")).await?);
    assert!(!value::is_set(ctx, ("test", "/domain/Parent/Missing")).await?);
    assert!(!value::is_set(ctx, ("test", "/domain/Parent/Default")).await?);
    assert!(!value::is_set(ctx, ("test", "/domain/Default")).await?);

    // Update values and make sure they are updated correctly without messing with other values
    // or defaults.
    attributes::update_attributes(
        ctx,
        component_id,
        serde_json::from_value(json!({
            "/domain/Parent/Updated": "new",
            "/domain/Parent/New": "new",
        }))?,
    )
    .await?;
    assert_eq!(
        json!({
            "Parent": {
                "Updated": "new",
                "New": "new",
                "Unchanged": "old",
                "Default": "default",
            },
            "Default": "default",
        }),
        component::domain(ctx, "test").await?
    );
    assert!(value::is_set(ctx, ("test", "/domain/Parent/Updated")).await?);
    assert!(value::is_set(ctx, ("test", "/domain/Parent/New")).await?);
    assert!(value::is_set(ctx, ("test", "/domain/Parent/Unchanged")).await?);
    assert!(!value::is_set(ctx, ("test", "/domain/Parent/Missing")).await?);
    assert!(!value::is_set(ctx, ("test", "/domain/Parent/Default")).await?);
    assert!(!value::is_set(ctx, ("test", "/domain/Default")).await?);

    Ok(())
}

// Test that updating an attribute value via the new subscription interface correctly enqueues
// update actions
#[test]
async fn update_attributes_enqueues_update_fn(ctx: &mut DalContext) -> Result<()> {
    // ======================================================
    // Creating a component  should enqueue a create action
    // ======================================================
    let component_jack =
        create_component_for_default_schema_name_in_default_view(ctx, "swifty", "jack antonoff")
            .await?;
    let component_swift =
        create_component_for_default_schema_name_in_default_view(ctx, "swifty", "taylor swift")
            .await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Apply changeset so it runs the creation action
    ChangeSetTestHelpers::apply_change_set_to_base(ctx).await?;

    // wait for actions to run
    ChangeSetTestHelpers::wait_for_actions_to_run(ctx).await?;

    ChangeSetTestHelpers::fork_from_head_change_set(ctx).await?;
    Action::remove_all_for_component_id(ctx, component_jack.id()).await?;

    // ======================================================
    // Updating values in a component that has a resource should enqueue an update action
    // ======================================================

    attributes::update_attributes(
        ctx,
        component_jack.id(),
        serde_json::from_value(json!({
            "/domain/name": "whomever",
        }))?,
    )
    .await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    let action_ids = Action::list_topologically(ctx).await?;

    let mut update_action_count = 0;

    for action_id in action_ids {
        let action = Action::get_by_id(ctx, action_id).await?;

        if action.state() == ActionState::Queued {
            let prototype_id = Action::prototype_id(ctx, action_id).await?;
            let prototype = ActionPrototype::get_by_id(ctx, prototype_id).await?;
            let component_id = Action::component_id(ctx, action_id)
                .await?
                .expect("is some");
            if prototype.kind == ActionKind::Update.into() && component_id == component_jack.id() {
                update_action_count += 1;
            };
        }
    }
    assert_eq!(1, update_action_count);

    // ======================================================
    // Updating values in a component that has a resource should not enqueue an update
    // action if the value didn't change
    // ======================================================

    attributes::update_attributes(
        ctx,
        component_swift.id(),
        serde_json::from_value(json!({
            "/domain/name": "taylor swift",
        }))?,
    )
    .await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;
    Action::remove_all_for_component_id(ctx, component_swift.id()).await?;

    let action_ids = Action::list_topologically(ctx).await?;

    let mut update_action_count = 0;

    for action_id in &action_ids {
        let action_id = *action_id;
        let action = Action::get_by_id(ctx, action_id).await?;
        if action.state() == ActionState::Queued {
            let prototype_id = Action::prototype_id(ctx, action_id).await?;
            let prototype = ActionPrototype::get_by_id(ctx, prototype_id).await?;
            let component_id = Action::component_id(ctx, action_id)
                .await?
                .expect("is some");
            if prototype.kind == ActionKind::Update.into() && component_id == component_swift.id() {
                update_action_count += 1;
            };
        }
    }
    // didn't actually change the value, so there should not be an update function for swifty!
    assert_eq!(0, update_action_count);

    Ok(())
}
