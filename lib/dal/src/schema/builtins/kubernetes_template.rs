use crate::schema::builtins::create_prop;
use crate::schema::builtins::kubernetes_metadata::create_metadata_prop;
use crate::schema::builtins::kubernetes_spec::create_spec_prop;
use crate::schema::SchemaResult;
use crate::{
    HistoryActor, Prop, PropId, PropKind, SchemaVariantId, StandardModel, Tenancy, Visibility,
};
use si_data::{NatsTxn, PgTxn};
use veritech::EncryptionKey;

#[allow(clippy::too_many_arguments)]
pub async fn create_template_prop(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    tenancy: &Tenancy,
    visibility: &Visibility,
    history_actor: &HistoryActor,
    variant_id: &SchemaVariantId,
    parent_prop_id: Option<PropId>,
    veritech: veritech::Client,
    encryption_key: &EncryptionKey,
) -> SchemaResult<Prop> {
    let template_prop = create_prop(
        txn,
        nats,
        veritech.clone(),
        encryption_key,
        tenancy,
        visibility,
        history_actor,
        variant_id,
        "template",
        PropKind::Object,
        parent_prop_id,
    )
    .await?;

    {
        let _optional_metadata_prop = create_metadata_prop(
            txn,
            nats,
            tenancy,
            visibility,
            history_actor,
            variant_id,
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
            tenancy,
            visibility,
            history_actor,
            variant_id,
            *template_prop.id(),
        )
        .await?;
    }
    Ok(template_prop)
}
