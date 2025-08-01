use std::collections::VecDeque;

use dal::{
    AttributePrototype,
    AttributeValue,
    Component,
    DalContext,
    Prop,
    attribute::prototype::argument::{
        AttributePrototypeArgument,
        value_source::ValueSource,
    },
};
use si_frontend_mv_types::incoming_connections::{
    Connection,
    IncomingConnections as IncomingConnectionsMv,
    ManagementConnections as ManagementConnectionsMv,
};
use si_id::ComponentId;
use telemetry::prelude::*;

use crate::Result;

#[instrument(
    name = "dal_materialized_views.incoming_connections",
    level = "debug",
    skip_all
)]
pub async fn assemble(ctx: DalContext, component_id: ComponentId) -> Result<IncomingConnectionsMv> {
    let ctx = &ctx;

    let mut connections = prop_to_prop(ctx, component_id).await?;
    connections.sort();

    Ok(IncomingConnectionsMv {
        id: component_id,
        connections,
    })
}

#[instrument(
    name = "dal_materialized_views.outgoing_mgmt_connections",
    level = "debug",
    skip_all
)]
pub async fn assemble_management(
    ctx: DalContext,
    component_id: ComponentId,
) -> Result<ManagementConnectionsMv> {
    let ctx = &ctx;

    let mut connections = management(ctx, component_id).await?;
    connections.sort();

    Ok(ManagementConnectionsMv {
        id: component_id,
        connections,
    })
}

async fn prop_to_prop(ctx: &DalContext, component_id: ComponentId) -> Result<Vec<Connection>> {
    let mut connections = Vec::new();

    let root_attribute_value_id = Component::root_attribute_value_id(ctx, component_id).await?;
    let mut work_queue = VecDeque::from([root_attribute_value_id]);

    while let Some(attribute_value_id) = work_queue.pop_front() {
        work_queue
            .extend(AttributeValue::get_child_av_ids_in_order(ctx, attribute_value_id).await?);

        let attribute_prototype_id = AttributeValue::prototype_id(ctx, attribute_value_id).await?;

        // Find all connections for the given attribute value. We'll collect everything into an
        // "in progress" cache to ensure that we don't perform unnecessary setup work if no
        // connections are found.
        let ap_args = AttributePrototype::list_arguments(ctx, attribute_prototype_id).await?;
        let mut in_progress = Vec::with_capacity(ap_args.len());
        for attribute_prototype_argument_id in ap_args {
            if let Some(ValueSource::ValueSubscription(subscription)) =
                AttributePrototypeArgument::value_source_opt(ctx, attribute_prototype_argument_id)
                    .await?
            {
                // If we can successfully resolve the subscription, we have found a connection!
                // Let's push the information we need into our "in progress" cache for later.
                if let Some(from_attribute_value_id) = subscription.resolve(ctx).await? {
                    let (_, from_attribute_value_path) =
                        AttributeValue::path_from_root(ctx, from_attribute_value_id).await?;
                    let from_component_id =
                        AttributeValue::component_id(ctx, from_attribute_value_id).await?;
                    let from_prop_id =
                        AttributeValue::prop_id(ctx, from_attribute_value_id).await?;
                    let from_prop_path = Prop::path_by_id(ctx, from_prop_id)
                        .await?
                        .with_replaced_sep_and_prefix("/");
                    in_progress.push((
                        from_component_id,
                        from_attribute_value_path,
                        from_attribute_value_id,
                        from_prop_id,
                        from_prop_path,
                    ))
                }
            }
        }

        // Only perform connections population setup if we have found connections.
        if !in_progress.is_empty() {
            let prop_id = AttributeValue::prop_id(ctx, attribute_value_id).await?;
            let prop_path = Prop::path_by_id(ctx, prop_id)
                .await?
                .with_replaced_sep_and_prefix("/");
            let (_, attribute_value_path) =
                AttributeValue::path_from_root(ctx, attribute_value_id).await?;
            connections.reserve(in_progress.len());

            for (
                from_component_id,
                from_attribute_value_path,
                from_attribute_value_id,
                from_prop_id,
                from_prop_path,
            ) in in_progress
            {
                connections.push(Connection::Prop {
                    from_component_id,
                    from_attribute_value_id,
                    from_attribute_value_path,
                    from_prop_id,
                    from_prop_path,
                    to_component_id: component_id,
                    to_prop_id: prop_id,
                    to_prop_path: prop_path.clone(),
                    to_attribute_value_id: attribute_value_id,
                    to_attribute_value_path: attribute_value_path.clone(),
                })
            }
        }
    }

    Ok(connections)
}

async fn management(ctx: &DalContext, component_id: ComponentId) -> Result<Vec<Connection>> {
    let component_managed_by_ids = Component::get_managed_by_id(ctx, component_id).await?;
    let mut connections = Vec::with_capacity(component_managed_by_ids.len());

    for managed_component_id in component_managed_by_ids {
        connections.push(Connection::Management {
            from_component_id: component_id,
            to_component_id: managed_component_id,
        })
    }

    Ok(connections)
}
