use dal::{
    AttributePrototype,
    AttributeValue,
    Component,
    DalContext,
    InputSocket,
    OutputSocket,
    attribute::prototype::argument::{
        AttributePrototypeArgument,
        value_source::ValueSource,
    },
};
use si_frontend_types::newhotness::component_connections::{
    ComponentConnectionsBeta as ComponentConnectionsMv,
    Connection,
};
use si_id::ComponentId;
use telemetry::prelude::*;

use crate::{
    Error,
    Result,
};

#[instrument(
    name = "dal_materialized_views.component_connections",
    level = "debug",
    skip_all
)]
pub async fn assemble(
    ctx: DalContext,
    component_id: ComponentId,
) -> Result<ComponentConnectionsMv> {
    let ctx = &ctx;

    let mut incoming = Vec::new();
    incoming.extend(incoming_socket_connections(ctx, component_id).await?);
    incoming.extend(incoming_prop_connections(ctx, component_id).await?);
    incoming.sort();

    let mut outgoing = Vec::new();
    outgoing.extend(outgoing_socket_connections(ctx, component_id).await?);
    outgoing.extend(outgoing_prop_connections(ctx, component_id).await?);
    outgoing.sort();

    Ok(ComponentConnectionsMv {
        id: component_id,
        incoming,
        outgoing,
    })
}

async fn incoming_socket_connections(
    ctx: &DalContext,
    component_id: ComponentId,
) -> Result<Vec<Connection>> {
    let schema_variant = Component::schema_variant_for_component_id(ctx, component_id).await?;
    let input_socket_ids =
        InputSocket::list_ids_for_schema_variant(ctx, schema_variant.id()).await?;

    let mut connections = Vec::new();
    for input_socket_id in input_socket_ids {
        let input_socket = InputSocket::get_by_id(ctx, input_socket_id).await?;
        let input_socket_attribute_value_id =
            InputSocket::component_attribute_value_for_input_socket_id(
                ctx,
                input_socket_id,
                component_id,
            )
            .await?;
        let input_socket_attribute_value_path =
            AttributeValue::get_path_for_id(ctx, input_socket_attribute_value_id)
                .await?
                .ok_or(Error::EmptyPathForAttributeValue(
                    input_socket_attribute_value_id,
                ))?;
        let attribute_prototype_id =
            AttributeValue::prototype_id(ctx, input_socket_attribute_value_id).await?;
        let attribute_prototype_argument_ids =
            AttributePrototype::list_arguments_for_id(ctx, attribute_prototype_id).await?;

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

            if let Some(targets) = attribute_prototype_argument.targets() {
                let output_socket_attribute_value_id =
                    OutputSocket::component_attribute_value_for_output_socket_id(
                        ctx,
                        output_socket_id,
                        targets.source_component_id,
                    )
                    .await?;
                let output_socket_attribute_value_path =
                    AttributeValue::get_path_for_id(ctx, output_socket_attribute_value_id)
                        .await?
                        .ok_or(Error::EmptyPathForAttributeValue(
                            output_socket_attribute_value_id,
                        ))?;

                connections.push(Connection::Socket {
                    from_component_id: targets.source_component_id,
                    from_attribute_value_id: output_socket_attribute_value_id,
                    from_attribute_value_path: output_socket_attribute_value_path,
                    from_socket_id: output_socket_id,
                    from_socket_name: output_socket.name().to_string(),
                    to_component_id: component_id,
                    to_socket_id: input_socket_id,
                    to_socket_name: input_socket.name().to_string(),
                    to_attribute_value_id: input_socket_attribute_value_id,
                    to_attribute_value_path: input_socket_attribute_value_path.clone(),
                });
            }
        }
    }

    Ok(connections)
}

async fn incoming_prop_connections(
    _ctx: &DalContext,
    _component_id: ComponentId,
) -> Result<Vec<Connection>> {
    let connections = Vec::new();

    Ok(connections)
}

async fn outgoing_socket_connections(
    _ctx: &DalContext,
    _component_id: ComponentId,
) -> Result<Vec<Connection>> {
    let connections = Vec::new();

    Ok(connections)
}

async fn outgoing_prop_connections(
    _ctx: &DalContext,
    _component_id: ComponentId,
) -> Result<Vec<Connection>> {
    let connections = Vec::new();

    Ok(connections)
}
