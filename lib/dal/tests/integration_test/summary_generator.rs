use dal::DalContext;
use dal_summary_generator::ComponentSummaryGenerator;
use dal_test::{
    Result,
    helpers::{
        ChangeSetTestHelpers,
        attribute::value,
        component,
        schema::variant,
    },
    test,
};
use pretty_assertions_sorted::assert_eq;
use serde_json::json;

#[test(enable_veritech)]
async fn generate_component_summaries(ctx: &mut DalContext) -> Result<()> {
    // Create three variants for a subscription daisy chain.
    variant::create(
        ctx,
        "source-component",
        r#"
            function main() {
                return {
                    props: [
                        { name: "output", kind: "string" },
                    ]
                };
            }
        "#,
    )
    .await?;
    variant::create(
        ctx,
        "target-component",
        r#"
            function main() {
                return {
                    props: [
                        { name: "input", kind: "string" },
                    ]
                };
            }
        "#,
    )
    .await?;
    variant::create(
        ctx,
        "middle-component",
        r#"
            function main() {
                return {
                    props: [
                        { name: "passthrough", kind: "string" },
                    ]
                };
            }
        "#,
    )
    .await?;

    // Create a component for each variant.
    let source_id = component::create(ctx, "source-component", "source").await?;
    let middle_id = component::create(ctx, "middle-component", "middle").await?;
    let target_id = component::create(ctx, "target-component", "target").await?;

    // Set a value on the source component.
    value::set(ctx, ("source", "/domain/output"), "toddhoward").await?;

    // Set up subscriptions: middle subscribes to source and target subscribes to middle. After that, commit.
    value::subscribe(
        ctx,
        ("middle", "/domain/passthrough"),
        ("source", "/domain/output"),
    )
    .await?;
    value::subscribe(
        ctx,
        ("target", "/domain/input"),
        ("middle", "/domain/passthrough"),
    )
    .await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Verify that the subscriptions worked.
    assert_eq!(
        json!("toddhoward"),
        value::get(ctx, ("middle", "/domain/passthrough")).await?
    );
    assert_eq!(
        json!("toddhoward"),
        value::get(ctx, ("target", "/domain/input")).await?
    );

    // Create components from existing test schemas that have actions and commit.
    let component_one_id = component::create(ctx, "swifty", "component_one").await?;
    let component_two_id = component::create(ctx, "swifty", "component_two").await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Now we can apply!
    ChangeSetTestHelpers::apply_change_set_to_base(ctx).await?;
    ChangeSetTestHelpers::wait_for_actions_to_run(ctx).await?;

    // Cache the HEAD context before creating a new one. We need both.
    let head_ctx = ctx.clone();
    ChangeSetTestHelpers::fork_from_head_change_set(ctx).await?;

    // Add two more components in the new change set and commit.
    let component_three_id = component::create(ctx, "swifty", "component_three").await?;
    let component_four_id = component::create(ctx, "swifty", "component_four").await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Call the generate function with all flags enabled.
    let generator = ComponentSummaryGenerator::new(true, true, true, true, true, true, true, true);
    let summaries = generator
        .generate(
            ctx,
            &head_ctx,
            &[
                source_id,
                middle_id,
                target_id,
                component_one_id,
                component_two_id,
                component_three_id,
                component_four_id,
            ],
        )
        .await?;
    assert_eq!(7, summaries.len());

    // Fetch all sumaries.
    let source_summary = summaries
        .get(&source_id.to_string())
        .expect("source component summary not found");
    let middle_summary = summaries
        .get(&middle_id.to_string())
        .expect("middle component summary not found");
    let target_summary = summaries
        .get(&target_id.to_string())
        .expect("target component summary not found");
    let component_one_summary = summaries
        .get(&component_one_id.to_string())
        .expect("component_one summary not found");
    let component_two_summary = summaries
        .get(&component_two_id.to_string())
        .expect("component_two summary not found");
    let component_three_summary = summaries
        .get(&component_three_id.to_string())
        .expect("component_three summary not found");
    let component_four_summary = summaries
        .get(&component_four_id.to_string())
        .expect("component_four summary not found");
    dbg!(
        "relevant summaries found",
        &source_summary,
        &middle_summary,
        &target_summary,
        &component_one_summary,
        &component_two_summary,
        &component_three_summary,
        &component_four_summary,
    );

    // Verify the source component summary - middle subscribes to source
    assert_eq!("source", source_summary.component_name);
    assert_eq!("source-component", source_summary.schema_name);
    assert_eq!(1, source_summary.subscriptions.len());
    let source_sub = &source_summary.subscriptions[0];
    assert_eq!(middle_id, source_sub.to_component_id);
    assert_eq!("middle", source_sub.to_component_name);
    assert!(source_sub.from_path.contains("output"));
    assert!(source_sub.to_path.contains("passthrough"));
    assert_eq!(Some(json!("toddhoward")), source_sub.current_value);

    // Verify the middle component summary - target subscribes to middle
    assert_eq!("middle", middle_summary.component_name);
    assert_eq!("middle-component", middle_summary.schema_name);
    assert_eq!(1, middle_summary.subscriptions.len());
    let middle_sub = &middle_summary.subscriptions[0];
    assert_eq!(target_id, middle_sub.to_component_id);
    assert_eq!("target", middle_sub.to_component_name);
    assert!(middle_sub.from_path.contains("passthrough"));
    assert!(middle_sub.to_path.contains("input"));
    assert_eq!(Some(json!("toddhoward")), middle_sub.current_value);

    // Verify the target component summary - no one subscribes to target
    assert_eq!("target", target_summary.component_name);
    assert_eq!("target-component", target_summary.schema_name);
    assert_eq!(0, target_summary.subscriptions.len());

    // Verify component_one has action functions
    assert_eq!("component_one", component_one_summary.component_name);
    assert_eq!("swifty", component_one_summary.schema_name);
    assert!(
        !component_one_summary.action_functions.is_empty(),
        "component_one should have action functions"
    );

    // Verify component_two has action functions
    assert_eq!("component_two", component_two_summary.component_name);
    assert_eq!("swifty", component_two_summary.schema_name);
    assert!(
        !component_two_summary.action_functions.is_empty(),
        "component_two should have action functions"
    );

    // Verify component_three has action functions
    assert_eq!("component_three", component_three_summary.component_name);
    assert_eq!("swifty", component_three_summary.schema_name);
    assert!(
        !component_three_summary.action_functions.is_empty(),
        "component_three should have action functions"
    );

    // Verify component_four has action functions
    assert_eq!("component_four", component_four_summary.component_name);
    assert_eq!("swifty", component_four_summary.schema_name);
    assert!(
        !component_four_summary.action_functions.is_empty(),
        "component_four should have action functions"
    );

    Ok(())
}
