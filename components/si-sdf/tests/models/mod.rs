pub mod entity;
pub mod update_clock;

use crate::filters::change_sets::create_change_set;
use crate::filters::edit_sessions::create_edit_session;
use crate::filters::nodes::create_node;
use crate::{test_cleanup, test_setup, DB};

use si_sdf::models::{list_model, PageToken, Query};

#[tokio::test]
async fn list_model_basic() {
    let test_account = test_setup().await.expect("failed to setup test");
    let change_set_id = create_change_set(&test_account).await;
    let edit_session_id = create_edit_session(&test_account, &change_set_id).await;

    for _ in 0 as u32..18 {
        create_node(&test_account, &change_set_id, &edit_session_id, "service").await;
    }

    let result = list_model(
        &DB,
        None,
        None,
        None,
        None,
        None,
        Some("node".into()),
        Some(test_account.billing_account.id.clone()),
    )
    .await
    .expect("failed to list");

    assert_eq!(result.items.len(), 10, "should have 10 items");
    assert_eq!(result.total_count, 18, "should have 18 items total");
    assert!(result.page_token.is_some(), "has a page token");

    let next_page_token = PageToken::unseal(&result.page_token.unwrap(), &DB.page_secret_key)
        .expect("cannot unseal the page token");

    let next_result = list_model(
        &DB,
        None,
        None,
        None,
        None,
        Some(next_page_token),
        None,
        None,
    )
    .await
    .expect("failed to list next set of results");
    assert_eq!(next_result.items.len(), 8, "should have 8 items");
    assert_eq!(next_result.total_count, 18, "should have 18 items total");
    assert!(
        next_result.page_token.is_none(),
        "does not have a page token"
    );

    test_cleanup(test_account)
        .await
        .expect("failed to finish test");
}
