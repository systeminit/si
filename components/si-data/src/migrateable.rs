use tracing::{self, trace};
use uuid::Uuid;

use crate::storable::Storable;

/// The Migrateable trait is something that can be migrated and managed as a versioned
/// object in the database.
pub trait Migrateable: Storable + std::fmt::Debug {
    //#[tracing::instrument]
    //fn generate_id(&mut self) {
    //    let generated_id = format!("{}:{}", <self as Storable>::type_name(), Uuid::new_v4());
    //    trace!(%generated_id);
    //    self.set_id(format!(
    //        "{}:{}",
    //        <self as Storable>::type_name(),
    //        Uuid::new_v4()
    //    ));
    //}
    fn set_natural_key(&mut self);
    fn natural_key(&self) -> &str;
}

//impl Migrateable {

// fn set_id<T: Into<String>>(&mut self, id: T) {
//     let span = span!(Level::TRACE, "set_id");
//     let _entered_span = span.enter();
//     self.id = id.into();
// }

// #[tracing::instrument]
// fn set_natural_key(&mut self) {
//     self.natural_key = format!("{}/{}", self.integration_service_id, self.name);
// }

// #[tracing::instrument]
// fn natural_key(&self) -> &str {
//     &self.natural_key
// }
//}
