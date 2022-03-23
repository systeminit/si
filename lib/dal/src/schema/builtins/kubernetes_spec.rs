use crate::schema::builtins::create_prop;
use crate::schema::SchemaResult;
use crate::{HistoryActor, Prop, PropId, PropKind, StandardModel, Visibility, WriteTenancy};
use si_data::{NatsTxn, PgTxn};
use veritech::EncryptionKey;

#[allow(clippy::too_many_arguments)]
pub async fn create_spec_prop(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    veritech: veritech::Client,
    encryption_key: &EncryptionKey,
    write_tenancy: &WriteTenancy,
    visibility: &Visibility,
    history_actor: &HistoryActor,
    parent_prop_id: PropId,
) -> SchemaResult<Prop> {
    let spec_prop = create_prop(
        txn,
        nats,
        veritech.clone(),
        encryption_key,
        write_tenancy,
        visibility,
        history_actor,
        "spec",
        PropKind::Object,
        Some(parent_prop_id),
    )
    .await?;

    {
        let containers_prop = create_prop(
            txn,
            nats,
            veritech.clone(),
            encryption_key,
            write_tenancy,
            visibility,
            history_actor,
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
                veritech.clone(),
                encryption_key,
                write_tenancy,
                visibility,
                history_actor,
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
                veritech.clone(),
                encryption_key,
                write_tenancy,
                visibility,
                history_actor,
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
                veritech.clone(),
                encryption_key,
                write_tenancy,
                visibility,
                history_actor,
                "env",
                PropKind::Array, // How to specify it as an array of objects?
                Some(*containers_prop.id()),
            )
            .await?;

            {
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
                    Some(*env_prop.id()),
                )
                .await?;
            }

            {
                let _value_prop = create_prop(
                    txn,
                    nats,
                    veritech.clone(),
                    encryption_key,
                    write_tenancy,
                    visibility,
                    history_actor,
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
                    veritech.clone(),
                    encryption_key,
                    write_tenancy,
                    visibility,
                    history_actor,
                    "valueFrom",
                    PropKind::Object,
                    Some(*env_prop.id()),
                )
                .await?;

                {
                    let _secret_key_ref_prop = create_prop(
                        txn,
                        nats,
                        veritech.clone(),
                        encryption_key,
                        write_tenancy,
                        visibility,
                        history_actor,
                        "secretKeyRef",
                        PropKind::Object,
                        Some(*_value_from_prop.id()),
                    )
                    .await?;

                    {
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
                            Some(*_secret_key_ref_prop.id()),
                        )
                        .await?;
                    }

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
                            Some(*_secret_key_ref_prop.id()),
                        )
                        .await?;
                    }

                    {
                        let _optional_prop = create_prop(
                            txn,
                            nats,
                            veritech.clone(),
                            encryption_key,
                            write_tenancy,
                            visibility,
                            history_actor,
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
                        veritech.clone(),
                        encryption_key,
                        write_tenancy,
                        visibility,
                        history_actor,
                        "configMapRef",
                        PropKind::Object,
                        Some(*_value_from_prop.id()),
                    )
                    .await?;

                    {
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
                            Some(*_config_map_ref_prop.id()),
                        )
                        .await?;
                    }

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
                            Some(*_config_map_ref_prop.id()),
                        )
                        .await?;
                    }

                    {
                        let _optional_prop = create_prop(
                            txn,
                            nats,
                            veritech.clone(),
                            encryption_key,
                            write_tenancy,
                            visibility,
                            history_actor,
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
                        veritech.clone(),
                        encryption_key,
                        write_tenancy,
                        visibility,
                        history_actor,
                        "resourceFieldRef",
                        PropKind::Object,
                        Some(*_value_from_prop.id()),
                    )
                    .await?;

                    {
                        let _container_name_prop = create_prop(
                            txn,
                            nats,
                            veritech.clone(),
                            encryption_key,
                            write_tenancy,
                            visibility,
                            history_actor,
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
                            veritech.clone(),
                            encryption_key,
                            write_tenancy,
                            visibility,
                            history_actor,
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
                            veritech.clone(),
                            encryption_key,
                            write_tenancy,
                            visibility,
                            history_actor,
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
                        veritech.clone(),
                        encryption_key,
                        write_tenancy,
                        visibility,
                        history_actor,
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
                            veritech.clone(),
                            encryption_key,
                            write_tenancy,
                            visibility,
                            history_actor,
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
                            veritech.clone(),
                            encryption_key,
                            write_tenancy,
                            visibility,
                            history_actor,
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
                veritech.clone(),
                encryption_key,
                write_tenancy,
                visibility,
                history_actor,
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
                veritech.clone(),
                encryption_key,
                write_tenancy,
                visibility,
                history_actor,
                "ports",
                PropKind::Array, // How to specify it as an array of objects?
                Some(*containers_prop.id()),
            )
            .await?;

            {
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
                    Some(*ports_prop.id()),
                )
                .await?;
            }

            {
                // TODO: int from 0 to 65536
                let _container_port_prop = create_prop(
                    txn,
                    nats,
                    veritech.clone(),
                    encryption_key,
                    write_tenancy,
                    visibility,
                    history_actor,
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
                    veritech.clone(),
                    encryption_key,
                    write_tenancy,
                    visibility,
                    history_actor,
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
                    veritech.clone(),
                    encryption_key,
                    write_tenancy,
                    visibility,
                    history_actor,
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
                    veritech.clone(),
                    encryption_key,
                    write_tenancy,
                    visibility,
                    history_actor,
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
                veritech.clone(),
                encryption_key,
                write_tenancy,
                visibility,
                history_actor,
                "volumeMounts",
                PropKind::Array, // How to specify it as an array of objects?
                Some(*containers_prop.id()),
            )
            .await?;

            {
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
                    Some(*volume_mounts_prop.id()),
                )
                .await?;
            }

            {
                let _mount_path_prop = create_prop(
                    txn,
                    nats,
                    veritech.clone(),
                    encryption_key,
                    write_tenancy,
                    visibility,
                    history_actor,
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
            veritech.clone(),
            encryption_key,
            write_tenancy,
            visibility,
            history_actor,
            "volumes",
            PropKind::Array, // How to specify it as an array of objects?
            Some(*spec_prop.id()),
        )
        .await?;

        {
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
                Some(*volumes_prop.id()),
            )
            .await?;
        }

        {
            let config_map_prop = create_prop(
                txn,
                nats,
                veritech.clone(),
                encryption_key,
                write_tenancy,
                visibility,
                history_actor,
                "configMap",
                PropKind::Object,
                Some(*volumes_prop.id()),
            )
            .await?;

            {
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
            veritech.clone(),
            encryption_key,
            write_tenancy,
            visibility,
            history_actor,
            "imagePullSecrets",
            PropKind::Array, // How to specify it as an array of objects?
            Some(*spec_prop.id()),
        )
        .await?;

        {
            let _name_prop = create_prop(
                txn,
                nats,
                veritech,
                encryption_key,
                write_tenancy,
                visibility,
                history_actor,
                "name",
                PropKind::String,
                Some(*image_pull_secrets_prop.id()),
            )
            .await?;
        }
    }

    Ok(spec_prop)
}
