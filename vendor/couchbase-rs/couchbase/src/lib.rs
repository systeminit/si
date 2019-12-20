#![doc(html_root_url = "https://docs.rs/couchbase/1.0.0-alpha.3")]

mod bucket;
mod cluster;
mod collection;
mod instance;
mod util;

pub mod error;
pub mod options;
pub mod result;
pub mod subdoc;

pub use crate::bucket::{Bucket, SharedBucket};
pub use crate::cluster::{Cluster, SharedCluster};
pub use crate::collection::{Collection, SharedCollection};
pub use crate::error::CouchbaseError;
