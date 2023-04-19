use axum::Json;
use dal::action_prototype::ActionKind;
use dal::component::ComponentKind;
use dal::func::argument::FuncArgumentKind;

use dal::Visibility;
use dal::{
    ActionPrototype, ActionPrototypeContext, DalContext, ExternalProvider, Func, FuncArgument,
    FuncBackendKind, FuncBackendResponseType, FuncBinding, InternalProvider, LeafInput,
    LeafInputLocation, LeafKind, Schema, SchemaId, SchemaVariant, SchemaVariantId, SocketArity,
    StandardModel, WorkflowPrototype, WorkflowPrototypeContext,
};
use serde::{Deserialize, Serialize};

use super::DevResult;
use crate::server::extract::{AccessBuilder, HandlerContext};

/// The name of the ["confirmation"](dal::FuncBackendResponseType::Confirmation) that could return
/// a "recommendation" with [`ActionKind::Create`](ActionKind::Create).
pub const CREATE_CONFIRMATION_NAME: &str = "dev:createConfirmation";
/// The name of the ["confirmation"](dal::FuncBackendResponseType::Confirmation) that could return
/// a "recommendation" with [`ActionKind::Destroy`](ActionKind::Destroy).
pub const DELETE_CONFIRMATION_NAME: &str = "dev:deleteConfirmation";

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AuthorSingleSchemaRequest {
    pub schema_name: String,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AuthorSingleSchemaResponse {
    pub schema_id: SchemaId,
    pub schema_variant_id: SchemaVariantId,
}

pub async fn author_single_schema_with_default_variant(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<AuthorSingleSchemaRequest>,
) -> DevResult<Json<AuthorSingleSchemaResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let (schema, schema_variant) =
        AuthoringHelper::author_single_schema_with_default_variant(&ctx, request.schema_name).await;

    ctx.commit().await?;

    Ok(Json(AuthorSingleSchemaResponse {
        schema_id: *schema.id(),
        schema_variant_id: *schema_variant.id(),
    }))
}

/// This unit struct provides bootstrapping methods for authoring the test within this module.
struct AuthoringHelper;

impl AuthoringHelper {
    /// Create a [`Schema`](dal::Schema) with a default [`SchemaVariant`](dal::SchemaVariant)
    /// that does not rely on "builtins".
    pub async fn author_single_schema_with_default_variant(
        ctx: &DalContext,
        schema_name: impl AsRef<str>,
    ) -> (Schema, SchemaVariant) {
        let mut schema = Schema::new(ctx, schema_name.as_ref(), &ComponentKind::Standard)
            .await
            .expect("cannot create schema");
        let (mut schema_variant, _) = SchemaVariant::new(ctx, *schema.id(), "v0")
            .await
            .expect("cannot create schema variant");

        let identity_func: Func = Func::find_by_attr(ctx, "name", &"si:identity".to_string())
            .await
            .expect("could not find identity func by name attr")
            .pop()
            .expect("identity func not found");
        let (identity_func_binding, identity_func_binding_return_value) =
            FuncBinding::create_and_execute(
                ctx,
                serde_json::json![{ "identity": null }],
                *identity_func.id(),
            )
            .await
            .expect("could not find or create identity func binding");
        let (identity_func_id, identity_func_binding_id, identity_func_binding_return_value_id) = (
            *identity_func.id(),
            *identity_func_binding.id(),
            *identity_func_binding_return_value.id(),
        );

        // NOTE(nick): it is possible to create cycles with these sockets. Please do not do that.
        let (_explicit_internal_provider, _input_socket) =
            InternalProvider::new_explicit_with_socket(
                ctx,
                *schema_variant.id(),
                "universal",
                identity_func_id,
                identity_func_binding_id,
                identity_func_binding_return_value_id,
                SocketArity::Many,
                false,
            )
            .await
            .expect("could not create explicit internal provider with socket");
        let (_external_provider, _output_socket) = ExternalProvider::new_with_socket(
            ctx,
            *schema.id(),
            *schema_variant.id(),
            "universal",
            None,
            identity_func_id,
            identity_func_binding_id,
            identity_func_binding_return_value_id,
            SocketArity::Many,
            false,
        )
        .await
        .expect("could not create external provider with socket");

        schema
            .set_default_schema_variant_id(ctx, Some(*schema_variant.id()))
            .await
            .expect("cannot set default schema variant");

        // Add the create and destroy actions and confirmations.
        Self::add_create_action_and_confirmation(ctx, *schema.id(), *schema_variant.id()).await;
        Self::add_destroy_action_and_confirmation(ctx, *schema.id(), *schema_variant.id()).await;
        Self::add_refresh_workflow(ctx, *schema.id(), *schema_variant.id()).await;

        // Finalize the schema variant and create the component.
        schema_variant
            .finalize(ctx, None)
            .await
            .expect("unable to finalize schema variant");

        (schema, schema_variant)
    }

    /// Create a [`Create`](dal::ActionKind::Create) [`action`](dal::ActionPrototype) and
    /// corresponding "confirmation" [`leaf`](dal::schema::variant::leaves).
    async fn add_create_action_and_confirmation(
        ctx: &DalContext,
        schema_id: SchemaId,
        schema_variant_id: SchemaVariantId,
    ) {
        let mut command_func = Func::new(
            ctx,
            "dev:createCommand",
            FuncBackendKind::JsCommand,
            FuncBackendResponseType::Command,
        )
        .await
        .expect("could not create func");
        let code = "async function create() {
          return { value: \"poop\", status: \"ok\" };
        }";
        command_func
            .set_code_plaintext(ctx, Some(code))
            .await
            .expect("set code");
        command_func
            .set_handler(ctx, Some("create"))
            .await
            .expect("set handler");
        let mut workflow_func = Func::new(
            ctx,
            "dev:createWorkflow",
            FuncBackendKind::JsWorkflow,
            FuncBackendResponseType::Workflow,
        )
        .await
        .expect("could not create func");
        let code = "async function create() {
          return {
            name: \"dev:createWorkflow\",
            kind: \"conditional\",
            steps: [
              {
                command: \"dev:createCommand\",
              },
            ],
          };
        }";
        workflow_func
            .set_code_plaintext(ctx, Some(code))
            .await
            .expect("set code");
        workflow_func
            .set_handler(ctx, Some("create"))
            .await
            .expect("set handler");

        // Create workflow and action prototypes.
        let workflow_prototype = WorkflowPrototype::new(
            ctx,
            *workflow_func.id(),
            serde_json::Value::Null,
            WorkflowPrototypeContext {
                schema_id,
                schema_variant_id,
                ..Default::default()
            },
            "create",
        )
        .await
        .expect("could not create workflow prototype");
        ActionPrototype::new(
            ctx,
            *workflow_prototype.id(),
            "create",
            ActionKind::Create,
            ActionPrototypeContext {
                schema_id,
                schema_variant_id,
                ..Default::default()
            },
        )
        .await
        .expect("unable to create action prototype");

        // Setup the confirmation function.
        let mut confirmation_func = Func::new(
            ctx,
            CREATE_CONFIRMATION_NAME,
            FuncBackendKind::JsAttribute,
            FuncBackendResponseType::Confirmation,
        )
        .await
        .expect("could not create func");
        let confirmation_func_id = *confirmation_func.id();
        let code = "async function exists(input) {
            if (!input.resource?.value) {
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
            .await
            .expect("set code");
        confirmation_func
            .set_handler(ctx, Some("exists"))
            .await
            .expect("set handler");
        let confirmation_func_argument = FuncArgument::new(
            ctx,
            "resource",
            FuncArgumentKind::String,
            None,
            confirmation_func_id,
        )
        .await
        .expect("could not create func argument");

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
        .await
        .expect("could not add leaf");
    }

    /// Create a [`Destroy`](dal::ActionKind::Destroy) [`action`](dal::ActionPrototype) and
    /// corresponding "confirmation" [`leaf`](dal::schema::variant::leaves).
    async fn add_destroy_action_and_confirmation(
        ctx: &DalContext,
        schema_id: SchemaId,
        schema_variant_id: SchemaVariantId,
    ) {
        let mut command_func = Func::new(
            ctx,
            "dev:destroyCommand",
            FuncBackendKind::JsCommand,
            FuncBackendResponseType::Command,
        )
        .await
        .expect("could not create func");
        let code = "async function destroy() {
          return { value: null, status: \"ok\" };
        }";
        command_func
            .set_code_plaintext(ctx, Some(code))
            .await
            .expect("set code");
        command_func
            .set_handler(ctx, Some("destroy"))
            .await
            .expect("set handler");
        let mut workflow_func = Func::new(
            ctx,
            "dev:destroyWorkflow",
            FuncBackendKind::JsWorkflow,
            FuncBackendResponseType::Workflow,
        )
        .await
        .expect("could not create func");
        let code = "async function destroy() {
          return {
            name: \"dev:destroyWorkflow\",
            kind: \"conditional\",
            steps: [
              {
                command: \"dev:destroyCommand\",
              },
            ],
          };
        }";
        workflow_func
            .set_code_plaintext(ctx, Some(code))
            .await
            .expect("set code");
        workflow_func
            .set_handler(ctx, Some("destroy"))
            .await
            .expect("set handler");

        // Create workflow and action prototypes.
        let workflow_prototype = WorkflowPrototype::new(
            ctx,
            *workflow_func.id(),
            serde_json::Value::Null,
            WorkflowPrototypeContext {
                schema_id,
                schema_variant_id,
                ..Default::default()
            },
            "destroy",
        )
        .await
        .expect("could not create workflow prototype");
        ActionPrototype::new(
            ctx,
            *workflow_prototype.id(),
            "destroy",
            ActionKind::Destroy,
            ActionPrototypeContext {
                schema_id,
                schema_variant_id,
                ..Default::default()
            },
        )
        .await
        .expect("unable to create action prototype");

        // Setup the confirmation function.
        let mut confirmation_func = Func::new(
            ctx,
            DELETE_CONFIRMATION_NAME,
            FuncBackendKind::JsAttribute,
            FuncBackendResponseType::Confirmation,
        )
        .await
        .expect("could not create func");
        let confirmation_func_id = *confirmation_func.id();
        let code = "async function exists(input) {
            if (input.resource?.value && input.deleted_at) {
                return {
                    success: false,
                    recommendedActions: [\"destroy\"]
                }
            }
            return {
                success: true,
                recommendedActions: [],
            }
        }";
        confirmation_func
            .set_code_plaintext(ctx, Some(code))
            .await
            .expect("set code");
        confirmation_func
            .set_handler(ctx, Some("exists"))
            .await
            .expect("set handler");

        // Create the func arguments.
        let deleted_at_confirmation_func_argument = FuncArgument::new(
            ctx,
            "deleted_at",
            FuncArgumentKind::String,
            None,
            confirmation_func_id,
        )
        .await
        .expect("could not create func argument");
        let resource_confirmation_func_argument = FuncArgument::new(
            ctx,
            "resource",
            FuncArgumentKind::String,
            None,
            confirmation_func_id,
        )
        .await
        .expect("could not create func argument");

        // Add the leaf for the confirmation.
        SchemaVariant::add_leaf(
            ctx,
            confirmation_func_id,
            schema_variant_id,
            None,
            LeafKind::Confirmation,
            vec![
                LeafInput {
                    location: LeafInputLocation::DeletedAt,
                    func_argument_id: *deleted_at_confirmation_func_argument.id(),
                },
                LeafInput {
                    location: LeafInputLocation::Resource,
                    func_argument_id: *resource_confirmation_func_argument.id(),
                },
            ],
        )
        .await
        .expect("could not add leaf");
    }

    /// Create a "refresh" [workflow](dal::WorkflowPrototype) and [`action`](dal::ActionPrototype).
    async fn add_refresh_workflow(
        ctx: &DalContext,
        schema_id: SchemaId,
        schema_variant_id: SchemaVariantId,
    ) {
        let mut command_func = Func::new(
            ctx,
            "dev:refreshCommand",
            FuncBackendKind::JsCommand,
            FuncBackendResponseType::Command,
        )
        .await
        .expect("could not create func");
        let code = "async function refresh() {
          return { status: \"ok\" };
        }";
        command_func
            .set_code_plaintext(ctx, Some(code))
            .await
            .expect("set code");
        command_func
            .set_handler(ctx, Some("refresh"))
            .await
            .expect("set handler");

        let mut workflow_func = Func::new(
            ctx,
            "dev:refreshWorkflow",
            FuncBackendKind::JsWorkflow,
            FuncBackendResponseType::Workflow,
        )
        .await
        .expect("could not create func");
        let code = "async function refresh() {
          return {
            name: \"dev:refreshWorkflow\",
            kind: \"conditional\",
            steps: [
              {
                command: \"dev:refreshCommand\",
              },
            ],
          };
        }";
        workflow_func
            .set_code_plaintext(ctx, Some(code))
            .await
            .expect("set code");
        workflow_func
            .set_handler(ctx, Some("refresh"))
            .await
            .expect("set handler");

        // Create the workflow and action prototype.
        let workflow_prototype = WorkflowPrototype::new(
            ctx,
            *workflow_func.id(),
            serde_json::Value::Null,
            WorkflowPrototypeContext {
                schema_id,
                schema_variant_id,
                ..Default::default()
            },
            "refresh",
        )
        .await
        .expect("could not create workflow prototype");
        ActionPrototype::new(
            ctx,
            *workflow_prototype.id(),
            "refresh",
            ActionKind::Refresh,
            ActionPrototypeContext {
                schema_id,
                schema_variant_id,
                ..Default::default()
            },
        )
        .await
        .expect("unable to create action prototype");
    }
}
