use crate::{test_cleanup, test_setup};
use crate::{DB, NATS, SETTINGS};
use si_sdf::filters::api;
use si_sdf::models::{Comparison, ListReply, Query};

#[tokio::test]
async fn list() {
    let test_account = test_setup().await.expect("failed to setup test");

    let filter = api(&DB, &NATS, &SETTINGS.jwt_encrypt.key);

    let res = warp::test::request()
        .method("GET")
        .header("authorization", &test_account.authorization)
        .path("/workspaces")
        .reply(&filter)
        .await;
    assert_eq!(res.status(), 200, "workspace list should succeed");
    let list_reply: ListReply =
        serde_json::from_slice(res.body()).expect("can generate a reply from the body");
    assert_eq!(1, list_reply.items.len());

    // With a query for a named workspace
    let query = Query::generate_for_string("name", Comparison::Equals, "bongos");
    let query_string = query
        .to_url_string()
        .expect("cannot create url safe string for query");

    let res = warp::test::request()
        .method("GET")
        .header("authorization", &test_account.authorization)
        .path(&format!("/workspaces?query={}", query_string))
        .reply(&filter)
        .await;
    assert_eq!(res.status(), 200, "workspace list should succeed");
    let list_reply: ListReply =
        serde_json::from_slice(res.body()).expect("can generate a reply from the body");
    assert_eq!(0, list_reply.items.len());

    test_cleanup(test_account)
        .await
        .expect("failed to finish test");
}
