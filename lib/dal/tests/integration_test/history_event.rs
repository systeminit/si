use dal::{DalContext, HistoryActor, HistoryEvent};
use dal_test::test;

#[test]
async fn new(ctx: &DalContext) {
    let history_event = HistoryEvent::new(
        ctx,
        "change_set.opened",
        "change set created",
        &serde_json::json!({}),
    )
    .await
    .expect("cannot create a new history event");

    assert_eq!(&history_event.actor, &HistoryActor::SystemInit);
    assert_eq!(&history_event.message, "change set created");
    assert_eq!(&history_event.data, &serde_json::json!({}));
    assert_eq!(&history_event.tenancy, ctx.tenancy());
}
