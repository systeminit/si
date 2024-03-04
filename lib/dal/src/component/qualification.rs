use serde::Deserialize;
use telemetry::prelude::*;

use crate::component::ComponentResult;
use crate::qualification::{QualificationSubCheckStatus, QualificationView};
use crate::schema::variant::root_prop::RootPropChild;
use crate::ws_event::WsEvent;
use crate::DalContext;
use crate::{Component, ComponentError, ComponentId};

// FIXME(nick): use the formal types from the new version of function authoring instead of this
// struct. This struct is a temporary stopgap until that's implemented.
#[derive(Deserialize, Debug)]
pub struct QualificationEntry {
    pub result: Option<QualificationSubCheckStatus>,
    pub message: Option<String>,
}

impl Component {
    #[instrument(skip_all)]
    pub async fn list_qualifications(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> ComponentResult<Vec<QualificationView>> {
        let component = Self::get_by_id(ctx, component_id).await?;

        let mut qualification_views = vec![];

        let qualification_map_value_id = component
            .attribute_values_for_prop(
                ctx,
                RootPropChild::Qualification
                    .prop_path()
                    .as_parts()
                    .as_slice(),
            )
            .await?
            .iter()
            .next()
            .copied()
            .ok_or(ComponentError::MissingQualificationsValue(component_id))?;

        let qualification_attribute_value_ids = {
            let workspace_snapshot = ctx.workspace_snapshot()?.read().await;
            match workspace_snapshot.ordered_children_for_node(qualification_map_value_id)? {
                Some(value_ids) => value_ids,
                None => return Ok(vec![]), // should probably be an error
            }
        };

        for qualification_attribute_value_id in qualification_attribute_value_ids {
            if let Some(view) =
                QualificationView::new(ctx, qualification_attribute_value_id.into()).await?
            {
                qualification_views.push(view);
            }
        }

        if let Some(view) = QualificationView::new_for_validations(ctx, component_id).await? {
            qualification_views.push(view);
        }

        qualification_views.sort();
        // We want the "all fields valid" to always be first

        WsEvent::checked_qualifications(ctx, component_id)
            .await?
            .publish_on_commit(ctx)
            .await?;

        Ok(qualification_views)
    }
}
