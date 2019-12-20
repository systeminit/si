use couchbase::subdoc::LookupInSpec;
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
        // Fetch only a partial list of fields
        let partial_result = collection
            .lookup_in("airport_1285", vec![LookupInSpec::get("geo")], None)
            .await;
        println!("Partial Result: {:?}", partial_result);

        // Fetch the full document (might be needed in combination with xattrs or macros)
        let full_result = collection
            .lookup_in(
                "airline_10123",
                vec![LookupInSpec::get_full_document()],
                None,
            )
            .await;
        println!("Full Result: {:?}", full_result);
    };
    block_on(f);
}
