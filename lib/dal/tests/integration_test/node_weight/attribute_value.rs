use dal::prop::PropPath;
use dal::{AttributeValue, DalContext, Prop};
use dal_test::expected::{
    apply_change_set_to_base, commit_and_update_snapshot_to_visibility, fork_from_head_change_set,
    update_visibility_and_snapshot_to_visibility, ExpectComponent,
};
use dal_test::helpers::{
    connect_components_with_socket_names, create_component_for_default_schema_name,
};
use dal_test::test;

#[test]
async fn change_in_output_component_produces_dvu_root_in_other_change_set(ctx: &mut DalContext) {
    let docker_image = create_component_for_default_schema_name(ctx, "Docker Image", "foobar")
        .await
        .expect("component creation");

    apply_change_set_to_base(ctx).await;

    let cs_with_butane = fork_from_head_change_set(ctx).await;
    let butane = create_component_for_default_schema_name(ctx, "Butane", "butane")
        .await
        .expect("create component");
    connect_components_with_socket_names(
        ctx,
        docker_image.id(),
        "Container Image",
        butane.id(),
        "Container Image",
    )
    .await
    .expect("able to connect");

    commit_and_update_snapshot_to_visibility(ctx).await;
    fork_from_head_change_set(ctx).await;

    let expect_component: ExpectComponent = docker_image.clone().into();

    let prop_id = expect_component
        .prop(ctx, PropPath::new(["root", "domain", "image"]))
        .await
        .prop()
        .id();

    let image_av_id = Prop::attribute_values_for_prop_id(ctx, prop_id)
        .await
        .expect("get attribute values")[0];

    AttributeValue::update(ctx, image_av_id, Some("unpossible".into()))
        .await
        .expect("able to update value");

    apply_change_set_to_base(ctx).await;

    update_visibility_and_snapshot_to_visibility(ctx, cs_with_butane.id).await;

    assert_eq!(
        serde_json::json!("unpossible"),
        AttributeValue::get_by_id_or_error(ctx, image_av_id)
            .await
            .expect("get av")
            .view(ctx)
            .await
            .expect("get view")
            .expect("has view")
    );

    // DVU debouncer does not run in tests so these roots will never get
    // processed unless we explicitly enqueue a dvu. It's enough to see that it
    // made it into the roots
    assert!(ctx
        .workspace_snapshot()
        .expect("get snap")
        .list_dependent_value_value_ids()
        .await
        .expect("able to get dvu values")
        .contains(&image_av_id.into()));
}
