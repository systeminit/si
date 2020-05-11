use crate::protobuf::IntegrationService;
use si_data::Db;

impl IntegrationService {
    pub async fn migrate(_db: &Db) -> si_data::Result<()> {
        Ok(())
    }
}
