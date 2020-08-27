use si_data::{DataError, Db};

pub use crate::protobuf::{Resource, ResourceUpdateReply, ResourceUpdateRequest};

use tracing::info_span;
use tracing_futures::Instrument as _;

impl Resource {
    pub async fn update(
        db: &Db,
        request: ResourceUpdateRequest,
    ) -> Result<ResourceUpdateReply, DataError> {
        let span = info_span!(
            "si.core.resource.update",
            resource.id = tracing::field::Empty,
        );
        async {
            let span = tracing::Span::current();

            let resource_id = request
                .id
                .ok_or_else(|| DataError::RequiredField("id".into()))?;
            span.record("resource.id", &tracing::field::display(&resource_id));
            let mut resource = Resource::get(db, &resource_id).await?;

            let new_entity_id = request
                .entity_id
                .ok_or_else(|| DataError::RequiredField("entity_id".into()))?;
            resource.entity_id = Some(new_entity_id);
            let new_node_id = request
                .node_id
                .ok_or_else(|| DataError::RequiredField("node_id".into()))?;
            resource.node_id = Some(new_node_id);
            let new_kind = request
                .kind
                .ok_or_else(|| DataError::RequiredField("kind".into()))?;
            resource.kind = Some(new_kind);
            let new_status = request
                .status
                .ok_or_else(|| DataError::RequiredField("status".into()))?;
            resource.status = Some(new_status);
            let new_health = request
                .health
                .ok_or_else(|| DataError::RequiredField("health".into()))?;
            resource.health = Some(new_health);
            let new_data = request
                .data
                .ok_or_else(|| DataError::RequiredField("data".into()))?;
            resource.data = Some(new_data);

            resource.save(db).await?;

            Ok(ResourceUpdateReply {
                item: Some(resource),
            })
        }
        .instrument(span)
        .await
    }
}
