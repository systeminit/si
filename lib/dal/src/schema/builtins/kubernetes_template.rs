use crate::schema::builtins::create_prop;
use crate::schema::builtins::kubernetes_metadata::create_metadata_prop;
use crate::schema::builtins::kubernetes_spec::create_spec_prop;
use crate::schema::SchemaResult;
use crate::{HistoryActor, Prop, PropId, PropKind, StandardModel, Visibility, WriteTenancy};
use si_data::{NatsTxn, PgTxn};
use veritech::EncryptionKey;

#[allow(clippy::too_many_arguments)]
pub async fn create_template_prop(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    write_tenancy: &WriteTenancy,
    visibility: &Visibility,
    history_actor: &HistoryActor,
    parent_prop_id: Option<PropId>,
    veritech: veritech::Client,
    encryption_key: &EncryptionKey,
) -> SchemaResult<Prop> {
    let template_prop = create_prop(
        txn,
        nats,
        veritech.clone(),
        encryption_key,
        write_tenancy,
        visibility,
        history_actor,
        "template",
        PropKind::Object,
        parent_prop_id,
    )
    .await?;

    {
        let _optional_metadata_prop = create_metadata_prop(
            txn,
            nats,
            write_tenancy,
            visibility,
            history_actor,
            false,
            Some(*template_prop.id()),
            veritech.clone(),
            encryption_key,
        )
        .await?;
    }

    {
        let _spec_prop = create_spec_prop(
            txn,
            nats,
            veritech,
            encryption_key,
            write_tenancy,
            visibility,
            history_actor,
            *template_prop.id(),
        )
        .await?;
    }
    Ok(template_prop)
}
