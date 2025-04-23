use dal::DalContext;
use dal_test::{
    expected::{
        ExpectComponent,
        apply_change_set_to_base,
        commit_and_update_snapshot_to_visibility,
        fork_from_head_change_set,
        update_visibility_and_snapshot_to_visibility,
    },
    helpers::connect_components_with_socket_names,
    test,
};
use pretty_assertions_sorted::assert_eq;

#[test]
async fn socket_with_arity_of_one_can_only_have_one_input(ctx: &mut DalContext) {
    let your_beetle = ExpectComponent::create_named(ctx, "private_language", "your_beetle").await;
    let my_beetle = ExpectComponent::create_named(ctx, "private_language", "my_beetle").await;
    let beetle_in_a_box = ExpectComponent::create_named(ctx, "etoiles", "beetle_in_a_box")
        .await
        .component(ctx)
        .await;

    apply_change_set_to_base(ctx).await;

    // connect your_beetle to beetle_in_a_box in one change set
    let cs_1 = fork_from_head_change_set(ctx).await;
    connect_components_with_socket_names(
        ctx,
        your_beetle.id(),
        "private_language",
        beetle_in_a_box.id(),
        "private_language",
    )
    .await
    .expect("connect components");
    commit_and_update_snapshot_to_visibility(ctx).await;

    let connections_on_cs_1 = beetle_in_a_box
        .incoming_connections(ctx)
        .await
        .expect("get incoming connections");
    assert_eq!(1, connections_on_cs_1.len());
    assert_eq!(your_beetle.id(), connections_on_cs_1[0].from_component_id);

    // connect my_beetle to beetle_in_a_box in another change set
    let _cs_2 = fork_from_head_change_set(ctx).await;
    connect_components_with_socket_names(
        ctx,
        my_beetle.id(),
        "private_language",
        beetle_in_a_box.id(),
        "private_language",
    )
    .await
    .expect("connect components");

    apply_change_set_to_base(ctx).await;

    // Confirm that in change set 2, we still have only one connection and it's now from my_beetle, not your_beetle
    let connections_on_head = beetle_in_a_box
        .incoming_connections(ctx)
        .await
        .expect("get incoming connections");

    assert_eq!(1, connections_on_head.len());
    assert_eq!(my_beetle.id(), connections_on_head[0].from_component_id);

    update_visibility_and_snapshot_to_visibility(ctx, cs_1.id).await;

    let connections_on_cs_1_again = beetle_in_a_box
        .incoming_connections(ctx)
        .await
        .expect("get incoming connections");
    assert_eq!(1, connections_on_cs_1_again.len());
    assert_eq!(
        my_beetle.id(),
        connections_on_cs_1_again[0].from_component_id
    );
}
