use std::sync::Arc;

use axum::{
    extract::{State, WebSocketUpgrade, ws::WebSocket},
    response::IntoResponse,
};
use dal::WorkspacePk;
use nats_multiplexer_client::MultiplexerClient;
use si_data_nats::NatsClient;
use telemetry::prelude::*;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

use crate::WsError;
use sdf_core::nats_multiplexer::NatsMultiplexerClients;
use sdf_extract::{
    request::TokenFromQueryParam,
    services::Nats,
    workspace::{TargetWorkspaceIdFromToken, WorkspaceAuthorization},
};

pub mod proto;

pub async fn bifrost_handler(
    wsu: WebSocketUpgrade,
    Nats(nats): Nats,
    _: TokenFromQueryParam,
    _: TargetWorkspaceIdFromToken,
    auth: WorkspaceAuthorization,
    State(shutdown_token): State<CancellationToken>,
    State(channel_multiplexer_clients): State<NatsMultiplexerClients>,
) -> Result<impl IntoResponse, WsError> {
    Ok(wsu.on_upgrade(move |socket| {
        run_bifrost_proto(
            socket,
            nats,
            auth.workspace_id,
            channel_multiplexer_clients.data_cache,
            shutdown_token,
        )
    }))
}

async fn run_bifrost_proto(
    mut socket: WebSocket,
    nats: NatsClient,
    workspace_pk: WorkspacePk,
    bifrost_multiplexer_client: Arc<Mutex<MultiplexerClient>>,
    shutdown_token: CancellationToken,
) {
    let proto = match proto::run(nats, workspace_pk, shutdown_token)
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
