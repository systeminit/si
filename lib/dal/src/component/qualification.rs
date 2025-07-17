use serde::Deserialize;
use telemetry::prelude::*;

use crate::{
    AttributeValue,
    AttributeValueId,
    Component,
    ComponentError,
    ComponentId,
    DalContext,
    component::ComponentResult,
    qualification::{
        QualificationSubCheckStatus,
        QualificationView,
    },
    schema::variant::root_prop::RootPropChild,
    ws_event::WsEvent,
};

// FIXME(nick): use the formal types from the new version of function authoring instead of this
// struct. This struct is a temporary stopgap until that's implemented.
#[derive(Deserialize, Debug)]
pub struct QualificationEntry {
    pub result: Option<QualificationSubCheckStatus>,
    pub message: Option<String>,
}

impl Component {
    pub async fn list_qualification_avs(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> ComponentResult<Vec<AttributeValue>> {
        let qualification_map_value_id =
            Self::find_qualification_map_attribute_value_id(ctx, component_id).await?;

        Ok(AttributeValue::get_child_avs_in_order(ctx, qualification_map_value_id).await?)
    }

    pub async fn list_qualification_statuses(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> ComponentResult<Vec<Option<QualificationSubCheckStatus>>> {
        let mut statuses = vec![];

        let qualification_avs = Self::list_qualification_avs(ctx, component_id).await?;

        for qualification_av in qualification_avs {
            let Some(qual_value) = AttributeValue::view(ctx, qualification_av.id()).await? else {
                continue;
            };

            let qualification_entry: QualificationEntry = serde_json::from_value(qual_value)?;

            statuses.push(qualification_entry.result);
        }

        if let Some(view) = QualificationView::new_for_validations(ctx, component_id).await? {
            if let Some(result) = view.result {
                statuses.push(Some(result.status))
            } else {
                statuses.push(None)
            }
        }

        Ok(statuses)
    }

    #[instrument(skip_all)]
    pub async fn list_qualifications(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> ComponentResult<Vec<QualificationView>> {
        let mut qualification_views = vec![];

        let qualification_avs = Self::list_qualification_avs(ctx, component_id).await?;

        for qualification_av in qualification_avs {
            if let Some(view) = QualificationView::new(ctx, qualification_av.id()).await? {
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

    /// This method finds the [`AttributeValueId`](crate::AttributeValue) corresponding to "/root/qualifications" for
    /// the given [`ComponentId`](Component).
    pub async fn find_qualification_map_attribute_value_id(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> ComponentResult<AttributeValueId> {
        match Self::attribute_values_for_prop_by_id(
            ctx,
            component_id,
            RootPropChild::Qualification
                .prop_path()
                .as_parts()
                .as_slice(),
        )
        .await?
        .first()
        {
            Some(qualification_map_attribute_value_id) => Ok(*qualification_map_attribute_value_id),
            None => Err(ComponentError::MissingQualificationsValue(component_id)),
        }
    }
}
