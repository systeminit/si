use dal::node_menu::GenerateMenuItem;
use dal::test::helpers::builtins::{Builtin, BuiltinsHarness};
use dal::{DalContext, DiagramKind};

use crate::dal::test;

#[test]
async fn get_node_menu(ctx: &DalContext<'_, '_>) {
    let mut harness = BuiltinsHarness::new();
    let _docker_image_payload = harness
        .create_component(ctx, "valorant", Builtin::DockerImage)
        .await;

    let gmi = GenerateMenuItem::new(ctx, DiagramKind::Configuration)
        .await
        .expect("cannot get items");
    let items = gmi.raw_items;

    let docker_image_item = items.iter().find(|(path, item)| {
        let mut path = path.clone();
        item.name == "image" && path.pop().expect("path not found") == "docker"
    });
    assert!(
        docker_image_item.is_some(),
        "menu must include the docker image item"
    );
}
