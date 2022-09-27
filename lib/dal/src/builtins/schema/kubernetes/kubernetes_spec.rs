use crate::builtins::schema::BuiltinSchemaHelpers;
use crate::{builtins::BuiltinsResult, DalContext, Prop, PropId, PropKind, StandardModel};

#[allow(dead_code)]
pub async fn create_spec_prop(ctx: &DalContext, parent_prop_id: PropId) -> BuiltinsResult<Prop> {
    let spec_prop = BuiltinSchemaHelpers::create_prop(
        ctx,
        "spec",
        PropKind::Object,
        None,
        Some(parent_prop_id),
        None,
    )
    .await?;

    {
        let containers_prop = BuiltinSchemaHelpers::create_prop(
            ctx,
            "containers",
            PropKind::Array,
            None,
            Some(*spec_prop.id()),
            None,
        )
        .await?;

        {
            // Do we want default values here?
            let _name_prop = BuiltinSchemaHelpers::create_prop(
                ctx,
                "name",
                PropKind::String,
                None,
                Some(*containers_prop.id()),
                None,
            )
            .await?;
        }

        {
            let _image_prop = BuiltinSchemaHelpers::create_prop(
                ctx,
                "image",
                PropKind::String,
                None,
                Some(*containers_prop.id()),
                None,
            )
            .await?;
        }

        {
            // si-registry has some editPartials, but I'm not clear what are they
            let env_prop = BuiltinSchemaHelpers::create_prop(
                ctx,
                "env",
                PropKind::Array,
                None,
                Some(*containers_prop.id()),
                None,
            )
            .await?;

            {
                let _name_prop = BuiltinSchemaHelpers::create_prop(
                    ctx,
                    "name",
                    PropKind::String,
                    None,
                    Some(*env_prop.id()),
                    None,
                )
                .await?;
            }

            {
                let _value_prop = BuiltinSchemaHelpers::create_prop(
                    ctx,
                    "value",
                    PropKind::String,
                    None,
                    Some(*env_prop.id()),
                    None,
                )
                .await?;
            }

            {
                let _value_from_prop = BuiltinSchemaHelpers::create_prop(
                    ctx,
                    "valueFrom",
                    PropKind::Object,
                    None,
                    Some(*env_prop.id()),
                    None,
                )
                .await?;

                {
                    let _secret_key_ref_prop = BuiltinSchemaHelpers::create_prop(
                        ctx,
                        "secretKeyRef",
                        PropKind::Object,
                        None,
                        Some(*_value_from_prop.id()),
                        None,
                    )
                    .await?;

                    {
                        let _name_prop = BuiltinSchemaHelpers::create_prop(
                            ctx,
                            "name",
                            PropKind::String,
                            None,
                            Some(*_secret_key_ref_prop.id()),
                            None,
                        )
                        .await?;
                    }

                    {
                        let _key_prop = BuiltinSchemaHelpers::create_prop(
                            ctx,
                            "key",
                            PropKind::String,
                            None,
                            Some(*_secret_key_ref_prop.id()),
                            None,
                        )
                        .await?;
                    }

                    {
                        let _optional_prop = BuiltinSchemaHelpers::create_prop(
                            ctx,
                            "optional",
                            PropKind::Boolean,
                            None,
                            Some(*_secret_key_ref_prop.id()),
                            None,
                        )
                        .await?;
                    }
                }

                {
                    let _config_map_ref_prop = BuiltinSchemaHelpers::create_prop(
                        ctx,
                        "configMapRef",
                        PropKind::Object,
                        None,
                        Some(*_value_from_prop.id()),
                        None,
                    )
                    .await?;

                    {
                        let _name_prop = BuiltinSchemaHelpers::create_prop(
                            ctx,
                            "name",
                            PropKind::String,
                            None,
                            Some(*_config_map_ref_prop.id()),
                            None,
                        )
                        .await?;
                    }

                    {
                        let _key_prop = BuiltinSchemaHelpers::create_prop(
                            ctx,
                            "key",
                            PropKind::String,
                            None,
                            Some(*_config_map_ref_prop.id()),
                            None,
                        )
                        .await?;
                    }

                    {
                        let _optional_prop = BuiltinSchemaHelpers::create_prop(
                            ctx,
                            "optional",
                            PropKind::Boolean,
                            None,
                            Some(*_config_map_ref_prop.id()),
                            None,
                        )
                        .await?;
                    }
                }

                {
                    let _resource_field_ref_prop = BuiltinSchemaHelpers::create_prop(
                        ctx,
                        "resourceFieldRef",
                        PropKind::Object,
                        None,
                        Some(*_value_from_prop.id()),
                        None,
                    )
                    .await?;

                    {
                        let _container_name_prop = BuiltinSchemaHelpers::create_prop(
                            ctx,
                            "containerName",
                            PropKind::String,
                            None,
                            Some(*_resource_field_ref_prop.id()),
                            None,
                        )
                        .await?;
                    }

                    {
                        let _resource_prop = BuiltinSchemaHelpers::create_prop(
                            ctx,
                            "resource",
                            PropKind::String,
                            None,
                            Some(*_resource_field_ref_prop.id()),
                            None,
                        )
                        .await?;
                    }

                    {
                        let _divisor_prop = BuiltinSchemaHelpers::create_prop(
                            ctx,
                            "divisor",
                            PropKind::String,
                            None,
                            Some(*_resource_field_ref_prop.id()),
                            None,
                        )
                        .await?;
                    }
                }

                {
                    let _field_ref_prop = BuiltinSchemaHelpers::create_prop(
                        ctx,
                        "fieldRef",
                        PropKind::Object,
                        None,
                        Some(*_value_from_prop.id()),
                        None,
                    )
                    .await?;

                    {
                        // TODO: this should be autopopulated
                        let _api_version_prop = BuiltinSchemaHelpers::create_prop(
                            ctx,
                            "apiVersion",
                            PropKind::String,
                            None,
                            Some(*_field_ref_prop.id()),
                            None,
                        )
                        .await?;
                    }

                    {
                        let _field_path_prop = BuiltinSchemaHelpers::create_prop(
                            ctx,
                            "fieldPath",
                            PropKind::String,
                            None,
                            Some(*_field_ref_prop.id()),
                            None,
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
            let _image_pull_policy_prop = BuiltinSchemaHelpers::create_prop(
                ctx,
                "imagePullPolicy",
                PropKind::String,
                None,
                Some(*containers_prop.id()),
                None,
            )
            .await?;
        }

        {
            let ports_prop = BuiltinSchemaHelpers::create_prop(
                ctx,
                "ports",
                PropKind::Array,
                None,
                Some(*containers_prop.id()),
                None,
            )
            .await?;

            {
                let _name_prop = BuiltinSchemaHelpers::create_prop(
                    ctx,
                    "name",
                    PropKind::String,
                    None,
                    Some(*ports_prop.id()),
                    None,
                )
                .await?;
            }

            {
                // TODO: int from 0 to 65536
                let _container_port_prop = BuiltinSchemaHelpers::create_prop(
                    ctx,
                    "containerPort",
                    PropKind::Integer,
                    None,
                    Some(*ports_prop.id()),
                    None,
                )
                .await?;
            }

            {
                let _host_ip_prop = BuiltinSchemaHelpers::create_prop(
                    ctx,
                    "hostIp",
                    PropKind::String,
                    None,
                    Some(*ports_prop.id()),
                    None,
                )
                .await?;
            }

            {
                // TODO: int from 0 to 65536
                let _host_port_prop = BuiltinSchemaHelpers::create_prop(
                    ctx,
                    "hostPort",
                    PropKind::Integer,
                    None,
                    Some(*ports_prop.id()),
                    None,
                )
                .await?;
            }

            {
                // TODO: validate to ensure it's either "TCP", "UDP" or "SCTP"
                // Is there a selector widget? If so how to enable it
                let _protocol_prop = BuiltinSchemaHelpers::create_prop(
                    ctx,
                    "protocol",
                    PropKind::String,
                    None,
                    Some(*ports_prop.id()),
                    None,
                )
                .await?;
            }
        }

        {
            let volume_mounts_prop = BuiltinSchemaHelpers::create_prop(
                ctx,
                "volumeMounts",
                PropKind::Array,
                None,
                Some(*containers_prop.id()),
                None,
            )
            .await?;

            {
                let _name_prop = BuiltinSchemaHelpers::create_prop(
                    ctx,
                    "name",
                    PropKind::String,
                    None,
                    Some(*volume_mounts_prop.id()),
                    None,
                )
                .await?;
            }

            {
                let _mount_path_prop = BuiltinSchemaHelpers::create_prop(
                    ctx,
                    "mountPath",
                    PropKind::String,
                    None,
                    Some(*volume_mounts_prop.id()),
                    None,
                )
                .await?;
            }
        }
    }

    {
        let volumes_prop = BuiltinSchemaHelpers::create_prop(
            ctx,
            "volumes",
            PropKind::Array,
            None,
            Some(*spec_prop.id()),
            None,
        )
        .await?;

        {
            let _name_prop = BuiltinSchemaHelpers::create_prop(
                ctx,
                "name",
                PropKind::String,
                None,
                Some(*volumes_prop.id()),
                None,
            )
            .await?;
        }

        {
            let config_map_prop = BuiltinSchemaHelpers::create_prop(
                ctx,
                "configMap",
                PropKind::Object,
                None,
                Some(*volumes_prop.id()),
                None,
            )
            .await?;

            {
                let _name_prop = BuiltinSchemaHelpers::create_prop(
                    ctx,
                    "name",
                    PropKind::String,
                    None,
                    Some(*config_map_prop.id()),
                    None,
                )
                .await?;
            }
        }
    }

    {
        let image_pull_secrets_prop = BuiltinSchemaHelpers::create_prop(
            ctx,
            "imagePullSecrets",
            PropKind::Array,
            None,
            Some(*spec_prop.id()),
            None,
        )
        .await?;

        {
            let _name_prop = BuiltinSchemaHelpers::create_prop(
                ctx,
                "name",
                PropKind::String,
                None,
                Some(*image_pull_secrets_prop.id()),
                None,
            )
            .await?;
        }
    }

    Ok(spec_prop)
}
