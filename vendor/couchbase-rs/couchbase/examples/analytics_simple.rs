use couchbase::{Cluster, CouchbaseError};
use futures::stream::StreamExt;
use futures::executor::block_on;
use serde_json::Value;

fn main() {
    env_logger::init();

    let mut cluster = Cluster::connect("couchbase://127.0.0.1", "Administrator", "password")
        .expect("Could not create cluster reference!");
    let _ = cluster.bucket("travel-sample");

    let f = async {
        let mut result = cluster
            .analytics_query("SELECT DataverseName FROM Metadata.`Dataverse`", None)
            .await
            .expect("Could not perform analytics query");

        println!(
            "---> rows {:?}",
            result
                .rows_as().expect("Rows already consumed")
                .collect::<Vec<Result<Value, CouchbaseError>>>().await
        );
        println!(
            "---> meta {:?}",
            result.meta().await.expect("Could not get analytics meta")
        );

        cluster.disconnect().expect("Could not shutdown properly");
    };
    block_on(f);
}
