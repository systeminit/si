// Auto-generated rust code!
// No-Touchy!

use tracing::{debug, info};
use tracing_futures::Instrument as _;
use tracing_opentelemetry::OpenTelemetrySpanExt as _;
use opentelemetry::api::propagation::text_propagator::HttpTextFormat;

#[derive(Debug)]
pub struct Service {
db: si_data::Db,
}

impl Service {
    pub fn new(db: si_data::Db) -> Service {
        Service { db }
    }

    }

#[tonic::async_trait]
impl crate::protobuf::cea_server::Cea for Service {
    
    
    
}

struct TonicMetaWrapper<'a>(&'a mut tonic::metadata::MetadataMap);

impl<'a> opentelemetry::api::propagation::Carrier for TonicMetaWrapper<'a> {
    fn get(&self, key: &'static str) -> Option<&str> {
        let raw_value = self.0.get(key)?;
        match raw_value.to_str() {
            Ok(value) => Some(value),
            Err(_e) => {
                debug!("Cannot extract header for trace parent, not a string");
                None
            }
        }
    }

    fn set(&mut self, key: &'static str, raw_value: String) {
        let value = match tonic::metadata::MetadataValue::from_str(&raw_value) {
            Ok(value) => value,
            Err(_e) => {
                debug!("Cannot insert header for trace parent, not a string");
                debug!("Inserting the empty string");
                tonic::metadata::MetadataValue::from_str("").unwrap()
            }
        };
        self.0.insert(key, value);
    }
}
