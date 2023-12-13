use crate::server::extract::{AccessBuilder, HandlerContext};
use crate::service::component::{ComponentError, ComponentResult};
use axum::response::IntoResponse;
use axum::Json;
use dal::{
    AttributeReadContext, AttributeValue, AttributeValueId, ChangeSet, ComponentId,
    DependentValuesUpdate, Prop, PropId, PropKind, StandardModel, Visibility, WsEvent,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DeletePropertyEditorValueRequest {
    pub attribute_value_id: AttributeValueId,
    pub prop_id: PropId,
    pub component_id: ComponentId,
    pub key: Option<String>,
    #[serde(flatten)]
    pub visibility: Visibility,
}

pub async fn delete_property_editor_value(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<DeletePropertyEditorValueRequest>,
) -> ComponentResult<impl IntoResponse> {
    let mut ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let mut force_changeset_pk = None;
    if ctx.visibility().is_head() {
        let change_set = ChangeSet::new(&ctx, ChangeSet::generate_name(), None).await?;

        let new_visibility = Visibility::new(change_set.pk, request.visibility.deleted_at);

        ctx.update_visibility(new_visibility);

        force_changeset_pk = Some(change_set.pk);

        WsEvent::change_set_created(&ctx, change_set.pk)
            .await?
            .publish_on_commit(&ctx)
            .await?;
    };

    let child_prop = Prop::get_by_id(&ctx, &request.prop_id)
        .await?
        .ok_or(ComponentError::PropNotFound(request.prop_id))?;

    let parent_prop = child_prop
        .parent_prop(&ctx)
        .await?
        .ok_or(ComponentError::RootPropAttributeValue)?;

    let mut av = AttributeValue::get_by_id(&ctx, &request.attribute_value_id)
        .await?
        .ok_or(ComponentError::AttributeValueNotFound)?;

    let parent_prop_attribute_read_context = AttributeReadContext {
        prop_id: Some(*parent_prop.id()),
        internal_provider_id: None,
        external_provider_id: None,
        component_id: Some(request.component_id),
    };

    let mut parent_av = AttributeValue::find_for_context(&ctx, parent_prop_attribute_read_context)
        .await?
        .ok_or(ComponentError::AttributeValueNotFound)?;

    if *parent_prop.kind() == PropKind::Array {
        let array_key = request.key.clone().ok_or(ComponentError::KeyNotFound)?;

        let index_map = parent_av
            .index_map_mut()
            .map(|index_map| index_map.to_owned());

        if let Some(mut index_map) = index_map {
            let (_, avi_id) = index_map
                .order_as_map()
                .iter()
                .find(|(key, _)| key == &array_key)
                .ok_or(ComponentError::AttributeValueNotFound)?
                .to_owned();

            av.delete_by_id(&ctx).await?;
            index_map.delete(avi_id);
            parent_av.index_map = Some(index_map.to_owned());
            parent_av.update_stored_index_map(&ctx).await?;
        }
    }

    if *parent_prop.kind() == PropKind::Map {
        av.delete_by_id(&ctx).await?;
    }

    WsEvent::change_set_written(&ctx)
        .await?
        .publish_on_commit(&ctx)
        .await?;

    ctx.enqueue_job(DependentValuesUpdate::new(
        ctx.access_builder(),
        *ctx.visibility(),
        vec![*parent_av.id()],
    ))
    .await?;

    ctx.commit().await?;

    let mut response = axum::response::Response::builder();
    if let Some(force_changeset_pk) = force_changeset_pk {
        response = response.header("force_changeset_pk", force_changeset_pk.to_string());
    }
    Ok(response.body(axum::body::Empty::new())?)
}
