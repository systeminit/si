use crate::builtins::schema::BuiltinSchemaHelpers;
use crate::{builtins::BuiltinsResult, DalContext, Prop, PropId, PropKind, StandardModel};

#[allow(dead_code)]
pub async fn create_spec_prop(
    ctx: &DalContext<'_, '_, '_>,
    parent_prop_id: PropId,
) -> BuiltinsResult<Prop> {
    let spec_prop =
        BuiltinSchemaHelpers::create_prop(ctx, "spec", PropKind::Object, Some(parent_prop_id))
            .await?;

    {
        let containers_prop = BuiltinSchemaHelpers::create_prop(
            ctx,
            "containers",
            PropKind::Array, // How to specify it as an array of objects?
            Some(*spec_prop.id()),
        )
        .await?;

        {
            // Do we want default values here?
            let _name_prop = BuiltinSchemaHelpers::create_prop(
                ctx,
                "name",
                PropKind::String,
                Some(*containers_prop.id()),
            )
            .await?;
        }

        {
            let _image_prop = BuiltinSchemaHelpers::create_prop(
                ctx,
                "image",
                PropKind::String,
                Some(*containers_prop.id()),
            )
            .await?;
        }

        {
            // si-registry has some editPartials, but I'm not clear what are they
            let env_prop = BuiltinSchemaHelpers::create_prop(
                ctx,
                "env",
                PropKind::Array, // How to specify it as an array of objects?
                Some(*containers_prop.id()),
            )
            .await?;

            {
                let _name_prop = BuiltinSchemaHelpers::create_prop(
                    ctx,
                    "name",
                    PropKind::String,
                    Some(*env_prop.id()),
                )
                .await?;
            }

            {
                let _value_prop = BuiltinSchemaHelpers::create_prop(
                    ctx,
                    "value",
                    PropKind::String,
                    Some(*env_prop.id()),
                )
                .await?;
            }

            {
                let _value_from_prop = BuiltinSchemaHelpers::create_prop(
                    ctx,
                    "valueFrom",
                    PropKind::Object,
                    Some(*env_prop.id()),
                )
                .await?;

                {
                    let _secret_key_ref_prop = BuiltinSchemaHelpers::create_prop(
                        ctx,
                        "secretKeyRef",
                        PropKind::Object,
                        Some(*_value_from_prop.id()),
                    )
                    .await?;

                    {
                        let _name_prop = BuiltinSchemaHelpers::create_prop(
                            ctx,
                            "name",
                            PropKind::String,
                            Some(*_secret_key_ref_prop.id()),
                        )
                        .await?;
                    }

                    {
                        let _key_prop = BuiltinSchemaHelpers::create_prop(
                            ctx,
                            "key",
                            PropKind::String,
                            Some(*_secret_key_ref_prop.id()),
                        )
                        .await?;
                    }

                    {
                        let _optional_prop = BuiltinSchemaHelpers::create_prop(
                            ctx,
                            "optional",
                            PropKind::Boolean,
                            Some(*_secret_key_ref_prop.id()),
                        )
                        .await?;
                    }
                }

                {
                    let _config_map_ref_prop = BuiltinSchemaHelpers::create_prop(
                        ctx,
                        "configMapRef",
                        PropKind::Object,
                        Some(*_value_from_prop.id()),
                    )
                    .await?;

                    {
                        let _name_prop = BuiltinSchemaHelpers::create_prop(
                            ctx,
                            "name",
                            PropKind::String,
                            Some(*_config_map_ref_prop.id()),
                        )
                        .await?;
                    }

                    {
                        let _key_prop = BuiltinSchemaHelpers::create_prop(
                            ctx,
                            "key",
                            PropKind::String,
                            Some(*_config_map_ref_prop.id()),
                        )
                        .await?;
                    }

                    {
                        let _optional_prop = BuiltinSchemaHelpers::create_prop(
                            ctx,
                            "optional",
                            PropKind::Boolean,
                            Some(*_config_map_ref_prop.id()),
                        )
                        .await?;
                    }
                }

                {
                    let _resource_field_ref_prop = BuiltinSchemaHelpers::create_prop(
                        ctx,
                        "resourceFieldRef",
                        PropKind::Object,
                        Some(*_value_from_prop.id()),
                    )
                    .await?;

                    {
                        let _container_name_prop = BuiltinSchemaHelpers::create_prop(
                            ctx,
                            "containerName",
                            PropKind::String,
                            Some(*_resource_field_ref_prop.id()),
                        )
                        .await?;
                    }

                    {
                        let _resource_prop = BuiltinSchemaHelpers::create_prop(
                            ctx,
                            "resource",
                            PropKind::String,
                            Some(*_resource_field_ref_prop.id()),
                        )
                        .await?;
                    }

                    {
                        let _divisor_prop = BuiltinSchemaHelpers::create_prop(
                            ctx,
                            "divisor",
                            PropKind::String,
                            Some(*_resource_field_ref_prop.id()),
                        )
                        .await?;
                    }
                }

                {
                    let _field_ref_prop = BuiltinSchemaHelpers::create_prop(
                        ctx,
                        "fieldRef",
                        PropKind::Object,
                        Some(*_value_from_prop.id()),
                    )
                    .await?;

                    {
                        // TODO: this should be autopopulated
                        let _api_version_prop = BuiltinSchemaHelpers::create_prop(
                            ctx,
                            "apiVersion",
                            PropKind::String,
                            Some(*_field_ref_prop.id()),
                        )
                        .await?;
                    }

                    {
                        let _field_path_prop = BuiltinSchemaHelpers::create_prop(
                            ctx,
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
            let _image_pull_policy_prop = BuiltinSchemaHelpers::create_prop(
                ctx,
                "imagePullPolicy",
                PropKind::String,
                Some(*containers_prop.id()),
            )
            .await?;
        }

        {
            let ports_prop = BuiltinSchemaHelpers::create_prop(
                ctx,
                "ports",
                PropKind::Array, // How to specify it as an array of objects?
                Some(*containers_prop.id()),
            )
            .await?;

            {
                let _name_prop = BuiltinSchemaHelpers::create_prop(
                    ctx,
                    "name",
                    PropKind::String,
                    Some(*ports_prop.id()),
                )
                .await?;
            }

            {
                // TODO: int from 0 to 65536
                let _container_port_prop = BuiltinSchemaHelpers::create_prop(
                    ctx,
                    "containerPort",
                    PropKind::Integer,
                    Some(*ports_prop.id()),
                )
                .await?;
            }

            {
                let _host_ip_prop = BuiltinSchemaHelpers::create_prop(
                    ctx,
                    "hostIp",
                    PropKind::String,
                    Some(*ports_prop.id()),
                )
                .await?;
            }

            {
                // TODO: int from 0 to 65536
                let _host_port_prop = BuiltinSchemaHelpers::create_prop(
                    ctx,
                    "hostPort",
                    PropKind::Integer,
                    Some(*ports_prop.id()),
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
                    Some(*ports_prop.id()),
                )
                .await?;
            }
        }

        {
            let volume_mounts_prop = BuiltinSchemaHelpers::create_prop(
                ctx,
                "volumeMounts",
                PropKind::Array, // How to specify it as an array of objects?
                Some(*containers_prop.id()),
            )
            .await?;

            {
                let _name_prop = BuiltinSchemaHelpers::create_prop(
                    ctx,
                    "name",
                    PropKind::String,
                    Some(*volume_mounts_prop.id()),
                )
                .await?;
            }

            {
                let _mount_path_prop = BuiltinSchemaHelpers::create_prop(
                    ctx,
                    "mountPath",
                    PropKind::String,
                    Some(*volume_mounts_prop.id()),
                )
                .await?;
            }
        }
    }

    {
        let volumes_prop = BuiltinSchemaHelpers::create_prop(
            ctx,
            "volumes",
            PropKind::Array, // How to specify it as an array of objects?
            Some(*spec_prop.id()),
        )
        .await?;

        {
            let _name_prop = BuiltinSchemaHelpers::create_prop(
                ctx,
                "name",
                PropKind::String,
                Some(*volumes_prop.id()),
            )
            .await?;
        }

        {
            let config_map_prop = BuiltinSchemaHelpers::create_prop(
                ctx,
                "configMap",
                PropKind::Object,
                Some(*volumes_prop.id()),
            )
            .await?;

            {
                let _name_prop = BuiltinSchemaHelpers::create_prop(
                    ctx,
                    "name",
                    PropKind::String,
                    Some(*config_map_prop.id()),
                )
                .await?;
            }
        }
    }

    {
        let image_pull_secrets_prop = BuiltinSchemaHelpers::create_prop(
            ctx,
            "imagePullSecrets",
            PropKind::Array, // How to specify it as an array of objects?
            Some(*spec_prop.id()),
        )
        .await?;

        {
            let _name_prop = BuiltinSchemaHelpers::create_prop(
                ctx,
                "name",
                PropKind::String,
                Some(*image_pull_secrets_prop.id()),
            )
            .await?;
        }
    }

    Ok(spec_prop)
}
