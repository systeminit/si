use crate::schema::builtins::create_prop;
use crate::schema::SchemaResult;
use crate::{HistoryActor, Prop, PropId, PropKind, StandardModel, Visibility, WriteTenancy};
use si_data::{NatsTxn, PgTxn};
use veritech::EncryptionKey;

#[allow(clippy::too_many_arguments)]
pub async fn create_selector_prop(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    veritech: veritech::Client,
    encryption_key: &EncryptionKey,
    write_tenancy: &WriteTenancy,
    visibility: &Visibility,
    history_actor: &HistoryActor,
    parent_prop_id: Option<PropId>,
) -> SchemaResult<Prop> {
    let selector_prop = create_prop(
        txn,
        nats,
        veritech.clone(),
        encryption_key,
        write_tenancy,
        visibility,
        history_actor,
        "selector",
        PropKind::Object,
        parent_prop_id,
    )
    .await?;

    {
        let match_expressions_prop = create_prop(
            txn,
            nats,
            veritech.clone(),
            encryption_key,
            write_tenancy,
            visibility,
            history_actor,
            "matchExpressions",
            PropKind::Array, // How to specify it as an array of objects?
            Some(*selector_prop.id()),
        )
        .await?;

        {
            let _key_prop = create_prop(
                txn,
                nats,
                veritech.clone(),
                encryption_key,
                write_tenancy,
                visibility,
                history_actor,
                "key",
                PropKind::String,
                Some(*match_expressions_prop.id()),
            )
            .await?;
        }

        {
            // TODO: validate to ensure it's either "In", "NotInt", "Exists", "DoesNotExist"
            // Is there a selector widget? If so how to enable it
            let _operator_prop = create_prop(
                txn,
                nats,
                veritech.clone(),
                encryption_key,
                write_tenancy,
                visibility,
                history_actor,
                "operator",
                PropKind::String,
                Some(*match_expressions_prop.id()),
            )
            .await?;
        }

        {
            let _values_prop = create_prop(
                txn,
                nats,
                veritech.clone(),
                encryption_key,
                write_tenancy,
                visibility,
                history_actor,
                "values",
                PropKind::Array, // How to specify it as an array of strings?
                Some(*match_expressions_prop.id()),
            )
            .await?;
        }
    }

    {
        let _match_labels_prop = create_prop(
            txn,
            nats,
            veritech,
            encryption_key,
            write_tenancy,
            visibility,
            history_actor,
            "matchLabels",
            PropKind::Array, // How to specify it as an array of strings?
            Some(*selector_prop.id()),
        )
        .await?;
    }
    Ok(selector_prop)
}
