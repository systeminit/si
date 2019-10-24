use couchbase::options::QueryOptions;
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
        let positional_options =
            QueryOptions::new().set_positional_parameters(vec![json!("Texas Wings")]);
        let mut positional_result = cluster
            .query(
                "select name, type from `travel-sample` where name = ?",
                Some(positional_options),
            )
            .await.expect("Had some data");

        println!(
            "Rows:\n{:?}",
            positional_result
                .rows_as().expect("rows consumed")
                .collect::<Vec<Result<Value, CouchbaseError>>>().await
        );

        let mut named_params = HashMap::new();
        named_params.insert("name".into(), json!("Texas Wings"));
        let named_options = QueryOptions::new().set_named_parameters(named_params);
        let mut named_result = cluster
            .query(
                "select name, type from `travel-sample` where name = $name",
                Some(named_options),
            )
            .await
            .expect("Could not perform query");

        println!(
            "Rows:\n{:?}",
            named_result
                .rows_as().expect("Failed because rows were consumed")
                .collect::<Vec<Result<Value, CouchbaseError>>>().await
        );

        cluster.disconnect().expect("Could not shutdown properly");
    };
    block_on(f);
    Ok(())
}
