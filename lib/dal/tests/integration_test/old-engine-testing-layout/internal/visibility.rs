use dal::{ChangeSetPk, DalContext, Visibility};
use dal_test::test;

#[test]
async fn head_is_visibile_to_head(ctx: &DalContext) {
    let visibility = Visibility::new_head(false);
    let check_visibility = visibility;

    let check = visibility
        .is_visible_to(ctx, &check_visibility)
        .await
        .expect("cannot check visibility");
    assert!(check);
}

#[test]
async fn head_is_visible_to_change_set(ctx: &DalContext) {
    let visibility = Visibility::new_head(false);
    let check_visibility = Visibility::new_change_set(ChangeSetPk::generate(), false);

    let check = visibility
        .is_visible_to(ctx, &check_visibility)
        .await
        .expect("cannot check visibility");
    assert!(check);
}

#[test]
async fn head_is_invisibile_to_deleted_head(ctx: &DalContext) {
    let visibility = Visibility::new_head(true);
    let check_visibility = Visibility::new_head(false);

    let check = visibility
        .is_visible_to(ctx, &check_visibility)
        .await
        .expect("cannot check visibility");
    assert!(!check);
}

#[test]
async fn delted_head_is_visibile_to_deleted_head(ctx: &DalContext) {
    let visibility = Visibility::new_head(true);
    let check_visibility = Visibility::new_head(true);

    let check = visibility
        .is_visible_to(ctx, &check_visibility)
        .await
        .expect("cannot check visibility");
    assert!(check);
}

#[test]
async fn change_set_is_not_visible_to_head(ctx: &DalContext) {
    let visibility = Visibility::new_change_set(ChangeSetPk::generate(), false);
    let check_visibility = Visibility::new_head(false);

    let check = visibility
        .is_visible_to(ctx, &check_visibility)
        .await
        .expect("cannot check visibility");
    assert!(!check);
}

#[test]
async fn change_set_is_visible_to_change_set(ctx: &DalContext) {
    let pk = ChangeSetPk::generate();
    let visibility = Visibility::new_change_set(pk, false);
    let check_visibility = Visibility::new_change_set(pk, false);

    let check = visibility
        .is_visible_to(ctx, &check_visibility)
        .await
        .expect("cannot check visibility");
    assert!(check);
}

#[test]
async fn change_set_is_invisible_to_different_change_set(ctx: &DalContext) {
    let visibility = Visibility::new_change_set(ChangeSetPk::generate(), false);
    let check_visibility = Visibility::new_change_set(ChangeSetPk::generate(), false);

    let check = visibility
        .is_visible_to(ctx, &check_visibility)
        .await
        .expect("cannot check visibility");
    assert!(!check);
}
