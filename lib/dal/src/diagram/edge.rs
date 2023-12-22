//! This module contains the "diagram edge" concept, which is a view of the underlying
//! [`graph`](crate::workspace_snapshot::graph) that shows a connection between two
//! [`Components`](crate::Component);

use serde::{Deserialize, Serialize};

use crate::attribute::prototype::argument::{
    AttributePrototypeArgument, AttributePrototypeArgumentId,
};
use crate::diagram::{DiagramError, DiagramResult};
use crate::workspace_snapshot::edge_weight::EdgeWeightKindDiscriminants;
use crate::{
    Component, ComponentId, DalContext, ExternalProvider, ExternalProviderId, InternalProviderId,
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

        // Start by gathering all external providers used by all components in the workspace.
        for component in Component::list(ctx).await? {
            let schema_variant_id_used_by_component =
                Component::schema_variant_id(ctx, component.id()).await?;
            let external_providers_used_by_component =
                ExternalProvider::list(ctx, schema_variant_id_used_by_component).await?;

            // Once we have the external providers, find the inter component attribute prototype arguments.
            // We will also need the destination components from their metadata.
            for external_provider in external_providers_used_by_component {
                let inter_component_attribute_prototype_argument_node_indices = {
                    let mut workspace_snapshot = ctx.workspace_snapshot()?.try_lock()?;
                    workspace_snapshot.incoming_sources_for_edge_weight_kind(
                        external_provider.id(),
                        EdgeWeightKindDiscriminants::InterComponent,
                    )?
                };

                for inter_component_attribute_prototype_argument_node_index in
                    inter_component_attribute_prototype_argument_node_indices
                {
                    // Cache the destination component from the inter component attribute prototype argument.
                    let inter_component_attribute_prototype_argument_id_raw = {
                        let mut workspace_snapshot = ctx.workspace_snapshot()?.try_lock()?;
                        workspace_snapshot
                            .get_node_weight(
                                inter_component_attribute_prototype_argument_node_index,
                            )?
                            .id()
                    };
                    let inter_component_attribute_prototype_argument =
                        AttributePrototypeArgument::get_by_id(
                            ctx,
                            inter_component_attribute_prototype_argument_id_raw.into(),
                        )
                        .await?;
                    let to_component_id = inter_component_attribute_prototype_argument
                        .inter_component_metadata()
                        .ok_or(DiagramError::InterComponentMetadataNotFound(
                            inter_component_attribute_prototype_argument.id(),
                            external_provider.id(),
                        ))?
                        .destination_component_id;

                    // Now, we need to find the explicit internal provider. Start by finding the attribute prototype.
                    let mut workspace_snapshot = ctx.workspace_snapshot()?.try_lock()?;
                    let attribute_prototype_node_index = *workspace_snapshot
                        .incoming_sources_for_edge_weight_kind(
                            inter_component_attribute_prototype_argument_id_raw,
                            EdgeWeightKindDiscriminants::PrototypeArgument,
                        )?
                        .get(0)
                        .ok_or(DiagramError::DestinationAttributePrototypeNotFound(
                            inter_component_attribute_prototype_argument.id(),
                        ))?;
                    let attribute_prototype_id_raw = workspace_snapshot
                        .get_node_weight(attribute_prototype_node_index)?
                        .id();

                    // Use the attribute prototype to find the explicit internal provider.
                    let explicit_internal_provider_node_index = *workspace_snapshot
                        .incoming_sources_for_edge_weight_kind(
                            attribute_prototype_id_raw,
                            EdgeWeightKindDiscriminants::Prototype,
                        )?
                        .get(0)
                        .ok_or(DiagramError::DestinationExplicitInternalProviderNotFound(
                            attribute_prototype_id_raw.into(),
                            inter_component_attribute_prototype_argument.id(),
                        ))?;
                    let explicit_internal_provider_node_weight = workspace_snapshot
                        .get_node_weight(explicit_internal_provider_node_index)?;

                    // We have all the information we need to assemble a diagram edge view.
                    views.push(Self::new(
                        inter_component_attribute_prototype_argument.id(),
                        component.id(),
                        external_provider.id(),
                        to_component_id,
                        explicit_internal_provider_node_weight.id().into(),
                    ));
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
