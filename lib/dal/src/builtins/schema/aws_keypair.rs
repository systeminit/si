use crate::builtins::schema::MigrationDriver;
use crate::builtins::BuiltinsError;
use crate::component::ComponentKind;
use crate::func::description::FuncDescription;
use crate::property_editor::schema::WidgetKind;
use crate::schema::variant::definition::SchemaVariantDefinitionMetadataJson;
use crate::schema::variant::leaves::LeafKind;
use crate::socket::SocketArity;
use crate::{
    action_prototype::ActionKind,
    schema::variant::leaves::{LeafInput, LeafInputLocation},
    FuncDescriptionContents,
};
use crate::{
    attribute::context::AttributeContextBuilder, func::argument::FuncArgument, ActionPrototype,
    ActionPrototypeContext, AttributePrototype, AttributePrototypeArgument,
    AttributePrototypeError, AttributeReadContext, AttributeValue, BuiltinsResult, DalContext,
    ExternalProvider, Func, InternalProvider, PropKind, SchemaError, StandardModel,
    WorkflowPrototype, WorkflowPrototypeContext,
};
use crate::{AttributeValueError, SchemaVariant};

// Documentation URL(s)
const AWS_REGIONS_DOCS_URL: &str =
    "https://docs.aws.amazon.com/general/latest/gr/rande.html#region-names-codes";
const EC2_TAG_DOCS_URL: &str =
    "https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/Using_Tags.html";
const EC2_KEYPAIR_PROPERTIES_DOCS_URL: &str =
    "https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/aws-resource-ec2-keypair.html#aws-resource-ec2-keypair-properties";

impl MigrationDriver {
    /// A [`Schema`](crate::Schema) migration for [`AWS Key Pair`](https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/aws-resource-ec2-keypair.html).
    pub async fn migrate_aws_keypair(
        &self,
        ctx: &DalContext,
        ui_menu_category: &str,
        node_color: &str,
    ) -> BuiltinsResult<()> {
        let name = "Key Pair";
        let (schema, mut schema_variant, root_prop, _, _, _) = match self
            .create_schema_and_variant_with_name(
                ctx,
                SchemaVariantDefinitionMetadataJson::new(
                    name,
                    None,
                    ui_menu_category,
                    node_color,
                    ComponentKind::Standard,
                    None,
                    None,
                ),
                None,
                "v1",
            )
            .await?
        {
            Some(tuple) => tuple,
            None => return Ok(()),
        };

        // Prop: /root/domain/KeyName
        let key_name_prop = self
            .create_prop(
                ctx,
                "KeyName",
                PropKind::String,
                None,
                Some(root_prop.domain_prop_id),
                Some(EC2_KEYPAIR_PROPERTIES_DOCS_URL.to_string()),
            )
            .await?;

        // Prop: /root/domain/KeyType
        let key_type_prop = self
            .create_prop(
                ctx,
                "KeyType",
                PropKind::String,
                Some((
                    WidgetKind::Select,
                    Some(serde_json::json!([
                        {
                            "label": "rsa",
                            "value": "rsa",
                        },
                        {
                            "label": "ed25519",
                            "value": "ed25519",
                        },
                    ])),
                )),
                Some(root_prop.domain_prop_id),
                Some(EC2_KEYPAIR_PROPERTIES_DOCS_URL.to_string()),
            )
            .await?;

        // Prop: /root/domain/tags
        let tags_map_prop = self
            .create_prop(
                ctx,
                "tags",
                PropKind::Map,
                None,
                Some(root_prop.domain_prop_id),
                Some(EC2_TAG_DOCS_URL.to_string()),
            )
            .await?;

        // Prop: /root/domain/tags/tag
        let tags_map_item_prop = self
            .create_prop(
                ctx,
                "tag",
                PropKind::String,
                None,
                Some(*tags_map_prop.id()),
                Some(EC2_TAG_DOCS_URL.to_string()),
            )
            .await?;

        // Prop: /root/domain/awsResourceType
        let mut aws_resource_type_prop = self
            .create_prop(
                ctx,
                "awsResourceType",
                PropKind::String,
                None,
                Some(root_prop.domain_prop_id),
                None,
            )
            .await?;
        aws_resource_type_prop.set_hidden(ctx, true).await?;

        // Prop: /root/domain/region
        let region_prop = self
            .create_prop(
                ctx,
                "region",
                PropKind::String,
                None,
                Some(root_prop.domain_prop_id),
                Some(AWS_REGIONS_DOCS_URL.to_string()),
            )
            .await?;

        // Add code generation
        let (code_generation_func_id, code_generation_func_argument_id) = self
            .find_func_and_single_argument_by_names(ctx, "si:generateAwsKeyPairJSON", "domain")
            .await?;
        SchemaVariant::add_leaf(
            ctx,
            code_generation_func_id,
            *schema_variant.id(),
            None,
            LeafKind::CodeGeneration,
            vec![LeafInput {
                location: LeafInputLocation::Domain,
                func_argument_id: code_generation_func_argument_id,
            }],
        )
        .await?;

        // Output Socket
        let identity_func_item = self
            .get_func_item("si:identity")
            .ok_or(BuiltinsError::FuncNotFoundInMigrationCache("si:identity"))?;
        let (key_name_external_provider, mut output_socket) = ExternalProvider::new_with_socket(
            ctx,
            *schema.id(),
            *schema_variant.id(),
            "Key Name2",
            None,
            identity_func_item.func_id,
            identity_func_item.func_binding_id,
            identity_func_item.func_binding_return_value_id,
            SocketArity::Many,
            false,
        )
        .await?;
        output_socket.set_human_name(ctx, Some("Key Name")).await?;

        // Input Socket
        let (region_explicit_internal_provider, _input_socket) =
            InternalProvider::new_explicit_with_socket(
                ctx,
                *schema_variant.id(),
                "Region",
                identity_func_item.func_id,
                identity_func_item.func_binding_id,
                identity_func_item.func_binding_return_value_id,
                SocketArity::Many,
                false,
            )
            .await?; // TODO(wendy) - Can a key pair have multiple regions? Idk!

        // Qualifications
        let qualification_func_name = "si:qualificationKeyPairCanCreate";
        let (qualification_func_id, qualification_func_argument_id) = self
            .find_func_and_single_argument_by_names(ctx, qualification_func_name, "domain")
            .await?;
        let code_func_argument =
            FuncArgument::find_by_name_for_func(ctx, "code", qualification_func_id)
                .await?
                .ok_or_else(|| {
                    BuiltinsError::BuiltinMissingFuncArgument(
                        qualification_func_name.to_string(),
                        "code".to_string(),
                    )
                })?;
        SchemaVariant::add_leaf(
            ctx,
            qualification_func_id,
            *schema_variant.id(),
            None,
            LeafKind::Qualification,
            vec![
                LeafInput {
                    location: LeafInputLocation::Domain,
                    func_argument_id: qualification_func_argument_id,
                },
                LeafInput {
                    location: LeafInputLocation::Code,
                    func_argument_id: *code_func_argument.id(),
                },
            ],
        )
        .await?;

        // Wrap it up.
        schema_variant.finalize(ctx, None).await?;

        // Set Defaults
        self.set_default_value_for_prop(
            ctx,
            *aws_resource_type_prop.id(),
            serde_json::json!["key-pair"],
        )
        .await?;
        self.set_default_value_for_prop(ctx, *key_type_prop.id(), serde_json::json!["rsa"])
            .await?;

        {
            // Connect the "/root/domain/key id" prop to the external provider.
            let external_provider_attribute_prototype_id = key_name_external_provider
                .attribute_prototype_id()
                .ok_or_else(|| {
                    BuiltinsError::MissingAttributePrototypeForExternalProvider(
                        *key_name_external_provider.id(),
                    )
                })?;
            let mut external_provider_attribute_prototype =
                AttributePrototype::get_by_id(ctx, external_provider_attribute_prototype_id)
                    .await?
                    .ok_or_else(|| {
                        AttributePrototypeError::NotFound(
                            *external_provider_attribute_prototype_id,
                            *ctx.visibility(),
                        )
                    })?;

            let key_name_metadata_func = Func::find_by_attr(
                ctx,
                "name",
                &"si:awsKeyPairMetadataExternalProvider".to_string(),
            )
            .await
            .expect("could not find si:awsKeyPairMetadataExternalProvider func by name attr")
            .pop()
            .expect("si:awsKeyPairMetadataExternalProvider func not found");
            external_provider_attribute_prototype
                .set_func_id(ctx, *key_name_metadata_func.id())
                .await?;

            let key_name_func_argument_name = "KeyName";
            let key_name_func_argument = FuncArgument::find_by_name_for_func(
                ctx,
                key_name_func_argument_name,
                *key_name_metadata_func.id(),
            )
            .await?
            .ok_or_else(|| {
                BuiltinsError::BuiltinMissingFuncArgument(
                    key_name_metadata_func.name().to_owned(),
                    key_name_func_argument_name.to_string(),
                )
            })?;

            let key_name_internal_provider =
                InternalProvider::find_for_prop(ctx, *key_name_prop.id())
                    .await?
                    .ok_or_else(|| {
                        BuiltinsError::ImplicitInternalProviderNotFoundForProp(*key_name_prop.id())
                    })?;
            AttributePrototypeArgument::new_for_intra_component(
                ctx,
                *external_provider_attribute_prototype_id,
                *key_name_func_argument.id(),
                *key_name_internal_provider.id(),
            )
            .await?;

            let applied_model_func_argument_name = "applied_model";
            let applied_model_func_argument = FuncArgument::find_by_name_for_func(
                ctx,
                applied_model_func_argument_name,
                *key_name_metadata_func.id(),
            )
            .await?
            .ok_or_else(|| {
                BuiltinsError::BuiltinMissingFuncArgument(
                    key_name_metadata_func.name().to_owned(),
                    applied_model_func_argument_name.to_string(),
                )
            })?;

            let applied_model_internal_provider =
                InternalProvider::find_for_prop(ctx, root_prop.applied_model_prop_id)
                    .await?
                    .ok_or_else(|| {
                        BuiltinsError::ImplicitInternalProviderNotFoundForProp(
                            root_prop.applied_model_prop_id,
                        )
                    })?;
            AttributePrototypeArgument::new_for_intra_component(
                ctx,
                *external_provider_attribute_prototype_id,
                *applied_model_func_argument.id(),
                *applied_model_internal_provider.id(),
            )
            .await?;

            let resource_func_argument_name = "resource";
            let resource_func_argument = FuncArgument::find_by_name_for_func(
                ctx,
                resource_func_argument_name,
                *key_name_metadata_func.id(),
            )
            .await?
            .ok_or_else(|| {
                BuiltinsError::BuiltinMissingFuncArgument(
                    key_name_metadata_func.name().to_owned(),
                    resource_func_argument_name.to_string(),
                )
            })?;

            let resource_internal_provider =
                InternalProvider::find_for_prop(ctx, root_prop.resource_prop_id)
                    .await?
                    .ok_or_else(|| {
                        BuiltinsError::ImplicitInternalProviderNotFoundForProp(
                            root_prop.resource_prop_id,
                        )
                    })?;
            AttributePrototypeArgument::new_for_intra_component(
                ctx,
                *external_provider_attribute_prototype_id,
                *resource_func_argument.id(),
                *resource_internal_provider.id(),
            )
            .await?;

            let domain_func_argument_name = "domain";
            let domain_func_argument = FuncArgument::find_by_name_for_func(
                ctx,
                domain_func_argument_name,
                *key_name_metadata_func.id(),
            )
            .await?
            .ok_or_else(|| {
                BuiltinsError::BuiltinMissingFuncArgument(
                    key_name_metadata_func.name().to_owned(),
                    domain_func_argument_name.to_string(),
                )
            })?;

            let domain_internal_provider =
                InternalProvider::find_for_prop(ctx, root_prop.domain_prop_id)
                    .await?
                    .ok_or_else(|| {
                        BuiltinsError::ImplicitInternalProviderNotFoundForProp(
                            root_prop.domain_prop_id,
                        )
                    })?;
            AttributePrototypeArgument::new_for_intra_component(
                ctx,
                *external_provider_attribute_prototype_id,
                *domain_func_argument.id(),
                *domain_internal_provider.id(),
            )
            .await?;
        }

        let tags_map_attribute_read_context =
            AttributeReadContext::default_with_prop(*tags_map_prop.id());
        let tags_map_attribute_value =
            AttributeValue::find_for_context(ctx, tags_map_attribute_read_context)
                .await?
                .ok_or(BuiltinsError::AttributeValueNotFoundForContext(
                    tags_map_attribute_read_context,
                ))?;
        let tags_map_item_attribute_context = AttributeContextBuilder::new()
            .set_prop_id(*tags_map_item_prop.id())
            .to_context()?;
        let name_tags_item_attribute_value_id = AttributeValue::insert_for_context(
            ctx,
            tags_map_item_attribute_context,
            *tags_map_attribute_value.id(),
            None,
            Some("Name".to_string()),
        )
        .await?;

        // Connect props to providers.

        let si_name_prop = self
            .find_child_prop_by_name(ctx, root_prop.si_prop_id, "name")
            .await?;
        let si_name_internal_provider = InternalProvider::find_for_prop(ctx, *si_name_prop.id())
            .await?
            .ok_or_else(|| {
                BuiltinsError::ImplicitInternalProviderNotFoundForProp(*si_name_prop.id())
            })?;
        let name_tags_item_attribute_value =
            AttributeValue::get_by_id(ctx, &name_tags_item_attribute_value_id)
                .await?
                .ok_or(BuiltinsError::AttributeValueNotFound(
                    name_tags_item_attribute_value_id,
                ))?;
        let mut name_tags_item_attribute_prototype = name_tags_item_attribute_value
            .attribute_prototype(ctx)
            .await?
            .ok_or(BuiltinsError::MissingAttributePrototypeForAttributeValue)?;
        name_tags_item_attribute_prototype
            .set_func_id(ctx, identity_func_item.func_id)
            .await?;
        let identity_arg =
            FuncArgument::find_by_name_for_func(ctx, "identity", identity_func_item.func_id)
                .await?
                .ok_or_else(|| {
                    BuiltinsError::BuiltinMissingFuncArgument(
                        "identity".to_string(),
                        "identity".to_string(),
                    )
                })?;
        AttributePrototypeArgument::new_for_intra_component(
            ctx,
            *name_tags_item_attribute_prototype.id(),
            *identity_arg.id(),
            *si_name_internal_provider.id(),
        )
        .await?;

        // Connect the "region" prop to the "Region" explicit internal provider.
        self.link_region_prop_to_explicit_internal_provider(
            ctx,
            region_prop.id(),
            identity_func_item.func_id,
            identity_func_item.func_argument_id,
            region_explicit_internal_provider.id(),
        )
        .await?;

        // Connect the "/root/si/name" field to the "/root/domain/KeyName" field.
        let key_name_attribute_value = AttributeValue::find_for_context(
            ctx,
            AttributeReadContext::default_with_prop(*key_name_prop.id()),
        )
        .await?
        .ok_or(AttributeValueError::Missing)?;
        let mut key_name_attribute_prototype = key_name_attribute_value
            .attribute_prototype(ctx)
            .await?
            .ok_or(AttributeValueError::MissingAttributePrototype)?;
        key_name_attribute_prototype
            .set_func_id(ctx, identity_func_item.func_id)
            .await?;
        let si_name_prop = self
            .find_child_prop_by_name(ctx, root_prop.si_prop_id, "name")
            .await?;
        let si_name_internal_provider = InternalProvider::find_for_prop(ctx, *si_name_prop.id())
            .await?
            .ok_or_else(|| {
                BuiltinsError::ImplicitInternalProviderNotFoundForProp(*si_name_prop.id())
            })?;
        AttributePrototypeArgument::new_for_intra_component(
            ctx,
            *key_name_attribute_prototype.id(),
            identity_func_item.func_argument_id,
            *si_name_internal_provider.id(),
        )
        .await?;

        let workflow_func_name = "si:awsKeyPairCreateWorkflow";
        let workflow_func = Func::find_by_attr(ctx, "name", &workflow_func_name)
            .await?
            .pop()
            .ok_or_else(|| SchemaError::FuncNotFound(workflow_func_name.to_owned()))?;
        let title = "Create Key Pair";
        let context = WorkflowPrototypeContext {
            schema_id: *schema.id(),
            schema_variant_id: *schema_variant.id(),
            ..Default::default()
        };
        let workflow_prototype = WorkflowPrototype::new(
            ctx,
            *workflow_func.id(),
            serde_json::Value::Null,
            context,
            title,
        )
        .await?;

        // Add confirmations.
        let confirmation_func_name = "si:confirmationResourceExists";
        let confirmation_func = Func::find_by_attr(ctx, "name", &confirmation_func_name)
            .await?
            .pop()
            .ok_or_else(|| SchemaError::FuncNotFound(confirmation_func_name.to_owned()))?;
        let confirmation_func_argument_name = "resource";
        let confirmation_func_argument = FuncArgument::find_by_name_for_func(
            ctx,
            confirmation_func_argument_name,
            *confirmation_func.id(),
        )
        .await?
        .ok_or_else(|| {
            BuiltinsError::BuiltinMissingFuncArgument(
                confirmation_func_name.to_string(),
                confirmation_func_argument_name.to_string(),
            )
        })?;
        SchemaVariant::add_leaf(
            ctx,
            *confirmation_func.id(),
            *schema_variant.id(),
            None,
            LeafKind::Confirmation,
            vec![LeafInput {
                location: LeafInputLocation::Resource,
                func_argument_id: *confirmation_func_argument.id(),
            }],
        )
        .await
        .expect("could not add leaf");
        FuncDescription::new(
            ctx,
            *confirmation_func.id(),
            *schema_variant.id(),
            FuncDescriptionContents::Confirmation {
                name: "Key Pair Exists?".to_string(),
                success_description: Some("Key Pair exists!".to_string()),
                failure_description: Some("This Key Pair has not been created yet. Please run the fix above to create it!".to_string()),
                provider: Some("AWS".to_string()),
            },
        )
            .await?;

        // Add delete confirmations.
        self.add_deletion_confirmation_and_workflow(
            ctx,
            name,
            &schema_variant,
            Some("AWS"),
            "si:awsKeyPairDeleteWorkflow",
        )
        .await?;

        // Add update confirmation
        self.add_update_confirmation(ctx, name, &schema_variant, Some("AWS"))
            .await?;

        let context = ActionPrototypeContext {
            schema_id: *schema.id(),
            schema_variant_id: *schema_variant.id(),
            ..Default::default()
        };
        ActionPrototype::new(
            ctx,
            *workflow_prototype.id(),
            "create",
            ActionKind::Create,
            context,
        )
        .await?;

        let keypair_refresh_workflow_func_name = "si:awsKeyPairRefreshWorkflow";
        let keypair_refresh_workflow_func =
            Func::find_by_attr(ctx, "name", &keypair_refresh_workflow_func_name)
                .await?
                .pop()
                .ok_or_else(|| {
                    SchemaError::FuncNotFound(keypair_refresh_workflow_func_name.to_owned())
                })?;
        let title = "Refresh Key Pair Resource";
        let context = WorkflowPrototypeContext {
            schema_id: *schema.id(),
            schema_variant_id: *schema_variant.id(),
            ..Default::default()
        };
        let workflow_prototype = WorkflowPrototype::new(
            ctx,
            *keypair_refresh_workflow_func.id(),
            serde_json::Value::Null,
            context,
            title,
        )
        .await?;

        let context = ActionPrototypeContext {
            schema_id: *schema.id(),
            schema_variant_id: *schema_variant.id(),
            ..Default::default()
        };
        ActionPrototype::new(
            ctx,
            *workflow_prototype.id(),
            "refresh",
            ActionKind::Refresh,
            context,
        )
        .await?;

        Ok(())
    }
}
