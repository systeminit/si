use crate::entity::{InferPropertiesRequest, InferPropertiesResponse};
use futures::{FutureExt, StreamExt};
use opentelemetry::global;
use opentelemetry_http::HeaderInjector;
use reqwest::Url;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use si_data::{EventLogFS, EventLogFSError};
use std::{net::ToSocketAddrs, sync::Arc};
use thiserror::Error;
use tracing::{
    debug,
    field::{display, Empty},
    instrument, Span,
};
use tracing_opentelemetry::OpenTelemetrySpanExt;
use tungstenite::http::Request;

#[derive(Error, Debug)]
pub enum VeritechError {
    #[error("bad automerge response")]
    BadAutomerge,
    #[error("binary websocket data is not supported")]
    Binary,
    #[error("event log fs error: {0}")]
    EventLogFS(#[from] EventLogFSError),
    #[error("http request error: {0}")]
    Http(#[from] tungstenite::http::Error),
    #[error("missing eventLog: {0}")]
    MissingEventLog(u64),
    #[error("cannot determine host from veritech http_url")]
    MissingHost,
    #[error("cannot determine port from veritech http_url")]
    MissingPort,
    #[error("no reply from veritech")]
    NoReply,
    #[error("failed to resolve pg hostname")]
    ResolveHostname(std::io::Error),
    #[error("resolved hostname returned no entries")]
    ResolveHostnameNoEntries,
    #[error("reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("tokio send error: {0}")]
    SendError(
        #[from]
        tokio::sync::mpsc::error::SendError<
            std::result::Result<
                tokio_tungstenite::tungstenite::protocol::Message,
                tokio_tungstenite::tungstenite::Error,
            >,
        >,
    ),
    #[error("serde error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("tokio task join error: {0}")]
    TokioJoin(#[from] tokio::task::JoinError),
    #[error("tungestenite: {0}")]
    Tungstenite(#[from] tungstenite::Error),
    #[error("url parse error: {0}")]
    UrlParse(#[from] url::ParseError),
    #[error("websocket error: {0}")]
    WebSocket(String),
}

pub type VeritechResult<T> = Result<T, VeritechError>;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ProtocolLog {
    pub fake_id: u64,
    //pub level: EventLogLevel,
    pub message: String,
    pub payload: serde_json::Value,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ProtocolOutputLine {
    pub fake_id: u64,
    pub event_log_id: u64,
    //pub stream: OutputLineStream,
    pub line: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Protocol<REP> {
    pub reply: Option<REP>,
    pub log: Option<ProtocolLog>,
    pub output_line: Option<ProtocolOutputLine>,
}

#[allow(dead_code)]
impl<REP> Protocol<REP> {
    fn has_reply(&self) -> bool {
        self.reply.is_some()
    }

    fn get_reply(self) -> REP {
        self.reply
            .expect("cannot get reply; should've checked with has_reply")
    }

    fn has_log(&self) -> bool {
        self.log.is_some()
    }

    fn get_log(self) -> ProtocolLog {
        self.log
            .expect("cannot get log; should've checked with has_log")
    }

    fn has_output_line(&self) -> bool {
        self.output_line.is_some()
    }

    fn get_output_line(self) -> ProtocolOutputLine {
        self.output_line
            .expect("cannot get output_line; should've checked with has_output_line")
    }
}

#[derive(Clone, Debug)]
struct ConnectionMetadata {
    http_user_agent: String,
    http_scheme: String,
    net_peer_ip: String,
    net_peer_port: u16,
    net_transport: &'static str,
    peer_service: &'static str,
}

#[derive(Clone, Debug)]
pub struct Veritech {
    ws_url: String,
    http_url: String,
    client: reqwest::Client,
    event_log_fs: EventLogFS,
    metadata: Arc<ConnectionMetadata>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
enum VeritechMessage<PROGRESS: std::fmt::Debug> {
    Protocol(PROGRESS),
}

impl Veritech {
    pub async fn new(
        settings: &si_settings::Veritech,
        event_log_fs: EventLogFS,
    ) -> VeritechResult<Self> {
        let ws_url = settings.ws_url.clone();
        let http_url = settings.http_url.clone();
        let client = reqwest::Client::new();

        let url = Url::parse(&http_url)?;
        let host = url.host_str().ok_or(VeritechError::MissingHost)?;
        let http_scheme = url.scheme().to_string();
        let net_peer_port = url
            .port_or_known_default()
            .ok_or(VeritechError::MissingPort)?;

        let resolving_hostname = format!("{}:{}", host, net_peer_port);
        let net_peer_ip = tokio::task::spawn_blocking(move || {
            resolving_hostname
                .to_socket_addrs()
                .map_err(VeritechError::ResolveHostname)
                .and_then(|mut iter| iter.next().ok_or(VeritechError::ResolveHostnameNoEntries))
                .and_then(|socket_addr| Ok(socket_addr.ip().to_string()))
        })
        .await??;

        let metadata = Arc::new(ConnectionMetadata {
            http_user_agent: user_agent(),
            http_scheme,
            net_peer_ip,
            net_peer_port,
            net_transport: "ip_tcp",
            peer_service: "si-veritech",
        });

        Ok(Self {
            ws_url,
            http_url,
            event_log_fs,
            client,
            metadata,
        })
    }

    #[instrument(
        name = "veritechclient.infer_properties",
        skip(self, request),
        fields(
            si.entity.r#type = %request.entity_type,
            si.entity.id = %request.entity.id,
        )
    )]
    pub async fn infer_properties(
        &self,
        request: InferPropertiesRequest,
    ) -> VeritechResult<InferPropertiesResponse> {
        let response: InferPropertiesResponse = self.send_sync("inferProperties", request).await?;
        Ok(response)
    }

    #[instrument(
        name = "veritechclient.send_sync",
        skip(self, path, request),
        fields(
            http.response_content_length = Empty,
            http.user_agent = %self.metadata.http_user_agent,
            http.flavor = "1.1",
            http.method = "POST",
            http.url = Empty,
            http.scheme = %self.metadata.http_scheme,
            http.status_code = Empty,
            http.target = Empty,
            net.peer.ip = %self.metadata.net_peer_ip,
            net.peer.port = %self.metadata.net_peer_port,
            net.transport = %self.metadata.net_transport,
            peer.service = %self.metadata.peer_service,
        )
    )]
    pub async fn send_sync<REQ, REP>(
        &self,
        path: impl AsRef<str>,
        request: REQ,
    ) -> VeritechResult<REP>
    where
        REQ: Serialize,
        REP: DeserializeOwned + std::fmt::Debug,
    {
        let span = Span::current();
        let path = path.as_ref();
        let full_url = format!("{}/{}", self.http_url, path);
        span.record("http.target", &display(&path));
        span.record("http.url", &display(&full_url));

        let mut request = self
            .client
            .post(&full_url)
            .header("User-Agent", &self.metadata.http_user_agent)
            .json(&serde_json::json!(&request))
            .build()?;
        global::get_text_map_propagator(|propagator| {
            propagator.inject_context(
                &span.context(),
                &mut HeaderInjector(&mut request.headers_mut()),
            )
        });

        let res = self.client.execute(request).await?;

        span.record("http.status_code", &res.status().as_u16());
        if let Some(ref content_length) = res.content_length() {
            span.record("http.response_content_length", content_length);
        }

        let response: REP = res.json().await?;
        Ok(response)
    }

    #[instrument(
        name = "veritechclient.send_async",
        skip(self, path, request),
        fields(
            http.response_content_length = Empty,
            http.user_agent = %self.metadata.http_user_agent,
            http.flavor = "1.1",
            http.method = "WS",
            http.url = Empty,
            http.scheme = %self.metadata.http_scheme,
            http.status_code = Empty,
            http.target = Empty,
            net.peer.ip = %self.metadata.net_peer_ip,
            net.peer.port = %self.metadata.net_peer_port,
            net.transport = %self.metadata.net_transport,
            peer.service = %self.metadata.peer_service,
        )
    )]
    pub async fn send_async<REQ, PROTOCOL>(
        &self,
        path: impl AsRef<str>,
        request: REQ,
        progress_tx: tokio::sync::mpsc::UnboundedSender<PROTOCOL>,
    ) -> VeritechResult<()>
    where
        REQ: Serialize,
        PROTOCOL: 'static + DeserializeOwned + std::fmt::Debug + Send,
    {
        let span = Span::current();
        let path = path.as_ref();
        let full_url = format!("{}/{}", &self.ws_url, path);
        span.record("http.target", &display(&path));
        span.record("http.url", &display(&full_url));

        let mut builder = Request::builder()
            .uri(full_url)
            .header("User-Agent", &self.metadata.http_user_agent);
        global::get_text_map_propagator(|propagator| {
            propagator.inject_context(
                &span.context(),
                &mut HeaderInjector(&mut builder.headers_mut().expect(
                    "RequestBuilder should not have an error--this would be a developer bug!",
                )),
            )
        });
        let client_request = builder.body(())?;

        let (ws_stream, _) = tokio_tungstenite::connect_async(client_request).await?;

        let (ws_tx, ws_rx) = tokio::sync::mpsc::unbounded_channel();

        let (ws_write, mut ws_read) = ws_stream.split();

        tokio::task::spawn(
            tokio_stream::wrappers::UnboundedReceiverStream::new(ws_rx)
                .forward(ws_write)
                .map(move |result| {
                    if let Err(ref err) = result {
                        // Doesn't look like `warp::Error` can deref the inner error, which is
                        // rather sad. The "Connection closed normall" 'error' appears to be safe
                        // and benign, so we'll ignore this one and warn on all others
                        match err.to_string().as_ref() {
                            "Connection closed normally" => {
                                dbg!("ws client send closed normally; err={:?}", err);
                            }
                            _ => {
                                dbg!("ws client send error; err={}", err);
                            }
                        }
                        result
                    } else {
                        result
                    }
                }),
        );

        let req = serde_json::to_string(&request).expect("failed making request json");

        tokio::spawn(async move {
            match ws_tx.send(Ok(tungstenite::protocol::Message::text(req))) {
                Ok(_) => {}
                Err(e) => {
                    dbg!(&e);
                }
            };
            while let Some(message_result) = ws_read.next().await {
                match message_result {
                    Ok(tungstenite::protocol::Message::Text(data)) => {
                        let reply: VeritechMessage<PROTOCOL> = match serde_json::from_str(&data) {
                            Ok(data) => data,
                            Err(e) => {
                                dbg!("failed to deserialize: {:?}", e);
                                continue;
                            }
                        };
                        match reply {
                            VeritechMessage::Protocol(progress) => {
                                match progress_tx.send(progress) {
                                    Ok(_) => {}
                                    Err(e) => {
                                        dbg!("cannot send progress down channel; abort!: {:?}", &e);
                                        return;
                                    }
                                }
                            }
                        }
                    }
                    Ok(tungstenite::protocol::Message::Binary(_)) => {
                        dbg!("received binary message; we only accept text");
                        return;
                    }
                    Ok(tungstenite::protocol::Message::Close(data)) => match data {
                        Some(frame) => match frame.code {
                            tungstenite::protocol::frame::coding::CloseCode::Normal => {
                                //dbg!("closed socket normally");
                            }
                            err => {
                                dbg!("request failed; err={:?}", err);
                                dbg!(&frame);
                                return;
                            }
                        },
                        None => {
                            dbg!("websocket closed for unknown reasons");
                            return;
                        }
                    },
                    Ok(tungstenite::protocol::Message::Ping(data)) => {
                        dbg!("ping; data={:?}", data);
                    }
                    Ok(tungstenite::protocol::Message::Pong(data)) => {
                        dbg!("pong; data={:?}", data);
                    }
                    Err(e) => {
                        dbg!("received an error: {:?}", &e);
                        return;
                    }
                }
            }
        });
        Ok(())
    }
}

fn user_agent() -> String {
    let uname = uname::uname().expect("Failed to get `uname` information");
    let user_agent = format!(
        "{}/{} (sysname:{}; release:{}; machine:{})",
        option_env!("CARGO_BIN_NAME").unwrap_or_else(|| env!("CARGO_PKG_NAME")),
        env!("CARGO_PKG_VERSION"),
        uname.sysname,
        uname.release,
        uname.machine,
    );
    debug!(http.user_agent = user_agent.as_str(), "veritech user-agent");

    user_agent
}
