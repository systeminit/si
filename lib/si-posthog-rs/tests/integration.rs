use std::time::Duration;
use tokio::test;

#[test(flavor = "multi_thread", worker_threads = 5)]
async fn full_send_event() {
    let (client, sender) = si_posthog::new()
        .api_endpoint("https://e.systeminit.com")
        .api_key("phc_SoQak5PP054RdTumd69bOz7JhM0ekkxxTXEQsbn3Zg9")
        .build()
        .expect("cannot create posthog client and sender");
    tokio::spawn(sender.run());
    client
        .capture(
            "test-full_send_event_rust",
            "bobo@systeminit.com",
            serde_json::json!({"test": "passes"}),
        )
        .expect("cannot send message");
    tokio::time::sleep(Duration::from_millis(1000)).await;
}
