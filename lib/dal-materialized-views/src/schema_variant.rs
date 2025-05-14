use dal::{
    DalContext,
    InputSocket,
    OutputSocket,
    SchemaVariant,
    SchemaVariantId,
};
use si_frontend_mv_types::{
    InputSocket as InputSocketMv,
    OutputSocket as OutputSocketMv,
    Prop,
    schema_variant::{
        ConnectionAnnotation,
        SchemaVariant as SchemaVariantMv,
    },
};
use telemetry::prelude::*;

use crate::mgmt_prototype_view_list;

#[instrument(
    name = "dal_materialized_views.schema_variant",
    level = "debug",
    skip_all
)]
pub async fn assemble(ctx: DalContext, id: SchemaVariantId) -> super::Result<SchemaVariantMv> {
    let schema_variant = SchemaVariant::get_by_id(&ctx, id).await?;
    let schema_id = schema_variant.schema(&ctx).await?.id();
    let sv = schema_variant.into_frontend_type(&ctx, schema_id).await?;

    let mut input_sockets = Vec::with_capacity(sv.input_sockets.len());
    for socket_data in sv.input_sockets {
        let dal_socket = InputSocket::get_by_id(&ctx, socket_data.id).await?;
        let mut annotations = Vec::new();
        for annotation in dal_socket.connection_annotations() {
            annotations.push(ConnectionAnnotation {
                tokens: annotation.tokens,
            });
        }
        let socket = InputSocketMv {
            id: socket_data.id,
            name: socket_data.name,
            eligible_to_send_data: socket_data.eligible_to_send_data,
            annotations,
            arity: dal_socket.arity().to_string(),
        };
        input_sockets.push(socket);
    }
    input_sockets.sort_by_key(|s| s.id);

    let mut output_sockets = Vec::with_capacity(sv.output_sockets.len());
    for socket_data in sv.output_sockets {
        let dal_socket = OutputSocket::get_by_id(&ctx, socket_data.id).await?;
        let mut annotations = Vec::new();
        for annotation in dal_socket.connection_annotations() {
            annotations.push(ConnectionAnnotation {
                tokens: annotation.tokens,
            });
        }
        let socket = OutputSocketMv {
            id: socket_data.id,
            name: socket_data.name,
            eligible_to_receive_data: socket_data.eligible_to_receive_data,
            annotations,
            arity: dal_socket.arity().to_string(),
        };
        output_sockets.push(socket);
    }
    output_sockets.sort_by_key(|s| s.id);

    let mut props: Vec<Prop> = sv.props.into_iter().map(Into::into).collect();
    props.sort_by_key(|p| p.id);
    let mgmt_functions = mgmt_prototype_view_list::assemble(&ctx, id).await?;
    Ok(SchemaVariantMv {
        id: sv.schema_variant_id,
        schema_variant_id: sv.schema_variant_id,
        schema_id,
        schema_name: sv.schema_name,
        version: sv.version,
        display_name: sv.display_name,
        category: sv.category,
        description: sv.description,
        link: sv.link,
        color: sv.color,
        input_sockets,
        output_sockets,
        props,
        is_locked: sv.is_locked,
        timestamp: sv.timestamp,
        can_create_new_components: sv.can_create_new_components,
        can_contribute: sv.can_contribute,
        mgmt_functions,
    })
}
