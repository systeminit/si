use dal::{node_menu::GenerateMenuItem, DalContext};
use dal_test::test;

/// Recommended to run with the following environment variable:
/// ```shell
/// SI_TEST_BUILTIN_SCHEMAS=test
/// ```
#[test]
async fn get_node_menu(ctx: &DalContext) {
    let gmi = GenerateMenuItem::new(ctx, true)
        .await
        .expect("cannot get items");
    let raw_items = gmi.raw_items;

    let item = raw_items.iter().find(|(path, item)| {
        let mut path = path.clone();
        item.name == "starfield" && path.pop().expect("path not found") == "test exclusive"
    });
    assert!(item.is_some());
}
