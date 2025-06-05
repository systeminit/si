use std::collections::VecDeque;

use dal::{
    AttributePrototype,
    AttributeValue,
    Component,
    DalContext,
    InputSocket,
    OutputSocket,
    Prop,
    attribute::prototype::argument::{
        AttributePrototypeArgument,
        value_source::ValueSource,
    },
};
use si_frontend_mv_types::incoming_connections::{
    Connection,
    IncomingConnections as IncomingConnectionsMv,
};
use si_id::ComponentId;
use telemetry::prelude::*;

use crate::{
    Error,
    Result,
};

#[instrument(
    name = "dal_materialized_views.incoming_connections",
    level = "debug",
    skip_all
)]
pub async fn assemble(ctx: DalContext, component_id: ComponentId) -> Result<IncomingConnectionsMv> {
    let ctx = &ctx;

    let mut connections = Vec::new();
    connections.extend(socket_to_socket(ctx, component_id).await?);
    connections.extend(prop_to_prop(ctx, component_id).await?);
    connections.sort();

    Ok(IncomingConnectionsMv {
        id: component_id,
        connections,
    })
}

async fn socket_to_socket(ctx: &DalContext, component_id: ComponentId) -> Result<Vec<Connection>> {
    let schema_variant = Component::schema_variant_for_component_id(ctx, component_id).await?;
    let input_sockets = InputSocket::list(ctx, schema_variant.id()).await?;

    let mut connections = Vec::new();
    for input_socket in input_sockets {
        let input_socket_id = input_socket.id();
        let input_socket_attribute_value_id =
            InputSocket::component_attribute_value_id(ctx, input_socket_id, component_id).await?;
        let attribute_prototype_id =
            AttributeValue::prototype_id(ctx, input_socket_attribute_value_id).await?;
        let mut attribute_prototype_argument_ids =
            AttributePrototype::list_arguments(ctx, attribute_prototype_id).await?;
        attribute_prototype_argument_ids.sort();

        // Don't bother gathering information to cache if there are no prototype arguments.
        if attribute_prototype_argument_ids.is_empty() {
            continue;
        }

        let input_socket_attribute_value_path =
            AttributeValue::get_path_for_id(ctx, input_socket_attribute_value_id)
                .await?
                .unwrap_or_default();

        for attribute_prototype_argument_id in attribute_prototype_argument_ids {
            let attribute_prototype_argument =
                AttributePrototypeArgument::get_by_id(ctx, attribute_prototype_argument_id).await?;
            let value_source =
                AttributePrototypeArgument::value_source(ctx, attribute_prototype_argument_id)
                    .await?;
            let ValueSource::OutputSocket(output_socket_id) = value_source else {
                return Err(Error::InvalidValueSource(value_source));
            };
            let output_socket = OutputSocket::get_by_id(ctx, output_socket_id).await?;

            // We've found the connection if the attribute prototype argument has targets whose
            // destination component ID for the current component. We also need to ensure that we
            // do not accidentally collect an outgoing connection.
            let source_component_id = match attribute_prototype_argument.targets() {
                Some(targets) if targets.destination_component_id == component_id => {
                    targets.source_component_id
                }
                Some(_) | None => continue,
            };

            let output_socket_attribute_value_id = OutputSocket::component_attribute_value_id(
                ctx,
                output_socket_id,
                source_component_id,
            )
            .await?;
            let output_socket_attribute_value_path =
                AttributeValue::get_path_for_id(ctx, output_socket_attribute_value_id)
                    .await?
                    .unwrap_or_default();

            connections.push(Connection::Socket {
                from_component_id: source_component_id.into(),
                from_attribute_value_id: output_socket_attribute_value_id,
                from_attribute_value_path: output_socket_attribute_value_path,
                from_socket_id: output_socket_id,
                from_socket_name: output_socket.name().to_string(),
                to_component_id: component_id.into(),
                to_socket_id: input_socket_id,
                to_socket_name: input_socket.name().to_string(),
                to_attribute_value_id: input_socket_attribute_value_id,
                to_attribute_value_path: input_socket_attribute_value_path.clone(),
            });
        }
    }

    Ok(connections)
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
        let mut in_progress = Vec::new();
        for attribute_prototype_argument_id in
            AttributePrototype::list_arguments(ctx, attribute_prototype_id).await?
        {
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

            for (
                from_component_id,
                from_attribute_value_path,
                from_attribute_value_id,
                from_prop_id,
                from_prop_path,
            ) in in_progress
            {
                connections.push(Connection::Prop {
                    from_component_id: from_component_id.into(),
                    from_attribute_value_id,
                    from_attribute_value_path,
                    from_prop_id,
                    from_prop_path,
                    to_component_id: component_id.into(),
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
