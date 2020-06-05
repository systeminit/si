#![recursion_limit = "256"]

pub mod protobuf {
    include!(concat!(env!("OUT_DIR"), "/si.data.rs"));
}

pub mod db;
pub mod error;
pub mod migrateable;
pub mod mvcc;
pub mod page_token;
pub mod password;
pub mod query;
pub mod storable;

pub use db::{Db, ListResult};
pub use error::{required_field_err, DataError, Result};
pub use migrateable::Migrateable;
pub use protobuf::DataStorableChangeSetEventType;
pub use query::{
    DataQuery, DataQueryBooleanTerm, DataQueryItems, DataQueryItemsExpression,
    DataQueryItemsExpressionComparison, DataQueryItemsExpressionFieldType,
};
pub use storable::{DataStorable, Reference, Storable};

pub fn uuid_string() -> String {
    uuid::Uuid::new_v4().to_string()
}
