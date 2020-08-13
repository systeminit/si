use si_data::{DataError, Db};

pub use crate::protobuf::{Node, NodeSetPositionReply, NodeSetPositionRequest};

use tracing::info_span;
use tracing_futures::Instrument as _;

impl Node {
    pub async fn set_position(
        db: &Db,
        request: NodeSetPositionRequest,
    ) -> Result<NodeSetPositionReply, DataError> {
        let span = info_span!(
            "si.core.node.set_position",
            node.id = tracing::field::Empty,
            node.position = tracing::field::Empty,
        );
        async {
            let span = tracing::Span::current();

            let node_id = request
                .id
                .ok_or_else(|| DataError::RequiredField("id".into()))?;
            span.record("node.id", &tracing::field::display(&node_id));
            let position = request
                .position
                .ok_or_else(|| DataError::RequiredField("position".into()))?;
            span.record("node.position", &tracing::field::debug(&position));
            let mut node = Node::get(db, &node_id).await?;
            node.position = Some(position);
            node.save(db).await?;

            Ok(NodeSetPositionReply { item: Some(node) })
        }
        .instrument(span)
        .await
    }
}
