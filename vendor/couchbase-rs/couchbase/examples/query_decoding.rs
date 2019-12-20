use couchbase::{Cluster, CouchbaseError};
use futures::executor::block_on;
use futures::stream::StreamExt;
use serde_derive::Deserialize;

#[derive(Debug, Deserialize)]
struct Airport {
    airportname: String,
    icao: String,
}

fn main() {
    env_logger::init();

    let mut cluster = Cluster::connect("couchbase://127.0.0.1", "Administrator", "password")
        .expect("Could not create cluster reference");
    let _ = cluster.bucket("travel-sample");

    let f = async {

        let mut result = cluster
            .query(
                "select airportname, icao from `travel-sample` where type = \"airport\" limit 2",
                None,
            )
            .await
            .expect("Could not perform query");

        println!(
            "---> rows {:?}",
            result
                .rows_as().expect("Rows already consumed")
                .collect::<Vec<Result<Airport, CouchbaseError>>>().await
        );
        println!(
            "---> meta {:?}",
            result.meta().await.expect("Could not get query meta")
        );

        cluster.disconnect().expect("Could not shutdown properly");
    };
    block_on(f);
}
