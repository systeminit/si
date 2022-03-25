use std::sync::Arc;

use si_data::{
    nats,
    pg::{self, InstrumentedClient, InstrumentedTransaction, PgPoolResult},
    NatsClient, NatsTxn, PgPool, PgTxn,
};
use thiserror::Error;
use veritech::EncryptionKey;

use crate::{HistoryActor, ReadTenancy, Visibility, WriteTenancy};

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

    /// Consumes and returns a tuple of a new [`DalContextBuilder`] and a PostgreSQL connection.
    ///
    /// The database connection is obtained by requesting one from the inner database connection
    /// pool and is returned apart from the `DalContextBuilder` to ensure that the connection's
    /// ownership is distinct and seperate from the builder.
    pub async fn into_builder_and_pg_conn(
        self,
    ) -> PgPoolResult<(DalContextBuilder, InstrumentedClient)> {
        let pg_conn = self.pg_pool.get().await?;

        Ok((
            DalContextBuilder {
                services_context: self,
            },
            pg_conn,
        ))
    }
}

/// A context type which holds references to underlying services, transactions, and read/write
/// context for DAL objects.
#[derive(Debug)]
pub struct DalContext<'s, 't> {
    /// A reference to a [`ServicesContext`] which has handles to common core services.
    services_context: &'s ServicesContext,
    /// A reference to a set of atomically related transactions.
    txns: &'t Transactions<'t>,
    /// A suitable read tenancy for the consuming DAL objects.
    read_tenancy: ReadTenancy,
    /// A suitable write tenancy for the consuming DAL objects.
    write_tenancy: WriteTenancy,
    /// A suitable [`Visibility`] scope for the consuming DAL objects.
    visibility: Visibility,
    /// A suitable [`HistoryActor`] for the consuming DAL objects.
    history_actor: HistoryActor,
}

impl DalContext<'_, '_> {
    /// Takes a reference to a [`ServicesContext`] and returns a builder to construct a
    /// `DalContext`.
    pub fn builder(services_context: ServicesContext) -> DalContextBuilder {
        DalContextBuilder { services_context }
    }

    /// Gets a reference to the DAL context's Postgres pool.
    pub fn pg_pool(&self) -> &PgPool {
        &self.services_context.pg_pool
    }

    /// Gets the dal context's pg txn.
    pub fn pg_txn(&self) -> &InstrumentedTransaction {
        &self.txns.pg_txn
    }

    /// Gets a reference to the DAL context's NATS connection.
    pub fn nats_conn(&self) -> &NatsClient {
        &self.services_context.nats_conn
    }

    /// Gets the dal context's nats txn.
    pub fn nats_txn(&self) -> &NatsTxn {
        &self.txns.nats_txn
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
    pub fn read_tenancy(&self) -> &ReadTenancy {
        &self.read_tenancy
    }

    /// Gets a reference to the dal context's write tenancy.
    pub fn write_tenancy(&self) -> &WriteTenancy {
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
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RequestContext {
    /// A suitable read tenancy for the consuming DAL objects.
    pub read_tenancy: ReadTenancy,
    /// A suitable write tenancy for the consuming DAL objects.
    pub write_tenancy: WriteTenancy,
    /// A suitable [`Visibility`] scope for the consuming DAL objects.
    pub visibility: Visibility,
    /// A suitable [`HistoryActor`] for the consuming DAL objects.
    pub history_actor: HistoryActor,
}

/// A request context builder which requires a [`Visibility`] to be completed.
#[derive(Debug)]
pub struct AccessBuilder {
    /// A suitable read tenancy for the consuming DAL objects.
    read_tenancy: ReadTenancy,
    /// A suitable write tenancy for the consuming DAL objects.
    write_tenancy: WriteTenancy,
    /// A suitable [`HistoryActor`] for the consuming DAL objects.
    history_actor: HistoryActor,
}

impl AccessBuilder {
    /// Constructs a new instance given a set of tenancies and a [`HistoryActor`].
    pub fn new(
        read_tenancy: ReadTenancy,
        write_tenancy: WriteTenancy,
        history_actor: HistoryActor,
    ) -> Self {
        Self {
            read_tenancy,
            write_tenancy,
            history_actor,
        }
    }

    /// Builds and returns a new [`RequestContext`] using the default [`Visibillity`].
    pub fn build_head(self) -> RequestContext {
        RequestContext {
            read_tenancy: self.read_tenancy,
            write_tenancy: self.write_tenancy,
            visibility: Visibility::new_head(false),
            history_actor: self.history_actor,
        }
    }

    /// Builds and returns a new [`RequestContext`] using the given [`Visibillity`].
    pub fn build(self, visibility: Visibility) -> RequestContext {
        RequestContext {
            read_tenancy: self.read_tenancy,
            write_tenancy: self.write_tenancy,
            visibility,
            history_actor: self.history_actor,
        }
    }
}

/// A builder for a [`DalContext`].
pub struct DalContextBuilder {
    /// A [`ServicesContext`] which has handles to common core services.
    services_context: ServicesContext,
}

impl DalContextBuilder {
    /// Contructs and returns a new [`DalContext`] using the given transaction references and
    /// [`RequestContext`].
    pub fn build<'t>(
        &self,
        handler_context: RequestContext,
        txns: &'t Transactions,
    ) -> DalContext<'_, 't> {
        DalContext {
            services_context: &self.services_context,
            txns,
            read_tenancy: handler_context.read_tenancy,
            write_tenancy: handler_context.write_tenancy,
            visibility: handler_context.visibility,
            history_actor: handler_context.history_actor,
        }
    }

    /// Gets a reference to the DAL context's NATS connection.
    pub fn nats_conn(&self) -> &NatsClient {
        &self.services_context.nats_conn
    }
}

#[derive(Debug, Error)]
pub enum TransactionsError {
    #[error(transparent)]
    Pg(#[from] pg::Error),
    #[error(transparent)]
    Nats(#[from] nats::Error),
}

// A set of atomically-related transactions.
//
// Ideally, all of these inner transactions would be committed or rolled back together, hence the
// API methods.
#[derive(Debug)]
pub struct Transactions<'a> {
    /// A PostgreSQL transaction.
    pg_txn: PgTxn<'a>,
    /// A NATS transaction.
    nats_txn: NatsTxn,
}

impl<'a> Transactions<'a> {
    /// Creates and returns a new `Transactions` instance.
    pub fn new(pg_txn: PgTxn<'a>, nats_txn: NatsTxn) -> Self {
        Self { pg_txn, nats_txn }
    }

    /// Gets a reference to the PostgreSQL transaction.
    pub fn pg(&self) -> &PgTxn<'a> {
        &self.pg_txn
    }

    /// Gets a reference to the NATS transaction.
    pub fn nats(&self) -> &NatsTxn {
        &self.nats_txn
    }

    /// Consumes all inner transactions and committing all changes made within them.
    pub async fn commit(self) -> Result<(), TransactionsError> {
        self.pg_txn.commit().await?;
        self.nats_txn.commit().await?;
        Ok(())
    }

    /// Rolls all inner transactions back, discarding all changes made within them.
    ///
    /// This is equivalent to the transaction's `Drop` implementations, but provides any error
    /// encountered to the caller.
    pub async fn rollback(self) -> Result<(), TransactionsError> {
        self.pg_txn.rollback().await?;
        self.nats_txn.rollback().await?;
        Ok(())
    }
}
