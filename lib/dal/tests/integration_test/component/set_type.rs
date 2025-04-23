use dal::{
    Component,
    ComponentType,
    DalContext,
};
use dal_test::{
    helpers::create_component_for_default_schema_name_in_default_view,
    test,
};

#[test]
async fn set_type(ctx: &mut DalContext) {
    let component =
        create_component_for_default_schema_name_in_default_view(ctx, "starfield", "black star")
            .await
            .expect("could not create component");

    pretty_assertions_sorted::assert_eq!(
        component.get_type(ctx).await.expect("could not get type"),
        ComponentType::Component
    );

    Component::set_type_by_id(ctx, component.id(), ComponentType::ConfigurationFrameUp)
        .await
        .expect("could not update type");

    pretty_assertions_sorted::assert_eq!(
        component.get_type(ctx).await.expect("could not get type"),
        ComponentType::ConfigurationFrameUp
    );
}
