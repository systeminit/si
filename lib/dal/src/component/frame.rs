use serde::{Deserialize, Serialize};
use telemetry::prelude::*;
use thiserror::Error;

use crate::attribute::value::AttributeValueError;
use crate::socket::input::InputSocketError;
use crate::workspace_snapshot::edge_weight::{EdgeWeightError, EdgeWeightKind};
use crate::workspace_snapshot::WorkspaceSnapshotError;
use crate::{
    AttributeValue, Component, ComponentError, ComponentId, ComponentType, DalContext, InputSocket,
    TransactionsError,
};

use super::{InputSocketMatch, OutputSocketMatch};

#[remain::sorted]
#[derive(Error, Debug)]
pub enum FrameError {
    #[error("aggregation frames unsupported: {0}")]
    AggregateFramesUnsupported(ComponentId),
    #[error("attribute value error: {0}")]
    AttributeValueError(#[from] AttributeValueError),
    #[error("component error: {0}")]
    Component(#[from] ComponentError),
    #[error("edge weight error: {0}")]
    EdgeWeight(#[from] EdgeWeightError),
    #[error("input socket error: {0}")]
    InputSocketError(#[from] InputSocketError),
    #[error("parent is not a frame (child id: {0}) (parent id: {1})")]
    ParentIsNotAFrame(ComponentId, ComponentId),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] WorkspaceSnapshotError),
}

pub type FrameResult<T> = Result<T, FrameError>;

/// A unit struct containing logic for working with frames.
pub struct Frame;

impl Frame {
    /// Provides an ability to remove the existing ['Component']'s parent``
    #[instrument(level = "info", skip_all)]
    pub async fn orphan_child(ctx: &DalContext, child_id: ComponentId) -> FrameResult<()> {
        if let Some(parent_id) = Component::get_parent_by_id(ctx, child_id).await? {
            Self::detach_child_from_parent_inner(ctx, parent_id, child_id).await?;
        }
        Ok(())
    }
    /// Provides the ability to attach or replace a child [`Component`]'s parent
    #[instrument(level = "info", skip_all)]
    pub async fn upsert_parent(
        ctx: &DalContext,
        child_id: ComponentId,
        new_parent_id: ComponentId,
    ) -> FrameResult<()> {
        // let's see if we need to even do anything
        if let Some(current_parent_id) = Component::get_parent_by_id(ctx, child_id).await? {
            if current_parent_id == new_parent_id {
                return Ok(());
            }
        }

        match Component::get_type_by_id(ctx, new_parent_id).await? {
            ComponentType::ConfigurationFrameDown | ComponentType::ConfigurationFrameUp => {
                Self::orphan_child(ctx, child_id).await?;
                Self::attach_child_to_parent_inner(ctx, new_parent_id, child_id).await?;
            }
            ComponentType::Component => {
                return Err(FrameError::ParentIsNotAFrame(child_id, new_parent_id))
            }
            ComponentType::AggregationFrame => {
                return Err(FrameError::AggregateFramesUnsupported(new_parent_id))
            }
        }
        Ok(())
    }

    async fn attach_child_to_parent_inner(
        ctx: &DalContext,
        parent_id: ComponentId,
        child_id: ComponentId,
    ) -> FrameResult<()> {
        let cycle_check_guard = ctx.workspace_snapshot()?.enable_cycle_check().await;
        Component::add_edge_to_frame(ctx, parent_id, child_id, EdgeWeightKind::FrameContains)
            .await?;
        drop(cycle_check_guard);

        // when attaching a child, need to rerun any attribute prototypes for those impacted sockets then queue up dvu!

        let values_rerun = match Component::get_type_by_id(ctx, child_id).await? {
            ComponentType::Component
            | ComponentType::ConfigurationFrameDown
            | ComponentType::ConfigurationFrameUp => {
                let mut updated_avs = vec![];
                for (_, input_socket_match) in
                    Component::input_socket_attribute_values_for_component_id(ctx, child_id).await?
                {
                    if !InputSocket::is_manually_configured(ctx, input_socket_match).await? {
                        AttributeValue::update_from_prototype_function(
                            ctx,
                            input_socket_match.attribute_value_id,
                        )
                        .await?;
                        updated_avs.push(input_socket_match.attribute_value_id);
                    }
                }
                let mut work_queue = vec![parent_id];
                while let Some(parent) = work_queue.pop() {
                    if Component::get_type_by_id(ctx, parent).await?
                        == ComponentType::ConfigurationFrameUp
                    {
                        for (_, input_socket_match) in
                            Component::input_socket_attribute_values_for_component_id(ctx, child_id)
                                .await?
                        {
                            if !InputSocket::is_manually_configured(ctx, input_socket_match).await?
                            {
                                AttributeValue::update_from_prototype_function(
                                    ctx,
                                    input_socket_match.attribute_value_id,
                                )
                                .await?;
                                updated_avs.push(input_socket_match.attribute_value_id);
                            }
                        }
                    }
                    if let Some(next_parent) = Component::get_parent_by_id(ctx, parent).await? {
                        work_queue.push(next_parent);
                    }
                }
                updated_avs
            }
            ComponentType::AggregationFrame => vec![],
        };
        ctx.enqueue_dependent_values_update(values_rerun).await?;

        Ok(())
    }
    async fn detach_child_from_parent_inner(
        ctx: &DalContext,
        parent_id: ComponentId,
        child_id: ComponentId,
    ) -> FrameResult<()> {
        //when detaching a child, need to re-run any attribute prototypes for those impacted input sockets then queue up dvu!

        let values_rerun = match Component::get_type_by_id(ctx, child_id).await? {
            ComponentType::Component
            | ComponentType::ConfigurationFrameDown
            | ComponentType::ConfigurationFrameUp => {
                let mut updated_avs = vec![];
                for (_, input_socket_match) in
                    Component::input_socket_attribute_values_for_component_id(ctx, child_id).await?
                {
                    if !InputSocket::is_manually_configured(ctx, input_socket_match).await? {
                        AttributeValue::update_from_prototype_function(
                            ctx,
                            input_socket_match.attribute_value_id,
                        )
                        .await?;
                        updated_avs.push(input_socket_match.attribute_value_id);
                    }
                }
                let mut work_queue = vec![parent_id];
                while let Some(parent) = work_queue.pop() {
                    if Component::get_type_by_id(ctx, parent).await?
                        == ComponentType::ConfigurationFrameUp
                    {
                        for (_, input_socket_match) in
                            Component::input_socket_attribute_values_for_component_id(ctx, child_id)
                                .await?
                        {
                            if !InputSocket::is_manually_configured(ctx, input_socket_match).await?
                            {
                                AttributeValue::update_from_prototype_function(
                                    ctx,
                                    input_socket_match.attribute_value_id,
                                )
                                .await?;
                                updated_avs.push(input_socket_match.attribute_value_id);
                            }
                        }
                    }
                    if let Some(next_parent) = Component::get_parent_by_id(ctx, parent).await? {
                        work_queue.push(next_parent);
                    }
                }
                updated_avs
            }
            ComponentType::AggregationFrame => vec![],
        };

        Component::remove_edge_from_frame(ctx, parent_id, child_id).await?;
        ctx.enqueue_dependent_values_update(values_rerun).await?;
        Ok(())
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct InferredConnection {
    pub input_socket_match: InputSocketMatch,
    pub output_socket_match: OutputSocketMatch,
}
