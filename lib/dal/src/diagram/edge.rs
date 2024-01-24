//! This module contains the "diagram edge" concept, which is a view of the underlying
//! [`graph`](crate::workspace_snapshot::graph) that shows a connection between two
//! [`Components`](crate::Component);

use serde::{Deserialize, Serialize};

use crate::attribute::prototype::argument::value_source::ValueSource;
use crate::attribute::prototype::argument::{
    AttributePrototypeArgument, AttributePrototypeArgumentId,
};
use crate::diagram::DiagramResult;
use crate::{
    AttributeValue, Component, ComponentId, DalContext, ExternalProviderId, InternalProviderId,
};

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

        // Walk the input socket values for every component and find the attribute prototype
        // arguments for their prototypes
        for component in Component::list(ctx).await? {
            for (ip_value_id, to_explicit_internal_provider_id) in
                component.internal_provider_attribute_values(ctx).await?
            {
                let prototype_id = AttributeValue::prototype_id(ctx, ip_value_id).await?;
                if let Some(apa_id) =
                    AttributePrototypeArgument::list_ids_for_prototype(ctx, prototype_id)
                        .await?
                        .get(0)
                        .copied()
                {
                    let apa = AttributePrototypeArgument::get_by_id(ctx, apa_id).await?;
                    if let Some(targets) = apa.targets() {
                        if let Some(ValueSource::ExternalProvider(from_external_provider_id)) =
                            AttributePrototypeArgument::value_source_by_id(ctx, apa_id).await?
                        {
                            views.push(Self::new(
                                apa_id,
                                targets.source_component_id,
                                from_external_provider_id,
                                targets.destination_component_id,
                                to_explicit_internal_provider_id,
                            ))
                        }
                    }
                }
            }
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
