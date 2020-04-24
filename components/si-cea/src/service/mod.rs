use crate::agent::client::AgentClient;
use si_data::Db;
use std::convert::TryFrom;

pub mod auth;

pub mod prelude {
    pub use super::authnz;
    pub use crate::{
        CeaError, Component as _, Entity as _, EntityEvent as _, ListRequest as _,
        Service as CeaService, TonicResult,
    };
    pub use std::convert::TryFrom as _;
    pub use tonic::Request as TonicRequest;
    pub use tracing::{debug, debug_span};
    pub use tracing_futures::Instrument as _;
}

#[derive(Debug)]
pub struct Service {
    pub db: Db,
    pub agent: AgentClient,
}

impl Service {
    pub fn new(db: Db, agent: AgentClient) -> Service {
        Service { db, agent }
    }

    pub fn db(&self) -> &Db {
        &self.db
    }

    pub fn agent(&self) -> &AgentClient {
        &self.agent
    }
}

pub async fn authnz<T>(
    db: &Db,
    request: &tonic::Request<T>,
    endpoint: impl AsRef<str>,
) -> crate::CeaResult<crate::Authentication>
where
    T: std::fmt::Debug,
{
    let auth = crate::Authentication::try_from(request)?;
    auth.authorize_on_billing_account(db, endpoint.as_ref())
        .await?;

    Ok(auth)
}
