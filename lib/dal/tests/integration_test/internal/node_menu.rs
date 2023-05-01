use dal::{node_menu::GenerateMenuItem, DalContext};
use dal_test::helpers::component_bag::ComponentBagger;
use dal_test::test;

#[test]
async fn get_node_menu(ctx: &DalContext) {
    let mut bagger = ComponentBagger::new();
    let _docker_image_bag = bagger.create_component(ctx, "valorant", "Docker Image");

    let gmi = GenerateMenuItem::new(ctx).await.expect("cannot get items");
    let items = gmi.raw_items;

    let docker_image_item = items.iter().find(|(path, item)| {
        let mut path = path.clone();
        item.name == "Image" && path.pop().expect("path not found") == "Docker"
    });
    assert!(
        docker_image_item.is_some(),
        "menu must include the docker image item"
    );
}
