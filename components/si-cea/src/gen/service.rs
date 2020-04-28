// Auto-generated rust code!
// No-Touchy!

use tonic;
use tracing::{self, info, info_span};
use tracing_futures::Instrument as _;

use si_data;

#[derive(Debug)]
pub struct Service {
    pub db: si_data::Db,
}

impl Service {
    pub fn new(db: si_data::Db) -> Service {
        Service { db }
    }

    pub fn db(&self) -> &si_data::Db {
        &self.db
    }
}

#[tonic::async_trait]
impl crate::protobuf::cea_server::Cea for Service {}
