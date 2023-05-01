use crate::func::argument::FuncArgumentKind;
use crate::schema::variant::definition::{
    SchemaVariantDefinition, SchemaVariantDefinitionJson, SchemaVariantDefinitionMetadataJson,
};
use crate::schema::variant::leaves::LeafInputLocation;
use crate::schema::variant::leaves::LeafKind;
use crate::validation::{Validation, ValidationKind};
use crate::{
    builtins::schema::MigrationDriver, schema::variant::leaves::LeafInput, ActionKind,
    ActionPrototype, ActionPrototypeContext, AttributePrototypeArgument, AttributeReadContext,
    AttributeValue, AttributeValueError, BuiltinsError, ExternalProvider, Func, FuncArgument,
    FuncBackendKind, FuncBackendResponseType, InternalProvider, WorkflowPrototype,
    WorkflowPrototypeContext,
};
use crate::{BuiltinsResult, DalContext, SchemaVariant, StandardModel};

const DEFINITION: &str = include_str!("definitions/test_exclusive_fallout.json");
const DEFINITION_METADATA: &str = include_str!("definitions/test_exclusive_fallout.metadata.json");

impl MigrationDriver {
    pub async fn migrate_test_exclusive_fallout(&self, ctx: &DalContext) -> BuiltinsResult<()> {
        let definition: SchemaVariantDefinitionJson = serde_json::from_str(DEFINITION)?;
        let metadata: SchemaVariantDefinitionMetadataJson =
            serde_json::from_str(DEFINITION_METADATA)?;

        SchemaVariantDefinition::new_from_structs(ctx, metadata.clone(), definition.clone())
            .await?;

        let (
            mut schema,
            mut schema_variant,
            root_prop,
            _maybe_prop_cache,
            _explicit_internal_providers,
            _external_providers,
        ) = match self
            .create_schema_and_variant(ctx, metadata, Some(definition))
            .await?
        {
            Some(tuple) => tuple,
            None => return Ok(()),
        };
        schema.set_ui_hidden(ctx, true).await?;
        let schema_variant_id = *schema_variant.id();

        // Setup the confirmation function.
        let mut confirmation_func = Func::new(
            ctx,
            "test:confirmationFallout",
            FuncBackendKind::JsAttribute,
            FuncBackendResponseType::Confirmation,
        )
        .await?;
        let confirmation_func_id = *confirmation_func.id();
        let code = "async function exists(input) {
            if (!input.resource?.payload) {
                return {
                    success: false,
                    recommendedActions: [\"create\"]
                }
            }
            return {
                success: true,
                recommendedActions: [],
            }
        }";
        confirmation_func
            .set_code_plaintext(ctx, Some(code))
            .await?;
        confirmation_func.set_handler(ctx, Some("exists")).await?;
        let confirmation_func_argument = FuncArgument::new(
            ctx,
            "resource",
            FuncArgumentKind::String,
            None,
            confirmation_func_id,
        )
        .await?;

        // Add the leaf for the confirmation.
        SchemaVariant::add_leaf(
            ctx,
            confirmation_func_id,
            schema_variant_id,
            None,
            LeafKind::Confirmation,
            vec![LeafInput {
                location: LeafInputLocation::Resource,
                func_argument_id: *confirmation_func_argument.id(),
            }],
        )
        .await?;

        // Create command and workflow funcs for our workflow and action prototypes.
        let mut command_func = Func::new(
            ctx,
            "test:createCommandFallout",
            FuncBackendKind::JsCommand,
            FuncBackendResponseType::Command,
        )
        .await?;
        let code = "async function create() {
            return { payload: \"poop\", status: \"ok\" };
        }";
        command_func.set_code_plaintext(ctx, Some(code)).await?;
        command_func.set_handler(ctx, Some("create")).await?;
        let mut workflow_func = Func::new(
            ctx,
            "test:createWorkflowFallout",
            FuncBackendKind::JsWorkflow,
            FuncBackendResponseType::Workflow,
        )
        .await?;
        let code = "async function create() {
          return {
            name: \"test:createWorkflowFallout\",
            kind: \"conditional\",
            steps: [
              {
                command: \"test:createCommandFallout\",
              },
            ],
          };
        }";
        workflow_func.set_code_plaintext(ctx, Some(code)).await?;
        workflow_func.set_handler(ctx, Some("create")).await?;

        // Create workflow and action prototypes.
        let workflow_prototype = WorkflowPrototype::new(
            ctx,
            *workflow_func.id(),
            serde_json::Value::Null,
            WorkflowPrototypeContext {
                schema_id: *schema.id(),
                schema_variant_id: *schema_variant.id(),
                ..Default::default()
            },
            "Create Fallout",
        )
        .await?;
        ActionPrototype::new(
            ctx,
            *workflow_prototype.id(),
            "create",
            ActionKind::Create,
            ActionPrototypeContext {
                schema_id: *schema.id(),
                schema_variant_id,
                ..Default::default()
            },
        )
        .await?;

        // Get the identity func and cache props.
        let identity_func_item = self
            .get_func_item("si:identity")
            .ok_or(BuiltinsError::FuncNotFoundInMigrationCache("si:identity"))?;
        let special_prop = schema_variant
            .find_prop(ctx, &["root", "domain", "special"])
            .await?;
        let rads_prop = schema_variant
            .find_prop(ctx, &["root", "domain", "rads"])
            .await?;
        let si_name_prop = schema_variant
            .find_prop(ctx, &["root", "si", "name"])
            .await?;
        let domain_name_prop = schema_variant
            .find_prop(ctx, &["root", "domain", "name"])
            .await?;
        let active_prop = schema_variant
            .find_prop(ctx, &["root", "domain", "active"])
            .await?;

        // Create validation(s).
        self.create_validation(
            ctx,
            ValidationKind::Builtin(Validation::IntegerIsNotEmpty { value: None }),
            *rads_prop.id(),
            *schema.id(),
            schema_variant_id,
        )
        .await?;
        self.create_validation(
            ctx,
            ValidationKind::Builtin(Validation::IntegerIsBetweenTwoIntegers {
                value: None,
                lower_bound: -1,
                upper_bound: 1001,
            }),
            *rads_prop.id(),
            *schema.id(),
            schema_variant_id,
        )
        .await?;

        // Finalize the schema variant.
        schema_variant.finalize(ctx, None).await?;

        // Set default values for props.
        self.set_default_value_for_prop(ctx, *active_prop.id(), serde_json::json![true])
            .await?;

        // Connect the "/root/domain/special" prop to the "bethesda" external provider.
        {
            let implicit_internal_provider =
                InternalProvider::find_for_prop(ctx, *special_prop.id())
                    .await?
                    .ok_or(BuiltinsError::ImplicitInternalProviderNotFoundForProp(
                        *special_prop.id(),
                    ))?;
            let external_provider_name = "bethesda".to_string();
            let external_provider = ExternalProvider::find_for_schema_variant_and_name(
                ctx,
                schema_variant_id,
                &external_provider_name,
            )
            .await?
            .ok_or(BuiltinsError::ExternalProviderNotFound(
                external_provider_name,
            ))?;
            let external_provider_attribute_prototype_id =
                external_provider.attribute_prototype_id().ok_or_else(|| {
                    BuiltinsError::MissingAttributePrototypeForExternalProvider(
                        *external_provider.id(),
                    )
                })?;
            AttributePrototypeArgument::new_for_intra_component(
                ctx,
                *external_provider_attribute_prototype_id,
                identity_func_item.func_argument_id,
                *implicit_internal_provider.id(),
            )
            .await?;
        }

        // Connect the "/root/si/name" field to the "/root/domain/name" field.
        {
            let domain_name_attribute_value = AttributeValue::find_for_context(
                ctx,
                AttributeReadContext::default_with_prop(*domain_name_prop.id()),
            )
            .await?
            .ok_or(AttributeValueError::Missing)?;
            let mut domain_name_attribute_prototype = domain_name_attribute_value
                .attribute_prototype(ctx)
                .await?
                .ok_or(AttributeValueError::MissingAttributePrototype)?;
            domain_name_attribute_prototype
                .set_func_id(ctx, identity_func_item.func_id)
                .await?;
            let si_name_internal_provider =
                InternalProvider::find_for_prop(ctx, *si_name_prop.id())
                    .await?
                    .ok_or_else(|| {
                        BuiltinsError::ImplicitInternalProviderNotFoundForProp(*si_name_prop.id())
                    })?;
            AttributePrototypeArgument::new_for_intra_component(
                ctx,
                *domain_name_attribute_prototype.id(),
                identity_func_item.func_argument_id,
                *si_name_internal_provider.id(),
            )
            .await?;
        }

        // Connect "/root" to the appropriate external provider.
        {
            let root_implicit_internal_provider =
                InternalProvider::find_for_prop(ctx, root_prop.prop_id)
                    .await?
                    .ok_or(BuiltinsError::ImplicitInternalProviderNotFoundForProp(
                        root_prop.prop_id,
                    ))?;
            let external_provider_name = "fallout".to_string();
            let external_provider = ExternalProvider::find_for_schema_variant_and_name(
                ctx,
                schema_variant_id,
                &external_provider_name,
            )
            .await?
            .ok_or(BuiltinsError::ExternalProviderNotFound(
                external_provider_name,
            ))?;
            let external_provider_attribute_prototype_id =
                external_provider.attribute_prototype_id().ok_or_else(|| {
                    BuiltinsError::MissingAttributePrototypeForExternalProvider(
                        *external_provider.id(),
                    )
                })?;
            AttributePrototypeArgument::new_for_intra_component(
                ctx,
                *external_provider_attribute_prototype_id,
                identity_func_item.func_argument_id,
                *root_implicit_internal_provider.id(),
            )
            .await?;
        }

        Ok(())
    }
}
