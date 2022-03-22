use crate::dal::test;
use crate::test_setup;
use dal::Visibility;

#[test]
async fn head_is_visibile_to_head() {
    test_setup!(
        ctx,
        _secret_key,
        pg,
        conn,
        txn,
        nats_conn,
        _nats,
        _veritech,
        _encr_key
    );

    let visibility = Visibility::new_head(false);
    let check_visibility = visibility;

    let check = visibility
        .is_visible_to(&txn, &check_visibility)
        .await
        .expect("cannot check visibility");
    assert!(check);
}

#[test]
async fn head_is_visible_to_change_set() {
    test_setup!(
        ctx,
        _secret_key,
        pg,
        conn,
        txn,
        nats_conn,
        _nats,
        _veritech,
        _encr_key
    );

    let visibility = Visibility::new_head(false);
    let check_visibility = Visibility::new_change_set(1.into(), false);

    let check = visibility
        .is_visible_to(&txn, &check_visibility)
        .await
        .expect("cannot check visibility");
    assert!(check);
}

#[test]
async fn head_is_visible_to_edit_session() {
    test_setup!(
        ctx,
        _secret_key,
        pg,
        conn,
        txn,
        nats_conn,
        _nats,
        _veritech,
        _encr_key
    );

    let visibility = Visibility::new_head(false);
    let check_visibility = Visibility::new_edit_session(1.into(), 2.into(), false);

    let check = visibility
        .is_visible_to(&txn, &check_visibility)
        .await
        .expect("cannot check visibility");
    assert!(check);
}

#[test]
async fn head_is_invisibile_to_deleted_head() {
    test_setup!(
        ctx,
        _secret_key,
        pg,
        conn,
        txn,
        nats_conn,
        _nats,
        _veritech,
        _encr_key
    );

    let visibility = Visibility::new_head(true);
    let check_visibility = Visibility::new_head(false);

    let check = visibility
        .is_visible_to(&txn, &check_visibility)
        .await
        .expect("cannot check visibility");
    assert!(!check);
}

#[test]
async fn delted_head_is_visibile_to_deleted_head() {
    test_setup!(
        ctx,
        _secret_key,
        pg,
        conn,
        txn,
        nats_conn,
        _nats,
        _veritech,
        _encr_key
    );

    let visibility = Visibility::new_head(true);
    let check_visibility = Visibility::new_head(true);

    let check = visibility
        .is_visible_to(&txn, &check_visibility)
        .await
        .expect("cannot check visibility");
    assert!(check);
}

#[test]
async fn change_set_is_not_visible_to_head() {
    test_setup!(
        ctx,
        _secret_key,
        pg,
        conn,
        txn,
        nats_conn,
        _nats,
        _veritech,
        _encr_key
    );

    let visibility = Visibility::new_change_set(1.into(), false);
    let check_visibility = Visibility::new_head(false);

    let check = visibility
        .is_visible_to(&txn, &check_visibility)
        .await
        .expect("cannot check visibility");
    assert!(!check);
}

#[test]
async fn change_set_is_visible_to_change_set() {
    test_setup!(
        ctx,
        _secret_key,
        pg,
        conn,
        txn,
        nats_conn,
        _nats,
        _veritech,
        _encr_key
    );

    let visibility = Visibility::new_change_set(1.into(), false);
    let check_visibility = Visibility::new_change_set(1.into(), false);

    let check = visibility
        .is_visible_to(&txn, &check_visibility)
        .await
        .expect("cannot check visibility");
    assert!(check);
}

#[test]
async fn change_set_is_invisible_to_different_change_set() {
    test_setup!(
        ctx,
        _secret_key,
        pg,
        conn,
        txn,
        nats_conn,
        _nats,
        _veritech,
        _encr_key
    );

    let visibility = Visibility::new_change_set(1.into(), false);
    let check_visibility = Visibility::new_change_set(2.into(), false);

    let check = visibility
        .is_visible_to(&txn, &check_visibility)
        .await
        .expect("cannot check visibility");
    assert!(!check);
}

#[test]
async fn change_set_is_visible_to_edit_session() {
    test_setup!(
        ctx,
        _secret_key,
        pg,
        conn,
        txn,
        nats_conn,
        _nats,
        _veritech,
        _encr_key
    );

    let visibility = Visibility::new_change_set(1.into(), false);
    let check_visibility = Visibility::new_edit_session(1.into(), 1.into(), false);

    let check = visibility
        .is_visible_to(&txn, &check_visibility)
        .await
        .expect("cannot check visibility");
    assert!(check);
}

#[test]
async fn edit_session_is_visible_to_edit_session() {
    test_setup!(
        ctx,
        _secret_key,
        pg,
        conn,
        txn,
        nats_conn,
        _nats,
        _veritech,
        _encr_key
    );

    let visibility = Visibility::new_edit_session(1.into(), 1.into(), false);
    let check_visibility = Visibility::new_edit_session(1.into(), 1.into(), false);

    let check = visibility
        .is_visible_to(&txn, &check_visibility)
        .await
        .expect("cannot check visibility");
    assert!(check);
}

#[test]
async fn edit_session_is_invisible_change_set() {
    test_setup!(
        ctx,
        _secret_key,
        pg,
        conn,
        txn,
        nats_conn,
        _nats,
        _veritech,
        _encr_key
    );

    let visibility = Visibility::new_edit_session(1.into(), 1.into(), false);
    let check_visibility = Visibility::new_change_set(1.into(), false);

    let check = visibility
        .is_visible_to(&txn, &check_visibility)
        .await
        .expect("cannot check visibility");
    assert!(!check);
}

#[test]
async fn edit_session_is_invisible_to_different_edit_session() {
    test_setup!(
        ctx,
        _secret_key,
        pg,
        conn,
        txn,
        nats_conn,
        _nats,
        _veritech,
        _encr_key
    );

    let visibility = Visibility::new_edit_session(1.into(), 1.into(), false);
    let check_visibility = Visibility::new_edit_session(1.into(), 2.into(), false);

    let check = visibility
        .is_visible_to(&txn, &check_visibility)
        .await
        .expect("cannot check visibility");
    assert!(!check);
}
