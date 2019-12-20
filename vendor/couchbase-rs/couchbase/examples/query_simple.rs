use couchbase::{Cluster, CouchbaseError};
use futures::executor::block_on;
use futures::stream::StreamExt;
use serde_json::Value;

fn main() {
    env_logger::init();

    let mut cluster = Cluster::connect("couchbase://127.0.0.1", "Administrator", "password")
        .expect("Could not create cluster reference");
    let _ = cluster.bucket("travel-sample");

    let f = async {
        let mut result = cluster
            .query("select name, type from `travel-sample` limit 5", None)
            .await
            .expect("Could not perform query");

        println!(
            "Rows:\n{:?}",
            result
                .rows_as().expect("Rows already consumed")
                .collect::<Vec<Result<Value, CouchbaseError>>>().await
        );
        println!(
            "Meta:\n{:?}",
            result.meta().await.expect("Could not get query meta")
        );

        cluster.disconnect().expect("Could not shutdown properly");
    };
    block_on(f);
}
