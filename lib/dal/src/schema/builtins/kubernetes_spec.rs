use crate::schema::builtins::create_prop;
use crate::schema::SchemaResult;
use crate::{
    HistoryActor, Prop, PropId, PropKind, SchemaVariantId, StandardModel, Tenancy, Visibility,
};
use si_data::{NatsTxn, PgTxn};

pub async fn create_spec_prop(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    tenancy: &Tenancy,
    visibility: &Visibility,
    history_actor: &HistoryActor,
    variant_id: &SchemaVariantId,
    parent_prop_id: PropId,
) -> SchemaResult<Prop> {
    let spec_prop = create_prop(
        txn,
        nats,
        tenancy,
        visibility,
        history_actor,
        variant_id,
        "spec",
        PropKind::Object,
        Some(parent_prop_id),
    )
    .await?;

    {
        let containers_prop = create_prop(
            txn,
            nats,
            tenancy,
            visibility,
            history_actor,
            variant_id,
            "containers",
            PropKind::Array, // How to specify it as an array of objects?
            Some(*spec_prop.id()),
        )
        .await?;

        {
            // Do we want default values here?
            let _name_prop = create_prop(
                txn,
                nats,
                tenancy,
                visibility,
                history_actor,
                variant_id,
                "name",
                PropKind::String,
                Some(*containers_prop.id()),
            )
            .await?;
        }

        {
            let _image_prop = create_prop(
                txn,
                nats,
                tenancy,
                visibility,
                history_actor,
                variant_id,
                "image",
                PropKind::String,
                Some(*containers_prop.id()),
            )
            .await?;
        }

        {
            // si-registry has some editPartials, but I'm not clear what are they
            let env_prop = create_prop(
                txn,
                nats,
                tenancy,
                visibility,
                history_actor,
                variant_id,
                "env",
                PropKind::Array, // How to specify it as an array of objects?
                Some(*containers_prop.id()),
            )
            .await?;

            {
                let _name_prop = create_prop(
                    txn,
                    nats,
                    tenancy,
                    visibility,
                    history_actor,
                    variant_id,
                    "name",
                    PropKind::String,
                    Some(*env_prop.id()),
                )
                .await?;
            }

            {
                let _value_prop = create_prop(
                    txn,
                    nats,
                    tenancy,
                    visibility,
                    history_actor,
                    variant_id,
                    "value",
                    PropKind::String,
                    Some(*env_prop.id()),
                )
                .await?;
            }

            {
                let _value_from_prop = create_prop(
                    txn,
                    nats,
                    tenancy,
                    visibility,
                    history_actor,
                    variant_id,
                    "valueFrom",
                    PropKind::Object,
                    Some(*env_prop.id()),
                )
                .await?;

                {
                    let _secret_key_ref_prop = create_prop(
                        txn,
                        nats,
                        tenancy,
                        visibility,
                        history_actor,
                        variant_id,
                        "secretKeyRef",
                        PropKind::Object,
                        Some(*_value_from_prop.id()),
                    )
                    .await?;

                    {
                        let _name_prop = create_prop(
                            txn,
                            nats,
                            tenancy,
                            visibility,
                            history_actor,
                            variant_id,
                            "name",
                            PropKind::String,
                            Some(*_secret_key_ref_prop.id()),
                        )
                        .await?;
                    }

                    {
                        let _key_prop = create_prop(
                            txn,
                            nats,
                            tenancy,
                            visibility,
                            history_actor,
                            variant_id,
                            "key",
                            PropKind::String,
                            Some(*_secret_key_ref_prop.id()),
                        )
                        .await?;
                    }

                    {
                        let _optional_prop = create_prop(
                            txn,
                            nats,
                            tenancy,
                            visibility,
                            history_actor,
                            variant_id,
                            "optional",
                            PropKind::Boolean,
                            Some(*_secret_key_ref_prop.id()),
                        )
                        .await?;
                    }
                }

                {
                    let _config_map_ref_prop = create_prop(
                        txn,
                        nats,
                        tenancy,
                        visibility,
                        history_actor,
                        variant_id,
                        "configMapRef",
                        PropKind::Object,
                        Some(*_value_from_prop.id()),
                    )
                    .await?;

                    {
                        let _name_prop = create_prop(
                            txn,
                            nats,
                            tenancy,
                            visibility,
                            history_actor,
                            variant_id,
                            "name",
                            PropKind::String,
                            Some(*_config_map_ref_prop.id()),
                        )
                        .await?;
                    }

                    {
                        let _key_prop = create_prop(
                            txn,
                            nats,
                            tenancy,
                            visibility,
                            history_actor,
                            variant_id,
                            "key",
                            PropKind::String,
                            Some(*_config_map_ref_prop.id()),
                        )
                        .await?;
                    }

                    {
                        let _optional_prop = create_prop(
                            txn,
                            nats,
                            tenancy,
                            visibility,
                            history_actor,
                            variant_id,
                            "optional",
                            PropKind::Boolean,
                            Some(*_config_map_ref_prop.id()),
                        )
                        .await?;
                    }
                }

                {
                    let _resource_field_ref_prop = create_prop(
                        txn,
                        nats,
                        tenancy,
                        visibility,
                        history_actor,
                        variant_id,
                        "resourceFieldRef",
                        PropKind::Object,
                        Some(*_value_from_prop.id()),
                    )
                    .await?;

                    {
                        let _container_name_prop = create_prop(
                            txn,
                            nats,
                            tenancy,
                            visibility,
                            history_actor,
                            variant_id,
                            "containerName",
                            PropKind::String,
                            Some(*_resource_field_ref_prop.id()),
                        )
                        .await?;
                    }

                    {
                        let _resource_prop = create_prop(
                            txn,
                            nats,
                            tenancy,
                            visibility,
                            history_actor,
                            variant_id,
                            "resource",
                            PropKind::String,
                            Some(*_resource_field_ref_prop.id()),
                        )
                        .await?;
                    }

                    {
                        let _divisor_prop = create_prop(
                            txn,
                            nats,
                            tenancy,
                            visibility,
                            history_actor,
                            variant_id,
                            "divisor",
                            PropKind::String,
                            Some(*_resource_field_ref_prop.id()),
                        )
                        .await?;
                    }
                }

                {
                    let _field_ref_prop = create_prop(
                        txn,
                        nats,
                        tenancy,
                        visibility,
                        history_actor,
                        variant_id,
                        "fieldRef",
                        PropKind::Object,
                        Some(*_value_from_prop.id()),
                    )
                    .await?;

                    {
                        // TODO: this should be autopopulated
                        let _api_version_prop = create_prop(
                            txn,
                            nats,
                            tenancy,
                            visibility,
                            history_actor,
                            variant_id,
                            "apiVersion",
                            PropKind::String,
                            Some(*_field_ref_prop.id()),
                        )
                        .await?;
                    }

                    {
                        let _field_path_prop = create_prop(
                            txn,
                            nats,
                            tenancy,
                            visibility,
                            history_actor,
                            variant_id,
                            "fieldPath",
                            PropKind::String,
                            Some(*_field_ref_prop.id()),
                        )
                        .await?;
                    }
                }
            }
        }

        {
            // TODO: validate to ensure it's either "Always", "Never", "IfNotPresent"
            // Is there a selector widget? If so how to enable it
            // TODO: required
            let _image_pull_policy_prop = create_prop(
                txn,
                nats,
                tenancy,
                visibility,
                history_actor,
                variant_id,
                "imagePullPolicy",
                PropKind::String,
                Some(*containers_prop.id()),
            )
            .await?;
        }

        {
            let ports_prop = create_prop(
                txn,
                nats,
                tenancy,
                visibility,
                history_actor,
                variant_id,
                "ports",
                PropKind::Array, // How to specify it as an array of objects?
                Some(*containers_prop.id()),
            )
            .await?;

            {
                let _name_prop = create_prop(
                    txn,
                    nats,
                    tenancy,
                    visibility,
                    history_actor,
                    variant_id,
                    "name",
                    PropKind::String,
                    Some(*ports_prop.id()),
                )
                .await?;
            }

            {
                // TODO: int from 0 to 65536
                let _container_port_prop = create_prop(
                    txn,
                    nats,
                    tenancy,
                    visibility,
                    history_actor,
                    variant_id,
                    "containerPort",
                    PropKind::Integer,
                    Some(*ports_prop.id()),
                )
                .await?;
            }

            {
                let _host_ip_prop = create_prop(
                    txn,
                    nats,
                    tenancy,
                    visibility,
                    history_actor,
                    variant_id,
                    "hostIp",
                    PropKind::String,
                    Some(*ports_prop.id()),
                )
                .await?;
            }

            {
                // TODO: int from 0 to 65536
                let _host_port_prop = create_prop(
                    txn,
                    nats,
                    tenancy,
                    visibility,
                    history_actor,
                    variant_id,
                    "hostPort",
                    PropKind::Integer,
                    Some(*ports_prop.id()),
                )
                .await?;
            }

            {
                // TODO: validate to ensure it's either "TCP", "UDP" or "SCTP"
                // Is there a selector widget? If so how to enable it
                let _protocol_prop = create_prop(
                    txn,
                    nats,
                    tenancy,
                    visibility,
                    history_actor,
                    variant_id,
                    "protocol",
                    PropKind::String,
                    Some(*ports_prop.id()),
                )
                .await?;
            }
        }

        {
            let volume_mounts_prop = create_prop(
                txn,
                nats,
                tenancy,
                visibility,
                history_actor,
                variant_id,
                "volumeMounts",
                PropKind::Array, // How to specify it as an array of objects?
                Some(*containers_prop.id()),
            )
            .await?;

            {
                let _name_prop = create_prop(
                    txn,
                    nats,
                    tenancy,
                    visibility,
                    history_actor,
                    variant_id,
                    "name",
                    PropKind::String,
                    Some(*volume_mounts_prop.id()),
                )
                .await?;
            }

            {
                let _mount_path_prop = create_prop(
                    txn,
                    nats,
                    tenancy,
                    visibility,
                    history_actor,
                    variant_id,
                    "mountPath",
                    PropKind::String,
                    Some(*volume_mounts_prop.id()),
                )
                .await?;
            }
        }
    }

    {
        let volumes_prop = create_prop(
            txn,
            nats,
            tenancy,
            visibility,
            history_actor,
            variant_id,
            "volumes",
            PropKind::Array, // How to specify it as an array of objects?
            Some(*spec_prop.id()),
        )
        .await?;

        {
            let _name_prop = create_prop(
                txn,
                nats,
                tenancy,
                visibility,
                history_actor,
                variant_id,
                "name",
                PropKind::String,
                Some(*volumes_prop.id()),
            )
            .await?;
        }

        {
            let config_map_prop = create_prop(
                txn,
                nats,
                tenancy,
                visibility,
                history_actor,
                variant_id,
                "configMap",
                PropKind::Object,
                Some(*volumes_prop.id()),
            )
            .await?;

            {
                let _name_prop = create_prop(
                    txn,
                    nats,
                    tenancy,
                    visibility,
                    history_actor,
                    variant_id,
                    "name",
                    PropKind::String,
                    Some(*config_map_prop.id()),
                )
                .await?;
            }
        }
    }

    {
        let image_pull_secrets_prop = create_prop(
            txn,
            nats,
            tenancy,
            visibility,
            history_actor,
            variant_id,
            "imagePullSecrets",
            PropKind::Array, // How to specify it as an array of objects?
            Some(*spec_prop.id()),
        )
        .await?;

        {
            let _name_prop = create_prop(
                txn,
                nats,
                tenancy,
                visibility,
                history_actor,
                variant_id,
                "name",
                PropKind::String,
                Some(*image_pull_secrets_prop.id()),
            )
            .await?;
        }
    }

    Ok(spec_prop)
}
