use std::sync::Arc;

use axum::{
    extract::{
        State,
        WebSocketUpgrade,
        ws::WebSocket,
    },
    response::IntoResponse,
};
use dal::{
    DedicatedExecutor,
    WorkspacePk,
};
use frigg::FriggStore;
use sdf_core::nats_multiplexer::{
    EddaUpdatesMultiplexerClient,
    NatsMultiplexerClients,
};
use sdf_extract::{
    ComputeExecutor,
    Nats,
    request::TokenFromQueryParam,
    workspace::{
        TargetWorkspaceIdFromToken,
        WorkspaceAuthorization,
    },
};
use si_data_nats::ConnectionMetadata;
use telemetry::prelude::*;
use telemetry_utils::{
    counter,
    monotonic,
};
use tokio_util::sync::CancellationToken;

use crate::WsError;

pub mod proto;

#[allow(clippy::too_many_arguments)]
pub async fn bifrost_handler(
    wsu: WebSocketUpgrade,
    _: TokenFromQueryParam,
    _: TargetWorkspaceIdFromToken,
    auth: WorkspaceAuthorization,
    Nats(nats): Nats,
    State(frigg): State<FriggStore>,
    ComputeExecutor(compute_executor): ComputeExecutor,
    State(shutdown_token): State<CancellationToken>,
    State(channel_multiplexer_clients): State<NatsMultiplexerClients>,
) -> Result<impl IntoResponse, WsError> {
    Ok(wsu.on_upgrade(move |socket| {
        run_bifrost_proto(
            socket,
            nats.metadata_clone(),
            frigg,
            auth.workspace_id,
            channel_multiplexer_clients.edda_updates,
            compute_executor,
            shutdown_token,
        )
    }))
}

async fn run_bifrost_proto(
    mut socket: WebSocket,
    metadata: Arc<ConnectionMetadata>,
    frigg: FriggStore,
    workspace_pk: WorkspacePk,
    bifrost_multiplexer_client: EddaUpdatesMultiplexerClient,
    compute_executor: DedicatedExecutor,
    shutdown_token: CancellationToken,
) {
    monotonic!(sdf_bifrost_connections_opened = 1);
    counter!(sdf_bifrost_active_connections = 1);

    let proto = match proto::run(
        metadata,
        frigg,
        compute_executor,
        workspace_pk,
        shutdown_token,
    )
    .start(bifrost_multiplexer_client)
    .await
    {
        Ok(started) => started,
        Err(err) => {
            warn!(error = ?err, "protocol failed to start");
            return;
        }
    };

    let proto = match proto.process(&mut socket).await {
        Ok(processed) => processed,
        Err(err) => {
            trace!(error = ?err, "failed to cleanly complete bifrost stream");
            return;
        }
    };

    if let Err(err) = proto.finish(socket).await {
        warn!(error = ?err, "failed to finish protocol");
    }
}
