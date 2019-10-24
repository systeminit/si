use couchbase::Cluster;
use futures::executor::block_on;
use serde_derive::Deserialize;

#[derive(Debug, Deserialize)]
struct Airport {
    airportname: String,
    icao: String,
}

fn main() {
    env_logger::init();

    let mut cluster = Cluster::connect("couchbase://127.0.0.1", "Administrator", "password")
        .expect("Could not create Cluster reference!");
    let bucket = cluster
        .bucket("travel-sample")
        .expect("Could not open bucket");
    let collection = bucket.default_collection();

    let f = async {
        let found_doc = collection
            .get("airport_1297", None)
            .await
            .expect("Error while loading doc");
        println!("Airline Document: {:?}", found_doc);
        println!(
            "Content Decoded {:?}",
            found_doc.content_as::<Airport>()
        );

        println!(
            "Document does exist?: {:?}",
            collection.exists("airport_1297", None).await
        );

        println!(
            "Airline Document: {:?}",
            collection.get("enoent", None).await
        );

        println!("Upsert: {:?}", collection.upsert("foo", "bar", None).await);
        println!("Get: {:?}", collection.get("foo", None).await);

        println!("Remove: {:?}", collection.remove("foo", None).await);
        println!("Get: {:?}", collection.get("foo", None).await);

        println!(
            "First Insert: {:?}",
            collection.insert("bla", "bla", None).await
        );
        println!(
            "Second Insert: {:?}",
            collection.insert("bla", "bla", None).await
        );
        cluster.disconnect().expect("Could not shutdown properly");
    };
    block_on(f);
}
