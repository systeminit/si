use couchbase::subdoc::MutateInSpec;
use couchbase::Cluster;
use futures::executor::block_on;

fn main() {
    env_logger::init();

    let mut cluster = Cluster::connect("couchbase://127.0.0.1", "Administrator", "password")
        .expect("Could not create Cluster reference!");
    let bucket = cluster
        .bucket("travel-sample")
        .expect("Could not open bucket");
    let collection = bucket.default_collection();

    let f = async {
        // Add field to document
        let insert_result = collection
            .mutate_in(
                "airport_1285",
                vec![MutateInSpec::upsert("updated", true).expect("could not encode value")],
                None,
            )
            .await;
        println!("Insert Result: {:?}", insert_result);
    };
    block_on(f);
}
