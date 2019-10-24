# Couchbase Rust SDK

[![LICENSE](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Crates.io Version](https://img.shields.io/crates/v/couchbase.svg)](https://crates.io/crates/couchbase)

This is the repository for the official, community supported Couchbase Rust SDK. It is currently a work in progress and built on top of [libcouchbase](https://github.com/couchbase/libcouchbase/).

## Requirements

Make sure to have all [libcouchbase](https://docs.couchbase.com/c-sdk/current/start-using-sdk.html) requirements satisfied to build it properly. Also [bindgen](https://rust-lang.github.io/rust-bindgen/requirements.html) requirements need to be in place. Other than that, you should be good to go out of the box if you use a recent rust version. We recommend the 2018 edition just because.

## Installation

```toml
[dependencies]
couchbase = "1.0.0-alpha.2"
```

## Usage

The `examples` folder has a bunch more, but here is a basic getting started doing a kv op:

```rust
use couchbase::Cluster;
use futures::Future;
use serde_derive::Deserialize;

#[derive(Debug, Deserialize)]
struct Airport {
    airportname: String,
    icao: String,
}

fn main() {
    let mut cluster = Cluster::connect("couchbase://127.0.0.1", "Administrator", "password")
        .expect("Could not create Cluster reference!");

    let bucket = cluster
        .bucket("travel-sample")
        .expect("Could not open bucket");
    let collection = bucket.default_collection();

    let found_doc = collection
        .get("airport_1297", None)
        .wait()
        .expect("Error while loading doc");
    println!("Airline Document: {:?}", found_doc);

    if found_doc.is_some() {
        println!(
            "Content Decoded {:?}",
            found_doc.unwrap().content_as::<Airport>()
        );
    }

    cluster.disconnect().expect("Could not shutdown properly");
}
```

## Examples
More examples can be found in the `examples` folder. Please open a ticket if something is not present or does not showcase what you need.