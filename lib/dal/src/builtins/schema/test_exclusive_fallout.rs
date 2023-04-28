use crate::func::argument::FuncArgumentKind;
use crate::schema::variant::definition::{
    SchemaVariantDefinition, SchemaVariantDefinitionJson, SchemaVariantDefinitionMetadataJson,
};
use crate::schema::variant::leaves::LeafInputLocation;
use crate::schema::variant::leaves::LeafKind;
use crate::{
    builtins::schema::MigrationDriver, schema::variant::leaves::LeafInput, ActionKind,
    ActionPrototype, ActionPrototypeContext, Func, FuncArgument, FuncBackendKind,
    FuncBackendResponseType, WorkflowPrototype, WorkflowPrototypeContext,
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

        // Finalize the schema variant.
        schema_variant.finalize(ctx, None).await?;

        Ok(())
    }
}
