use crate::schema::builtins::create_prop;
use crate::schema::SchemaResult;
use crate::{
    HistoryActor, Prop, PropId, PropKind, SchemaVariantId, StandardModel, Tenancy, Visibility,
};
use si_data::{NatsTxn, PgTxn};

#[allow(clippy::too_many_arguments)]
pub async fn create_selector_prop(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    veritech: veritech::Client,
    tenancy: &Tenancy,
    visibility: &Visibility,
    history_actor: &HistoryActor,
    variant_id: &SchemaVariantId,
    parent_prop_id: Option<PropId>,
) -> SchemaResult<Prop> {
    let selector_prop = create_prop(
        txn,
        nats,
        veritech.clone(),
        tenancy,
        visibility,
        history_actor,
        variant_id,
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
            tenancy,
            visibility,
            history_actor,
            variant_id,
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
                tenancy,
                visibility,
                history_actor,
                variant_id,
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
                tenancy,
                visibility,
                history_actor,
                variant_id,
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
                tenancy,
                visibility,
                history_actor,
                variant_id,
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
            tenancy,
            visibility,
            history_actor,
            variant_id,
            "matchLabels",
            PropKind::Array, // How to specify it as an array of strings?
            Some(*selector_prop.id()),
        )
        .await?;
    }
    Ok(selector_prop)
}
