use crate::schema::{SchemaResult, SchemaVariant};
use crate::{HistoryActor, Schema, SchemaKind, StandardModel, Tenancy, Visibility};
use si_data::{NatsTxn, PgTxn};

// TODO(nick): this function should return an "AlreadyExists" error instead of None wrapped by Ok,
// but since the client will have to deal with same scenario either way, it makes little difference
// for now.
pub async fn create_schema(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    tenancy: &Tenancy,
    visibility: &Visibility,
    history_actor: &HistoryActor,
    schema_name: &str,
    schema_kind: &SchemaKind,
) -> SchemaResult<Option<Schema>> {
    match default_schema_exists(txn, tenancy, visibility, schema_name).await? {
        true => Ok(None),
        false => {
            let schema = Schema::new(
                txn,
                nats,
                tenancy,
                visibility,
                history_actor,
                schema_name,
                schema_kind,
            )
            .await?;
            Ok(Some(schema))
        }
    }
}

// TODO(nick): there's one issue here. If the schema kind has changed, then this check will be
// inaccurate. As a result, we will be unable to re-create the schema without manual intervention.
// This should be fine since this code should likely only last as long as default schemas need to
// be created... which is hopefully not long.... hopefully...
async fn default_schema_exists(
    txn: &PgTxn<'_>,
    tenancy: &Tenancy,
    visibility: &Visibility,
    schema_name: &str,
) -> SchemaResult<bool> {
    Ok(
        !Schema::find_by_attr(txn, tenancy, visibility, "name", &schema_name.to_string())
            .await?
            .is_empty(),
    )
}

pub async fn create_and_set_default_schema_variant(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    tenancy: &Tenancy,
    visibility: &Visibility,
    history_actor: &HistoryActor,
    schema: &mut Schema,
) -> SchemaResult<()> {
    let variant = SchemaVariant::new(txn, nats, tenancy, visibility, history_actor, "v0").await?;
    variant
        .set_schema(txn, nats, visibility, history_actor, schema.id())
        .await?;
    schema
        .set_default_schema_variant_id(txn, nats, visibility, history_actor, Some(*variant.id()))
        .await
}
