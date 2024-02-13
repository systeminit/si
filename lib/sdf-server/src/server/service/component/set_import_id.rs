use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use crate::server::tracking::track;
use crate::service::component::{ComponentError, ComponentResult};
use axum::extract::OriginalUri;
use axum::response::IntoResponse;
use axum::Json;
use dal::prop::PropPath;
use dal::{
    AttributeContext, AttributeReadContext, AttributeValue, AttributeValueError, ChangeSet,
    Component, ComponentId, Prop, StandardModel, Visibility,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SetImportIdRequest {
    pub component_id: ComponentId,
    pub value: Option<Value>,
    #[serde(flatten)]
    pub visibility: Visibility,
}

pub async fn set_import_id(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Json(request): Json<SetImportIdRequest>,
) -> ComponentResult<impl IntoResponse> {
    dbg!("We got a request here!!");
    let mut ctx = builder.build(request_ctx.build(request.visibility)).await?;

    // let force_changeset_pk = ChangeSet::force_new(&mut ctx).await?;

    let component = Component::get_by_id(&ctx, &request.component_id)
        .await?
        .ok_or(ComponentError::ComponentNotFound(request.component_id))?;

    let component_schema = component
        .schema(&ctx)
        .await?
        .ok_or(ComponentError::SchemaNotFound)?;
    let schema_variant = component_schema.default_variant(&ctx).await?;

    let import_id_prop = Prop::find_prop_by_path(
        &ctx,
        *schema_variant.id(),
        &PropPath::new(["root", "import_id"]),
    )
    .await?;

    let read_context = AttributeReadContext::default_with_prop_and_component_id(
        *import_id_prop.id(),
        Some(*component.id()),
    );
    let av = AttributeValue::find_for_context(&ctx, read_context)
        .await?
        .ok_or(AttributeValueError::NotFoundForReadContext(read_context))?;

    let maybe_parent_av = av.parent_attribute_value(&ctx).await?;
    let parent_av_id = if let Some(parent_av) = maybe_parent_av {
        Some(*parent_av.id())
    } else {
        None
    };

    let attribute_context = AttributeContext::builder()
        .set_prop_id(*import_id_prop.id())
        .set_component_id(request.component_id)
        .to_context()?;
    AttributeValue::update_for_context(
        &ctx,
        *av.id(),
        parent_av_id,
        attribute_context,
        request.value,
        None,
    )
    .await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        "import_id_set",
        serde_json::json!({
            "component_id": component.id(),
            "component_schema_name": component_schema.name(),
        }),
    );

    ctx.commit().await?;

    let mut response = axum::response::Response::builder();
    // if let Some(force_changeset_pk) = force_changeset_pk {
    //     response = response.header("force_changeset_pk", force_changeset_pk.to_string());
    // }
    Ok(response.body(axum::body::Empty::new())?)
}
