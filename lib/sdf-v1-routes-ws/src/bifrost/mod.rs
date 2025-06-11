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
use sdf_core::nats_multiplexer::{
    EddaUpdatesMultiplexerClient,
    NatsMultiplexerClients,
};
use sdf_extract::{
    ComputeExecutor,
    request::TokenFromQueryParam,
    services::Nats,
    workspace::{
        TargetWorkspaceIdFromToken,
        WorkspaceAuthorization,
    },
};
use si_data_nats::NatsClient;
use telemetry::prelude::*;
use tokio_util::sync::CancellationToken;

use crate::WsError;

pub mod proto;

#[allow(clippy::too_many_arguments)]
pub async fn bifrost_handler(
    wsu: WebSocketUpgrade,
    Nats(nats): Nats,
    _: TokenFromQueryParam,
    _: TargetWorkspaceIdFromToken,
    auth: WorkspaceAuthorization,
    ComputeExecutor(compute_executor): ComputeExecutor,
    State(shutdown_token): State<CancellationToken>,
    State(channel_multiplexer_clients): State<NatsMultiplexerClients>,
) -> Result<impl IntoResponse, WsError> {
    Ok(wsu.on_upgrade(move |socket| {
        run_bifrost_proto(
            socket,
            nats,
            auth.workspace_id,
            channel_multiplexer_clients.edda_updates,
            compute_executor,
            shutdown_token,
        )
    }))
}

async fn run_bifrost_proto(
    mut socket: WebSocket,
    nats: NatsClient,
    workspace_pk: WorkspacePk,
    bifrost_multiplexer_client: EddaUpdatesMultiplexerClient,
    compute_executor: DedicatedExecutor,
    shutdown_token: CancellationToken,
) {
    let proto = match proto::run(nats, compute_executor, workspace_pk, shutdown_token)
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
