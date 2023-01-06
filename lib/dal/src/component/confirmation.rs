use serde::Deserialize;
use serde::Serialize;
use telemetry::prelude::*;

use crate::attribute::value::AttributeValue;
use crate::component::{
    ComponentResult, LIST_ALL_RESOURCE_IMPLICIT_INTERNAL_PROVIDER_ATTRIBUTE_VALUES,
};
use crate::func::binding_return_value::FuncBindingReturnValueId;
use crate::job::definition::DependentValuesUpdate;
use crate::ws_event::WsEvent;
use crate::{standard_model, DalContext, StandardModel, WsEventResult, WsPayload};
use crate::{Component, ComponentId};

// TODO(nick): replace existing view with this unused view.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ConfirmationStatusView {
    Running,
    Failure,
    Success,
}

// TODO(nick): replace existing view with this unused view.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfirmationView {
    func_binding_return_value_id: FuncBindingReturnValueId,
    title: String,
    component_id: ComponentId,
    description: Option<String>,
    output: Option<Vec<String>>,
    status: ConfirmationStatusView,
}

// TODO(nick): use this for listing confirmations, like qualifications in the future.
// FIXME(nick): use the formal types from the new version of function authoring instead of this
// struct. This struct is a temporary stopgap until that's implemented.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ConfirmationEntry {
    pub success: bool,
    #[serde(default)]
    pub recommended_actions: Vec<String>,
}

impl Component {
    /// List all [`AttributeValues`](crate::AttributeValue) whose
    /// [`AttributeContext`](crate::AttributeContext) contains a populated [`ComponentId`](Self)
    /// and a populated [`InternalProviderId`](crate::InternalProvider) where the latter is the
    /// ID for the _implicit_ [`InternalProvider`](crate::InternalProvider) corresponding to
    /// "/root/resource" (child of [`RootProp`](crate::RootProp).
    ///
    /// In other words, this query should find as many [`AttributeValues`](crate::AttributeValue)
    /// as there are [`Components`](Self) in the workspace.
    #[instrument(skip_all)]
    pub async fn list_all_resource_implicit_internal_provider_attribute_values(
        ctx: &DalContext,
    ) -> ComponentResult<Vec<AttributeValue>> {
        let rows = ctx
            .txns()
            .pg()
            .query(
                LIST_ALL_RESOURCE_IMPLICIT_INTERNAL_PROVIDER_ATTRIBUTE_VALUES,
                &[ctx.read_tenancy(), ctx.visibility()],
            )
            .await?;
        Ok(standard_model::objects_from_rows(rows)?)
    }

    /// Run confirmations for all [`Components`](Self) in the workspace by running a
    /// [`DependentValuesUpdate`](crate::job::definition::DependentValuesUpdate) job for every
    /// [`AttributeValue`](crate::AttributeValue) corresponding to the "/root/resource" implicit
    /// [`InternalProvider`](crate::InternalProvider) for every [`Component`](crate::Component).
    pub async fn run_all_confirmations(ctx: &DalContext) -> ComponentResult<()> {
        for resource_attribute_value in
            Component::list_all_resource_implicit_internal_provider_attribute_values(ctx).await?
        {
            ctx.enqueue_job(DependentValuesUpdate::new(
                ctx,
                *resource_attribute_value.id(),
            ))
            .await;
        }

        WsEvent::ran_confirmations(ctx).await?;

        Ok(())
    }
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ConfirmationRunPayload {
    success: bool,
}

impl WsEvent {
    pub async fn ran_confirmations(ctx: &DalContext) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::RanConfirmations(ConfirmationRunPayload { success: true }),
        )
        .await
    }
}
