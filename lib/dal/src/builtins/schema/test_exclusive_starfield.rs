use crate::func::argument::FuncArgumentKind;
use crate::schema::variant::definition::{
    SchemaVariantDefinition, SchemaVariantDefinitionJson, SchemaVariantDefinitionMetadataJson,
};
use crate::schema::variant::leaves::LeafInputLocation;
use crate::schema::variant::leaves::LeafKind;
use crate::{
    builtins::schema::MigrationDriver, schema::variant::leaves::LeafInput, ActionKind,
    ActionPrototype, ActionPrototypeContext, AttributePrototypeArgument, AttributeReadContext,
    AttributeValue, AttributeValueError, BuiltinsError, Func, FuncArgument, FuncBackendKind,
    FuncBackendResponseType, InternalProvider, WorkflowPrototype, WorkflowPrototypeContext,
};
use crate::{BuiltinsResult, DalContext, SchemaVariant, StandardModel};

const DEFINITION: &str = include_str!("definitions/test_exclusive_starfield.json");
const DEFINITION_METADATA: &str =
    include_str!("definitions/test_exclusive_starfield.metadata.json");

impl MigrationDriver {
    pub async fn migrate_test_exclusive_starfield(&self, ctx: &DalContext) -> BuiltinsResult<()> {
        let definition: SchemaVariantDefinitionJson = serde_json::from_str(DEFINITION)?;
        let metadata: SchemaVariantDefinitionMetadataJson =
            serde_json::from_str(DEFINITION_METADATA)?;

        SchemaVariantDefinition::new_from_structs(ctx, metadata.clone(), definition.clone())
            .await?;

        let (
            mut schema,
            mut schema_variant,
            _root_prop,
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
            "test:confirmationStarfield",
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

        // Add create command, workflow and action.
        {
            let mut command_func = Func::new(
                ctx,
                "test:createCommandStarfield",
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
                "test:createWorkflowStarfield",
                FuncBackendKind::JsWorkflow,
                FuncBackendResponseType::Workflow,
            )
            .await?;
            let code = "async function create() {
              return {
                name: \"test:createWorkflowStarfield\",
                kind: \"conditional\",
                steps: [
                  {
                    command: \"test:createCommandStarfield\",
                  },
                ],
              };
            }";
            workflow_func.set_code_plaintext(ctx, Some(code)).await?;
            workflow_func.set_handler(ctx, Some("create")).await?;
            let workflow_prototype = WorkflowPrototype::new(
                ctx,
                *workflow_func.id(),
                serde_json::Value::Null,
                WorkflowPrototypeContext {
                    schema_id: *schema.id(),
                    schema_variant_id: *schema_variant.id(),
                    ..Default::default()
                },
                "Create Starfield",
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
        }

        // Add refresh command, workflow and action.
        {
            let mut refresh_func = Func::new(
                ctx,
                "test:refreshCommandStarfield",
                FuncBackendKind::JsCommand,
                FuncBackendResponseType::Command,
            )
            .await?;
            let code = "async function refresh(component: Input): Promise<Output> {
              return { payload: \"poop\", status: \"ok\" };
            }";
            refresh_func.set_code_plaintext(ctx, Some(code)).await?;
            refresh_func.set_handler(ctx, Some("refresh")).await?;
            let mut workflow_func = Func::new(
                ctx,
                "test:refreshWorkflowStarfield",
                FuncBackendKind::JsWorkflow,
                FuncBackendResponseType::Workflow,
            )
            .await?;
            let code = "async function refresh(arg: Input): Promise<Output> {
              return {
                name: \"test:refreshWorkflowStarfield\",
                kind: \"conditional\",
                steps: [
                  {
                    command: \"test:refreshCommandStarfield\",
                    args: [arg],
                  },
                ],
              };
            }";
            workflow_func.set_code_plaintext(ctx, Some(code)).await?;
            workflow_func.set_handler(ctx, Some("refresh")).await?;
            let workflow_prototype = WorkflowPrototype::new(
                ctx,
                *workflow_func.id(),
                serde_json::Value::Null,
                WorkflowPrototypeContext {
                    schema_id: *schema.id(),
                    schema_variant_id: *schema_variant.id(),
                    ..Default::default()
                },
                "Refresh Starfield",
            )
            .await?;
            ActionPrototype::new(
                ctx,
                *workflow_prototype.id(),
                "refresh",
                ActionKind::Create,
                ActionPrototypeContext {
                    schema_id: *schema.id(),
                    schema_variant_id,
                    ..Default::default()
                },
            )
            .await?;
        }

        // Create the transformation func for one of the input sockets.
        let mut transformation_func = Func::new(
            ctx,
            "test:falloutEntriesToGalaxies",
            FuncBackendKind::Json,
            FuncBackendResponseType::Array,
        )
        .await?;
        let code =  "async function falloutEntriesToGalaxies(input: Input): Promise<Output> {
          let galaxies = [];
          let entries = input.entries;

          // Force the entries arg to be an Array (and return an empty array if the arg is absent/undefined/null).
          if (entries === undefined) return galaxies;
          if (entries === null) return galaxies;
          if (!Array.isArray(entries)) entries = [entries];

          entries.filter(i => i ?? false).forEach(function (entry) {

            let name = entry.si.name;
            let rads = entry.domain.rads;
            let galaxy = {
              sun: name + \"-sun\",
              planets: rads
            };

            galaxies.push(galaxy);
          });

          return galaxies;
        }";
        transformation_func
            .set_code_plaintext(ctx, Some(code))
            .await?;
        transformation_func
            .set_handler(ctx, Some("falloutEntriesToGalaxies"))
            .await?;
        let transformation_func_argument = FuncArgument::new(
            ctx,
            "entries",
            FuncArgumentKind::Array,
            Some(FuncArgumentKind::Object),
            *transformation_func.id(),
        )
        .await?;

        // Finalize the schema variant.
        schema_variant.finalize(ctx, None).await?;

        // Get the identity func and cache props.
        let identity_func_item = self
            .get_func_item("si:identity")
            .ok_or(BuiltinsError::FuncNotFoundInMigrationCache("si:identity"))?;
        let domain_name_prop = schema_variant
            .find_prop(ctx, &["root", "domain", "name"])
            .await?;
        let si_name_prop = schema_variant
            .find_prop(ctx, &["root", "si", "name"])
            .await?;
        let attributes_prop = schema_variant
            .find_prop(ctx, &["root", "domain", "attributes"])
            .await?;
        let galaxies_prop = schema_variant
            .find_prop(ctx, &["root", "domain", "universe", "galaxies"])
            .await?;

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

        // Connect the "bethesda" explicit internal provider to the "/root/domain/attributes" prop.
        {
            let explicit_internal_provider_name = "bethesda".to_string();
            let explicit_internal_provider =
                InternalProvider::find_explicit_for_schema_variant_and_name(
                    ctx,
                    schema_variant_id,
                    &explicit_internal_provider_name,
                )
                .await?
                .ok_or(BuiltinsError::ExplicitInternalProviderNotFound(
                    explicit_internal_provider_name,
                ))?;
            let attribute_read_context =
                AttributeReadContext::default_with_prop(*attributes_prop.id());
            let attribute_value = AttributeValue::find_for_context(ctx, attribute_read_context)
                .await?
                .ok_or(BuiltinsError::AttributeValueNotFoundForContext(
                    attribute_read_context,
                ))?;
            let mut attribute_prototype = attribute_value
                .attribute_prototype(ctx)
                .await?
                .ok_or(BuiltinsError::MissingAttributePrototypeForAttributeValue)?;
            attribute_prototype
                .set_func_id(ctx, &identity_func_item.func_id)
                .await?;
            AttributePrototypeArgument::new_for_intra_component(
                ctx,
                *attribute_prototype.id(),
                identity_func_item.func_argument_id,
                *explicit_internal_provider.id(),
            )
            .await?;
        }

        // Enable connections from the "fallout" explicit internal provider to the
        // "/root/domain/universe/galaxies/" field. We need to use the appropriate function with and
        // name the argument "galaxies".
        {
            let explicit_internal_provider_name = "fallout".to_string();
            let explicit_internal_provider =
                InternalProvider::find_explicit_for_schema_variant_and_name(
                    ctx,
                    schema_variant_id,
                    &explicit_internal_provider_name,
                )
                .await?
                .ok_or(BuiltinsError::ExplicitInternalProviderNotFound(
                    explicit_internal_provider_name,
                ))?;
            let galaxies_attribute_read_context =
                AttributeReadContext::default_with_prop(*galaxies_prop.id());
            let galaxies_attribute_value =
                AttributeValue::find_for_context(ctx, galaxies_attribute_read_context)
                    .await?
                    .ok_or(BuiltinsError::AttributeValueNotFoundForContext(
                        galaxies_attribute_read_context,
                    ))?;
            let mut galaxies_attribute_prototype = galaxies_attribute_value
                .attribute_prototype(ctx)
                .await?
                .ok_or(BuiltinsError::MissingAttributePrototypeForAttributeValue)?;
            galaxies_attribute_prototype
                .set_func_id(ctx, *transformation_func.id())
                .await?;
            AttributePrototypeArgument::new_for_intra_component(
                ctx,
                *galaxies_attribute_prototype.id(),
                *transformation_func_argument.id(),
                *explicit_internal_provider.id(),
            )
            .await?;
        }

        Ok(())
    }
}
