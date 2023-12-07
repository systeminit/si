use axum::Json;
use dal::action_prototype::ActionKind;
use dal::component::ComponentKind;

use dal::Visibility;
use dal::{
    ActionPrototype, ActionPrototypeContext, DalContext, ExternalProvider, Func, FuncBackendKind,
    FuncBackendResponseType, FuncBinding, InternalProvider, Schema, SchemaId, SchemaVariant,
    SchemaVariantId, SocketArity, StandardModel,
};
use serde::{Deserialize, Serialize};

use super::DevResult;
use crate::server::extract::{AccessBuilder, HandlerContext};

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
                vec![],
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
                "universal",
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
            "universal",
            SocketArity::Many,
            false,
        )
        .await
        .expect("could not create external provider with socket");

        schema
            .set_default_schema_variant_id(ctx, Some(*schema_variant.id()))
            .await
            .expect("cannot set default schema variant");

        Self::add_refresh_action(ctx, *schema_variant.id()).await;

        // Finalize the schema variant and create the component.
        schema_variant
            .finalize(ctx, None)
            .await
            .expect("unable to finalize schema variant");

        (schema, schema_variant)
    }

    /// Create a "refresh" [`action`](dal::ActionPrototype).
    async fn add_refresh_action(ctx: &DalContext, schema_variant_id: SchemaVariantId) {
        let mut action_func = Func::new(
            ctx,
            "dev:refreshAction",
            FuncBackendKind::JsAction,
            FuncBackendResponseType::Action,
        )
        .await
        .expect("could not create func");
        let code = "async function refresh() {
          return { status: \"ok\" };
        }";
        action_func
            .set_code_plaintext(ctx, Some(code))
            .await
            .expect("set code");
        action_func
            .set_handler(ctx, Some("refresh"))
            .await
            .expect("set handler");

        ActionPrototype::new(
            ctx,
            *action_func.id(),
            ActionKind::Refresh,
            ActionPrototypeContext { schema_variant_id },
        )
        .await
        .expect("unable to create action prototype");
    }
}
