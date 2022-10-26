use axum::Json;
use serde::Serialize;

use super::{
    extract::{Nats, NatsTxn, PgPool, PgRoTxn, PgRwTxn},
    AppResult,
};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Demo {
    is_cool: bool,
}

// NOTE in practice we wouldn't have asked for all these parameters in the same function, although
// there's no rule against it either.
pub async fn demo(
    // We have some custom extractors, including this one which gets us a read/write transaction.
    // Due to ownership and borrowing (and because we don't yet have GAT in Rust--trust me, I
    // discovered my own need for this to try and enable the experience I was after), we need to
    // call `.start().await?` on this type to start the transaction.
    mut txn: PgRwTxn,
    // Okay, what if we aren't mutating any state and this is purely a query/read-only situation?
    // Well, we have a custom extractor that starts a read-only transaction!
    mut ro_txn: PgRoTxn,
    // Maybe we need a low level instance of our PG pool? We have an extractor for this as well! In
    // this case we're using destructuring to get at the `si_data_pg::pg::PgPool` directly.
    PgPool(_pg): PgPool,
    // Just like with pg, we have our NATS txn which we can get, ready to start like so:
    mut nats_txn: NatsTxn,
    // And if a low level NATS client is required, then use the extractor like with `PgPool`
    Nats(_nats): Nats,
) -> AppResult<Json<Demo>> {
    // Start our RW txn
    let _txn = txn.start().await?;
    // Start our seperate RO txn
    let _ro_txn = ro_txn.start().await?;
    // Start our NATS txn
    let _nats_txn = nats_txn.start().await?;

    // Do some work...
    // We can use the `?` operator because this is nothing more than an async fn returning a
    // `Result`

    // Let's make our reply
    let reply = Demo { is_cool: true };

    // Use the JSON response wrapper which serializes into JSON and ultimately matches our return
    // type, nice
    Ok(Json(reply))
}
