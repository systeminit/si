use dal::{ComponentType, DalContext};
use dal_test::test;
use dal_test::test_harness::create_component_for_schema_name;

#[test]
async fn set_type(ctx: &mut DalContext) {
    let component = create_component_for_schema_name(ctx, "starfield", "black star").await;

    pretty_assertions_sorted::assert_eq!(
        component.get_type(ctx).await.expect("could not get type"),
        ComponentType::Component
    );

    component
        .set_type(ctx, ComponentType::ConfigurationFrameUp)
        .await
        .expect("could not set type");

    pretty_assertions_sorted::assert_eq!(
        component.get_type(ctx).await.expect("could not get type"),
        ComponentType::ConfigurationFrameUp
    );
}
