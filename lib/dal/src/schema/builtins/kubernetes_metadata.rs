use crate::schema::builtins::create_prop;
use crate::schema::SchemaResult;
use crate::{HistoryActor, Prop, PropId, PropKind, StandardModel, Visibility, WriteTenancy};
use si_data::{NatsTxn, PgTxn};
use veritech::EncryptionKey;

#[allow(clippy::too_many_arguments)]
pub async fn create_metadata_prop(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    write_tenancy: &WriteTenancy,
    visibility: &Visibility,
    history_actor: &HistoryActor,
    is_name_required: bool,
    parent_prop_id: Option<PropId>,
    veritech: veritech::Client,
    encryption_key: &EncryptionKey,
) -> SchemaResult<Prop> {
    let metadata_prop = create_prop(
        txn,
        nats,
        veritech.clone(),
        encryption_key,
        write_tenancy,
        visibility,
        history_actor,
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
        if is_name_required {
            // TODO: add a required field validation here
        }

        let _name_prop = create_prop(
            txn,
            nats,
            veritech.clone(),
            encryption_key,
            write_tenancy,
            visibility,
            history_actor,
            "name",
            PropKind::String,
            Some(*metadata_prop.id()),
        )
        .await?;
    }

    {
        let _generate_name_prop = create_prop(
            txn,
            nats,
            veritech.clone(),
            encryption_key,
            write_tenancy,
            visibility,
            history_actor,
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
            encryption_key,
            write_tenancy,
            visibility,
            history_actor,
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
            encryption_key,
            write_tenancy,
            visibility,
            history_actor,
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
            encryption_key,
            write_tenancy,
            visibility,
            history_actor,
            "annotations",
            PropKind::Map, // How to specify it as a map of string values?
            Some(*metadata_prop.id()),
        )
        .await?;
    }

    Ok(metadata_prop)
}
