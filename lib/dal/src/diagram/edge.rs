//! This module contains the "diagram edge" concept, which is a view of the underlying
//! [`graph`](crate::workspace_snapshot::graph) that shows a connection between two
//! [`Components`](crate::Component);

use serde::{Deserialize, Serialize};

use crate::attribute::prototype::argument::AttributePrototypeArgumentId;
use crate::component::IncomingConnection;
use crate::diagram::DiagramResult;
use crate::{Component, ComponentId, DalContext, ExternalProviderId, InternalProviderId};

pub type DiagramEdgeViewId = AttributePrototypeArgumentId;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DiagramEdgeView {
    pub id: String,
    pub from_component_id: String,
    pub from_external_provider_id: String,
    pub to_component_id: String,
    pub to_explicit_internal_provider_id: String,
    // pub change_status: ChangeStatus,
    // pub created_info: Option<HistoryEventMetadata>,
    // pub deleted_info: Option<HistoryEventMetadata>,
}

impl DiagramEdgeView {
    pub async fn list(ctx: &DalContext) -> DiagramResult<Vec<Self>> {
        let mut views = Vec::new();

        for component in Component::list(ctx).await? {
            let incoming_connections = component.incoming_connections(ctx).await?;

            views.extend(incoming_connections.iter().map(
                |IncomingConnection {
                     attribute_prototype_argument_id,
                     to_component_id,
                     to_internal_provider_id,
                     from_component_id,
                     from_external_provider_id,
                 }| {
                    DiagramEdgeView::new(
                        *attribute_prototype_argument_id,
                        *from_component_id,
                        *from_external_provider_id,
                        *to_component_id,
                        *to_internal_provider_id,
                    )
                },
            ))
        }

        Ok(views)
    }

    fn new(
        id: DiagramEdgeViewId,
        from_component_id: ComponentId,
        from_external_provider_id: ExternalProviderId,
        to_component_id: ComponentId,
        to_explicit_internal_provider_id: InternalProviderId,
    ) -> Self {
        Self {
            id: id.to_string(),
            from_component_id: from_component_id.to_string(),
            from_external_provider_id: from_external_provider_id.to_string(),
            to_component_id: to_component_id.to_string(),
            to_explicit_internal_provider_id: to_explicit_internal_provider_id.to_string(),
        }
    }
}
