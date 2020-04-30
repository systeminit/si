use crate::CeaResult;
use si_data::Db;
use std::convert::TryFrom;

pub mod auth;

pub mod prelude {
    pub use super::authnz;
    pub use crate::agent::client::AgentClient;
    pub use crate::component::Component as _;
    pub use crate::entity::Entity as _;
    pub use crate::entity_event::EntityEvent as _;
    pub use crate::error::{CeaError, CeaResult, TonicResult};
    pub use crate::list::ListRequest as _;
    pub use si_data::Db;
    pub use tonic::Request as TonicRequest;
    pub use tracing::{debug, debug_span};
    pub use tracing_futures::Instrument as _;
}

pub async fn authnz<T>(
    db: &Db,
    request: &tonic::Request<T>,
    endpoint: impl AsRef<str>,
) -> CeaResult<auth::Authentication>
where
    T: std::fmt::Debug,
{
    let auth = auth::Authentication::try_from(request)?;
    auth.authorize_on_billing_account(db, endpoint.as_ref())
        .await?;

    Ok(auth)
}
