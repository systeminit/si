use si_model::{Capability, Group};
use si_model_test::{
    create_group_with_users, create_user, one_time_setup, signup_new_billing_account, TestContext,
};

#[tokio::test]
async fn new() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, veritech, _event_log_fs, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");

    let nba = signup_new_billing_account(&pg, &txn, &nats, &nats_conn, &veritech).await;

    let user = create_user(
        &txn,
        &nats,
        "adam jacob",
        "adam@systeminit.com",
        &nba.billing_account.id,
    )
    .await;
    let user_ids = vec![user.id.clone()];
    let capabilities = vec![Capability {
        subject: String::from("any"),
        action: String::from("any"),
    }];

    let group = Group::new(
        &txn,
        &nats,
        "coolcats",
        user_ids.clone(),
        vec![],
        capabilities.clone(),
        &nba.billing_account.id,
    )
    .await
    .expect("cannot create group");
    assert_eq!(group.name, "coolcats");
    assert_eq!(group.user_ids, user_ids);
    assert_eq!(group.capabilities, capabilities);
}

#[tokio::test]
async fn get() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, veritech, _event_log_fs, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");

    let nba = signup_new_billing_account(&pg, &txn, &nats, &nats_conn, &veritech).await;

    let group = create_group_with_users(
        &txn,
        &nats,
        "funky",
        vec![String::from("adam"), String::from("fletcher")],
        vec![Capability::new("any", "any")],
        &nba.billing_account.id,
    )
    .await;

    let obj = Group::get(&txn, &group.id).await.expect("cannot get group");
    assert_eq!(obj.name, group.name);
    assert_eq!(obj.user_ids, group.user_ids);
    assert_eq!(obj.capabilities, group.capabilities);
}

#[tokio::test]
async fn get_administrators_group() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, veritech, _event_log_fs, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");
    let nba = signup_new_billing_account(&pg, &txn, &nats, &nats_conn, &veritech).await;

    let admin = Group::get_administrators_group(&txn, &nba.billing_account.id)
        .await
        .expect("cannot get administrators group");
    assert_eq!(admin.name, "administrators");
}

#[tokio::test]
async fn save() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, veritech, _event_log_fs, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");
    let nba = signup_new_billing_account(&pg, &txn, &nats, &nats_conn, &veritech).await;

    let mut group = create_group_with_users(
        &txn,
        &nats,
        "funky",
        vec![String::from("adam"), String::from("fletcher")],
        vec![Capability::new("any", "any")],
        &nba.billing_account.id,
    )
    .await;

    group.name = String::from("chastisement");
    let cg = group
        .save(&txn, &nats)
        .await
        .expect("cannot save group with changed name");
    assert_eq!(group.name, cg.name, "change name");

    group
        .capabilities
        .push(Capability::new("justify", "whatevs"));
    let cg = group
        .save(&txn, &nats)
        .await
        .expect("cannot save group with changed capabilities");
    assert_eq!(group.capabilities, cg.capabilities, "add capabilities");

    let rows = txn
        .query(
            "SELECT subject, action FROM group_capabilities WHERE group_id = si_id_to_primary_key_v1($1) ORDER BY subject",
            &[&group.id],
        )
        .await
        .expect("cannot select capabilities for group");
    assert_eq!(rows.len(), 2, "we have the right number of capabilities");
    let mut rows_iter = rows.iter();
    let row = rows_iter.next().expect("have a row");
    let subject: String = row.get("subject");
    let action: String = row.get("action");
    assert_eq!(String::from("any"), subject);
    assert_eq!(String::from("any"), action);
    let row = rows_iter.next().expect("have a second row");
    let subject: String = row.get("subject");
    let action: String = row.get("action");
    assert_eq!(String::from("justify"), subject);
    assert_eq!(String::from("whatevs"), action);

    group.capabilities = vec![
        Capability::new("justify", "whatevs"),
        Capability::new("poop", "canoe"),
    ];
    let cg = group
        .save(&txn, &nats)
        .await
        .expect("cannot save group with changed capabilities");
    assert_eq!(group.capabilities, cg.capabilities, "add capabilities");
    let rows = txn
        .query(
            "SELECT subject, action FROM group_capabilities WHERE group_id = si_id_to_primary_key_v1($1) ORDER BY subject",
            &[&group.id],
        )
        .await
        .expect("cannot select capabilities for group");
    assert_eq!(rows.len(), 2, "we have the right number of capabilities");
    let mut rows_iter = rows.iter();
    let row = rows_iter.next().expect("have a row");
    let subject: String = row.get("subject");
    let action: String = row.get("action");
    assert_eq!(String::from("justify"), subject);
    assert_eq!(String::from("whatevs"), action);
    let row = rows_iter.next().expect("have a second row");
    let subject: String = row.get("subject");
    let action: String = row.get("action");
    assert_eq!(String::from("poop"), subject);
    assert_eq!(String::from("canoe"), action);

    let new_user = create_user(
        &txn,
        &nats,
        "athena",
        "athena@ancient.localdomain",
        &nba.billing_account.id,
    )
    .await;
    let second_user_id = group.user_ids[0].clone();
    group.user_ids = vec![new_user.id.clone(), second_user_id.clone()];
    let cg = group
        .save(&txn, &nats)
        .await
        .expect("cannot save group with changed users");
    assert_eq!(group.user_ids, cg.user_ids, "add users");
    let rows = txn
        .query(
            "SELECT user_id FROM group_user_members WHERE group_id = si_id_to_primary_key_v1($1) ORDER BY user_id",
            &[&group.id],
        )
        .await
        .expect("cannot select users for group");
    assert_eq!(rows.len(), 2, "we have the right number of users");
    let mut rows_iter = rows.iter();
    let row = rows_iter.next().expect("have a row");
    let raw_user_id: i64 = row.get("user_id");
    let user_id = format!("user:{}", raw_user_id);
    assert_eq!(&user_id, &second_user_id);
    let row = rows_iter.next().expect("have a second row");
    let raw_user_id: i64 = row.get("user_id");
    let user_id = format!("user:{}", raw_user_id);
    assert_eq!(&user_id, &new_user.id);
}
