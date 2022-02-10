use crate::schema::builtins::{create_prop, create_string_prop_with_default};
use crate::schema::SchemaResult;
use crate::{
    HistoryActor, Prop, PropId, PropKind, SchemaVariantId, StandardModel, Tenancy, Visibility,
};
use si_data::{NatsTxn, PgTxn};

#[allow(clippy::too_many_arguments)]
pub async fn create_metadata_prop(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    tenancy: &Tenancy,
    visibility: &Visibility,
    history_actor: &HistoryActor,
    variant_id: &SchemaVariantId,
    schema_name: Option<String>,
    is_schema_name_required: bool,
    parent_prop_id: Option<PropId>,
    veritech: veritech::Client,
) -> SchemaResult<Prop> {
    let metadata_prop = create_prop(
        txn,
        nats,
        veritech.clone(),
        tenancy,
        visibility,
        history_actor,
        variant_id,
        "metadata",
        PropKind::Object,
        parent_prop_id,
    )
    .await?;

    {
        // TODO: add validation
        //validation: [
        //  {
        //    kind: ValidatorKind.Regex,
        //    regex: "^[A-Za-z0-9](?:[A-Za-z0-9-]{0,251}[A-Za-z0-9])?$",
        //    message: "Kubernetes names must be valid DNS subdomains",
        //    link:
        //      "https://kubernetes.io/docs/concepts/overview/working-with-objects/names/#dns-subdomain-names",
        //  },
        //],
        if is_schema_name_required {
            // TODO: add a required field validation here
        }

        let _name_prop = if let Some(schema_name) = schema_name {
            // Note: do we actually want this branch to exist?
            create_string_prop_with_default(
                txn,
                nats,
                tenancy,
                visibility,
                history_actor,
                variant_id,
                "name",
                schema_name,
                Some(*metadata_prop.id()),
                veritech.clone(),
            )
            .await?;
        } else {
            create_prop(
                txn,
                nats,
                veritech.clone(),
                tenancy,
                visibility,
                history_actor,
                variant_id,
                "name",
                PropKind::String,
                Some(*metadata_prop.id()),
            )
            .await?;
        };
    }

    {
        let _generate_name_prop = create_prop(
            txn,
            nats,
            veritech.clone(),
            tenancy,
            visibility,
            history_actor,
            variant_id,
            "generateName",
            PropKind::String,
            Some(*metadata_prop.id()),
        )
        .await?;
    }

    {
        // Note: should this come from a k8s namespace component configuring us?
        let _namespace_prop = create_prop(
            txn,
            nats,
            veritech.clone(),
            tenancy,
            visibility,
            history_actor,
            variant_id,
            "namespace",
            PropKind::String,
            Some(*metadata_prop.id()),
        )
        .await?;
    }

    {
        let _labels_prop = create_prop(
            txn,
            nats,
            veritech.clone(),
            tenancy,
            visibility,
            history_actor,
            variant_id,
            "labels",
            PropKind::Map, // How to specify it as a map of string values?
            Some(*metadata_prop.id()),
        )
        .await?;
    }

    {
        let _annotations_prop = create_prop(
            txn,
            nats,
            veritech,
            tenancy,
            visibility,
            history_actor,
            variant_id,
            "annotations",
            PropKind::Map, // How to specify it as a map of string values?
            Some(*metadata_prop.id()),
        )
        .await?;
    }

    Ok(metadata_prop)
}
