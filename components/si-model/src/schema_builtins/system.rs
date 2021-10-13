use crate::{Schema, SchemaResult};
use si_data::{NatsTxn, PgTxn};

pub async fn migrate(txn: &PgTxn<'_>, nats: &NatsTxn) -> SchemaResult<()> {
    let _schema =
        Schema::find_or_create_global(&txn, &nats, "si", "system", "system", "systems").await?;
    Ok(())
}
