use std::sync::Arc;

use si_data::{
    nats,
    pg::{self, InstrumentedClient, InstrumentedTransaction, PgPoolResult},
    NatsClient, NatsTxn, PgError, PgPool, PgTxn,
};
use veritech::EncryptionKey;

use crate::{HistoryActor, Tenancy, Visibility};

/// A context type which contains handles to common core service dependencies.
///
/// These services are typically used by most DAL objects, such as a database connection pool, a
/// function execution client, etc.
#[derive(Clone, Debug)]
pub struct ServicesContext {
    /// A PostgreSQL connection pool.
    pg_pool: PgPool,
    /// A connected NATS client
    nats_conn: NatsClient,
    /// A Veritech client, connected via a NATS connection.
    veritech: veritech::Client,
    /// A key for re-recrypting messages to the function execution system.
    encryption_key: Arc<veritech::EncryptionKey>,
}

impl ServicesContext {
    /// Constructs a new instance of a `ServicesContext`.
    pub fn new(
        pg_pool: PgPool,
        nats_conn: NatsClient,
        veritech: veritech::Client,
        encryption_key: Arc<veritech::EncryptionKey>,
    ) -> Self {
        Self {
            pg_pool,
            nats_conn,
            veritech,
            encryption_key,
        }
    }

    /// Returns a tuple of a new [`DalContextBuilder`] and a PostgreSQL connection.
    ///
    /// The database connection is obtained by requesting one from the inner database connection
    /// pool and is returned apart from the `DalContextBuilder` to ensure that the connection's
    /// ownership is distinct and seperate from the builder.
    pub async fn builder_and_pg_conn(
        &self,
    ) -> PgPoolResult<(DalContextBuilder<'_>, InstrumentedClient)> {
        let pg_conn = self.pg_pool.get().await?;

        Ok((
            DalContextBuilder {
                services_context: self,
            },
            pg_conn,
        ))
    }

    /// Returns a tuple containing a newly created PostgreSQL transaction and a NATS transaction.
    pub async fn transactions<'a>(
        &self,
        pg_conn: &'a mut InstrumentedClient,
    ) -> Result<(pg::PgTxn<'a>, nats::NatsTxn), PgError> {
        let pg_txn = pg_conn.transaction().await?;
        let nats_txn = self.nats_conn.transaction();

        Ok((pg_txn, nats_txn))
    }
}

/// A context type which holds references to underlying services, transactions, and read/write
/// context for DAL objects.
#[derive(Debug)]
pub struct DalContext<'a> {
    /// A reference to a [`ServicesContext`] which has handles to common core services.
    services_context: &'a ServicesContext,
    /// A reference to a PostgreSQL transaction.
    pg_txn: &'a pg::PgTxn<'a>,
    /// A reference to a NATS transaction.
    nats_txn: &'a nats::NatsTxn,
    /// A suitable read tenancy for the consuming DAL objects.
    read_tenancy: Tenancy,
    /// A suitable write tenancy for the consuming DAL objects.
    write_tenancy: Tenancy,
    /// A suitable [`Visibility`] scope for the consuming DAL objects.
    visibility: Visibility,
    /// A suitable [`HistoryActor`] for the consuming DAL objects.
    history_actor: HistoryActor,
}

impl DalContext<'_> {
    /// Takes a reference to a [`ServicesContext`] and returns a builder to construct a
    /// `DalContext`.
    pub fn builder(services_context: &ServicesContext) -> DalContextBuilder<'_> {
        DalContextBuilder { services_context }
    }

    /// Gets a reference to the DAL context's Postgres pool.
    pub fn pg_pool(&self) -> &PgPool {
        &self.services_context.pg_pool
    }

    /// Gets the dal context's pg txn.
    pub fn pg_txn(&self) -> &InstrumentedTransaction {
        self.pg_txn
    }

    /// Gets a reference to the DAL context's NATS connection.
    pub fn nats_conn(&self) -> &NatsClient {
        &self.services_context.nats_conn
    }

    /// Gets the dal context's nats txn.
    pub fn nats_txn(&self) -> &NatsTxn {
        self.nats_txn
    }

    /// Gets a reference to the DAL context's Veritech client.
    pub fn veritech(&self) -> &veritech::Client {
        &self.services_context.veritech
    }

    /// Gets a reference to the DAL context's encryption key.
    pub fn encryption_key(&self) -> &EncryptionKey {
        &self.services_context.encryption_key
    }

    /// Gets a reference to the dal context's read tenancy.
    pub fn read_tenancy(&self) -> &Tenancy {
        &self.read_tenancy
    }

    /// Gets a reference to the dal context's write tenancy.
    pub fn write_tenancy(&self) -> &Tenancy {
        &self.write_tenancy
    }

    /// Gets the dal context's visibility.
    pub fn visibility(&self) -> &Visibility {
        &self.visibility
    }

    /// Gets a reference to the dal context's history actor.
    pub fn history_actor(&self) -> &HistoryActor {
        &self.history_actor
    }
}

/// A context which represents a suitable tenancies, visibilities, etc. for consumption by a set
/// of DAL objects.
pub struct HandlerContext {
    /// A suitable read tenancy for the consuming DAL objects.
    pub read_tenancy: Tenancy,
    /// A suitable write tenancy for the consuming DAL objects.
    pub write_tenancy: Tenancy,
    /// A suitable [`Visibility`] scope for the consuming DAL objects.
    pub visibility: Visibility,
    /// A suitable [`HistoryActor`] for the consuming DAL objects.
    pub history_actor: HistoryActor,
}

/// A builder for a [`DalContext`].
pub struct DalContextBuilder<'a> {
    /// A reference to a [`ServicesContext`] which has handles to common core services.
    services_context: &'a ServicesContext,
}

impl<'a> DalContextBuilder<'a> {
    /// Contructs and returns a new [`DalContext`] using the given transaction references and
    /// [`HandlerContext`].
    pub fn build(
        self,
        handler_context: HandlerContext,
        pg_txn: &'a PgTxn<'a>,
        nats_txn: &'a NatsTxn,
    ) -> DalContext<'a> {
        DalContext {
            services_context: self.services_context,
            pg_txn,
            nats_txn,
            read_tenancy: handler_context.read_tenancy,
            write_tenancy: handler_context.write_tenancy,
            visibility: handler_context.visibility,
            history_actor: handler_context.history_actor,
        }
    }
}
