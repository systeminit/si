extern crate couchbase;

mod mock;

#[test]
fn test_kv_ops() {
    let mut mock = mock::MockServer::start();

    std::thread::sleep(std::time::Duration::from_secs(5));

    mock.stop();
    /*let mut cluster = Cluster::connect("couchbase://127.0.0.1", "Administrator", "password");
    let bucket = cluster.bucket("travel-sample");
    let collection = bucket.default_collection();


    println!("{:?}", collection.get("foo", None));*/

    // cluster.disconnect();

    // std::thread::sleep(std::time::Duration::from_secs(100));
}
