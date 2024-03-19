use dal::{Action, ActionPrototype, DalContext};
use dal_test::test;
use dal_test::test_harness::create_component_for_schema_name;

#[test]
async fn list_actions_for_fallout(ctx: &mut DalContext) {
    let component = create_component_for_schema_name(ctx, "fallout", "actions").await;
    let schema_variant = component.schema_variant(ctx).await.unwrap();
    let actions = Action::for_component(ctx, component.id())
        .await
        .expect("unable to list actions for component");
    let action_prototypes = ActionPrototype::for_variant(ctx, schema_variant.id())
        .await
        .unwrap();
    dbg!(&actions, &action_prototypes);

    // TODO(nick): start here and figure out who generates actions.
    assert!(!actions.is_empty());
}
