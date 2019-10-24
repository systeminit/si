use couchbase::options::{QueryOptions, ScanConsistency};
use couchbase::{Cluster, CouchbaseError};
use futures::executor::block_on;
use futures::stream::StreamExt;
use serde_json::{json, Value};

use std::collections::HashMap;

fn main() -> Result<(), CouchbaseError> {
    env_logger::init();

    let mut cluster = Cluster::connect("couchbase://127.0.0.1", "Administrator", "password")?;
    let _ = cluster.bucket("travel-sample");

    let f = async {
        let query_options = QueryOptions::new().set_scan_consistency(ScanConsistency::RequestPlus);
        let mut request_plus_result = cluster
            .query(
                "select name, type from `travel-sample` where name = 'Texas Wings'",
                Some(query_options),
            )
            .await.expect("Had some data");

        println!(
            "Rows:\n{:?}",
            request_plus_result
                .rows_as().expect("rows consumed")
                .collect::<Vec<Result<Value, CouchbaseError>>>().await
        );
        cluster.disconnect().expect("Could not shutdown properly");
    };
    block_on(f);
    Ok(())
}

