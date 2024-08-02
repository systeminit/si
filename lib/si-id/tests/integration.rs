use std::env;

use si_id::{run_server, SiIdClient};
use tokio::select;
use tokio_util::{sync::CancellationToken, task::TaskTracker};

#[tokio::test]
async fn get_a_bunch_of_ids() {
    let token = CancellationToken::new();
    let tracker = TaskTracker::new();

    let server_token = token.clone();
    tracker.spawn(async move {
        env::set_var("MACHINE_ID", "1");
        run_server().await.expect("cannot run server");
        select! {
            e = run_server() => {
                panic!("Server errored: {:?}", e);
            },
            () = server_token.cancelled() => {
                return;
            }
        }
    });

    const MAX_ITER: usize = 18_000;
    let mut counter: usize = 0;
    while counter <= MAX_ITER {
        let client = SiIdClient::new("127.0.0.1:7765".parse().expect("not a valid address"))
            .await
            .expect("cannot create client");
        client.get_id().await.expect("cannot get an id");
        counter += 1;
    }
}
