use crate::workspace_snapshot::edge_weight::EdgeWeightKindDiscriminants;
use crate::{
    code_view::CodeView, schema::variant::root_prop::RootPropChild, AttributeValueId, Component,
    ComponentId, DalContext,
};

use super::{ComponentError, ComponentResult};

impl Component {
    /// List all [`CodeViews`](crate::CodeView) for based on the "code generation"
    /// [`leaves`](crate::schema::variant::leaves) for a given [`ComponentId`](Self).
    pub async fn list_code_generated(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> ComponentResult<(Vec<CodeView>, bool)> {
        let component = Self::get_by_id(ctx, component_id).await?;
        let _schema_variant = component.schema_variant(ctx).await?;

        let mut code_views: Vec<CodeView> = vec![];

        let code_av_props = component
            .attribute_values_for_prop(ctx, RootPropChild::Code.prop_path().as_parts().as_slice())
            .await?;

        if code_av_props.is_empty() {
            return Ok((code_views, false));
        }

        let code_value_id = code_av_props
            .first()
            .copied()
            .ok_or(ComponentError::MissingCodeValue(component_id))?;

        let mut child_av_ids: Vec<AttributeValueId> = vec![];
        let workspace_snapshot = ctx.workspace_snapshot()?;
        for child_target in workspace_snapshot
            .outgoing_targets_for_edge_weight_kind(
                code_value_id,
                EdgeWeightKindDiscriminants::Contain,
            )
            .await?
        {
            let av_id = workspace_snapshot.get_node_weight(child_target).await?.id();
            child_av_ids.push(av_id.into());
        }

        if child_av_ids.is_empty() {
            return Ok((vec![], false));
        }

        for child_av in child_av_ids {
            if let Some(code_view) = CodeView::new(ctx, child_av).await? {
                code_views.push(code_view)
            }
        }

        Ok((code_views.clone(), true))
    }

    // TODO(nick): big query potential.
    // Returns a [`HashSet`](std::collections::HashSet) of all the
    // [`AttributeValueIds`](crate::AttributeValue) corresponding to "code generation"
    // [`leaves`](crate::schema::variant::leaves) in the workspace.
    // pub async fn all_code_generation_attribute_values(
    //     ctx: &DalContext,
    // ) -> ComponentResult<HashSet<AttributeValueId>> {
    //     let mut values = HashSet::new();
    //     for component in Component::list(ctx).await? {
    //         values.extend(
    //             Self::all_code_generation_attribute_values_for_component(ctx, component.id())
    //                 .await?,
    //         );
    //     }
    //     Ok(values)
    // }

    // TODO(nick): big query potential.
    // Returns a [`HashSet`](std::collections::HashSet) of all the
    // [`AttributeValueIds`](crate::AttributeValue) corresponding to "code generation"
    // [`leaves`](crate::schema::variant::leaves) for a given [`ComponentId`](Self).
    // async fn all_code_generation_attribute_values_for_component(
    //     ctx: &DalContext,
    //     component_id: ComponentId,
    // ) -> ComponentResult<HashSet<AttributeValueId>> {
    //     let code_map_attribute_value = Self::root_prop_child_attribute_value_for_component(
    //         ctx,
    //         component_id,
    //         RootPropChild::Code,
    //     )
    //     .await?;
    //     Ok(HashSet::from_iter(
    //         code_map_attribute_value
    //             .child_attribute_values(ctx)
    //             .await?
    //             .iter()
    //             .map(|av| *av.id()),
    //     ))
    // }
}

// NOTE(nick): consider moving this somewhere else.
// #[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
// #[serde(rename_all = "camelCase")]
// pub struct CodeGeneratedPayload {
//     component_id: ComponentId,
// }

// NOTE(nick): consider moving this somewhere else.
// impl WsEvent {
//     pub async fn code_generated(
//         ctx: &DalContext,
//         component_id: ComponentId,
//     ) -> WsEventResult<Self> {
//         WsEvent::new(
//             ctx,
//             WsPayload::CodeGenerated(CodeGeneratedPayload { component_id }),
//         )
//         .await
//     }
// }
